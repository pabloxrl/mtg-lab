use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::Write;
use std::process::ExitCode;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    fixture_id: String,
    expected: Checkpoint,
    actual: Checkpoint,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    life: Life,
    active_player: Player,
    stack: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Life {
    p1: i32,
    p2: i32,
}

#[derive(Deserialize, Serialize, PartialEq)]
enum Player {
    #[serde(rename = "p1")]
    P1,
    #[serde(rename = "p2")]
    P2,
}

fn error(code: &str, message: &str) -> (u8, Value) {
    (2, json!({"status":"error", "code":code, "message":message}))
}

fn compare(fixture: &Fixture) -> (u8, Value) {
    let expected = &fixture.expected;
    let actual = &fixture.actual;
    let mismatch = |field: String, expected: Value, actual: Value| {
        (
            1,
            json!({"status":"mismatch", "fixture_id":fixture.fixture_id,
            "field":field, "expected":expected, "actual":actual}),
        )
    };
    for (field, left, right) in [
        ("life.p1", expected.life.p1, actual.life.p1),
        ("life.p2", expected.life.p2, actual.life.p2),
    ] {
        if left != right {
            return mismatch(field.into(), json!(left), json!(right));
        }
    }
    if expected.active_player != actual.active_player {
        return mismatch(
            "active_player".into(),
            json!(expected.active_player),
            json!(actual.active_player),
        );
    }
    for index in 0..expected.stack.len().max(actual.stack.len()) {
        let left = expected.stack.get(index);
        let right = actual.stack.get(index);
        if left != right {
            return mismatch(format!("stack[{index}]"), json!(left), json!(right));
        }
    }
    (0, json!({"status":"pass", "fixture_id":fixture.fixture_id}))
}

fn verify(bytes: &[u8]) -> (u8, Value) {
    // Serde's derived structs/enums also accept positional arrays/tagged objects.
    // Check JSON shapes first, then deserialize the original bytes so duplicate
    // keys are still rejected rather than lost in Value's map representation.
    let shape: Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(cause) => return error("invalid_fixture", &cause.to_string()),
    };
    for pointer in ["", "/expected", "/actual", "/expected/life", "/actual/life"] {
        if !shape.pointer(pointer).is_some_and(Value::is_object) {
            return error(
                "invalid_fixture",
                &format!(
                    "{} must be an object",
                    if pointer.is_empty() { "/" } else { pointer }
                ),
            );
        }
    }
    for pointer in ["/expected/active_player", "/actual/active_player"] {
        if !shape.pointer(pointer).is_some_and(Value::is_string) {
            return error("invalid_fixture", &format!("{pointer} must be a string"));
        }
    }
    let fixture: Fixture = match serde_json::from_slice(bytes) {
        Ok(fixture) => fixture,
        Err(cause) => return error("invalid_fixture", &cause.to_string()),
    };
    if fixture.schema_version != 1 {
        return error("unsupported_version", "schema_version must be 1");
    }
    if fixture.fixture_id.trim().is_empty() {
        return error("invalid_fixture", "fixture_id must not be blank");
    }
    compare(&fixture)
}

fn run() -> (u8, Value) {
    let mut args = std::env::args_os().skip(1);
    let path = args.next();
    if path.is_none() || args.next().is_some() {
        return error("usage", "expected exactly one fixture file path");
    }
    match std::fs::read(path.unwrap()) {
        Ok(bytes) => verify(&bytes),
        Err(_) => error("io", "could not read fixture file"),
    }
}

fn main() -> ExitCode {
    let (code, result) = run();
    // A broken output pipe is a failure, even if the checkpoints were equal.
    if writeln!(std::io::stdout().lock(), "{result}").is_err() {
        return ExitCode::from(2);
    }
    ExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_comparison_checks_second_player_before_active_player() {
        let fixture = Fixture {
            schema_version: 1,
            fixture_id: "unit/life".into(),
            expected: Checkpoint {
                life: Life { p1: 20, p2: 17 },
                active_player: Player::P1,
                stack: vec![],
            },
            actual: Checkpoint {
                life: Life { p1: 20, p2: -1 },
                active_player: Player::P2,
                stack: vec![],
            },
        };
        assert_eq!(
            compare(&fixture),
            (
                1,
                json!({
                    "status":"mismatch","fixture_id":"unit/life",
                    "field":"life.p2","expected":17,"actual":-1
                })
            )
        );
    }

    #[test]
    fn parser_rejects_duplicate_nested_fields() {
        let bytes = br#"{"schema_version":1,"fixture_id":"duplicate",
            "expected":{"life":{"p1":20,"p1":19,"p2":17},"active_player":"p1","stack":[]},
            "actual":{"life":{"p1":20,"p2":17},"active_player":"p1","stack":[]}}"#;
        let (code, diagnostic) = verify(bytes);
        assert_eq!(code, 2);
        assert_eq!(diagnostic["code"], "invalid_fixture");
        assert!(
            diagnostic["message"]
                .as_str()
                .unwrap()
                .contains("duplicate field")
        );
    }
}
