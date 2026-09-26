use serde_json::{Value, json};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

fn run(args: &[&std::ffi::OsStr]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mtg-fixture"))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .spawn()
        .unwrap();
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(5) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("CLI timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    child.wait_with_output().unwrap()
}

fn input(text: &str) -> Output {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "mtg-fixture-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::write(&path, text).unwrap();
    let result = run(&[path.as_os_str()]);
    std::fs::remove_file(path).unwrap();
    result
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/comparator/equal.json")).unwrap()
}

fn assert_result(out: Output, code: i32, expected: Value) {
    assert_eq!(out.status.code(), Some(code), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap(),
        expected
    );
    assert_eq!(out.stdout.last(), Some(&b'\n'));
}

#[test]
fn examples_detect_independently_specified_differences() {
    for (name, field, expected, actual) in [
        ("life", "life.p1", json!(20), json!(19)),
        ("active-player", "active_player", json!("p1"), json!("p2")),
        (
            "stack-entry",
            "stack[1]",
            json!("ability-b"),
            json!("ability-c"),
        ),
        (
            "stack-order",
            "stack[0]",
            json!("spell-a"),
            json!("ability-b"),
        ),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../fixtures/comparator/{name}.json"));
        assert_result(
            run(&[path.as_os_str()]),
            1,
            json!({
                "status":"mismatch", "fixture_id":format!("synthetic/{name}"),
                "field":field, "expected":expected, "actual":actual
            }),
        );
    }
}

#[test]
fn equal_is_deterministic_and_preserves_id() {
    let text = fixture().to_string();
    let first = input(&text);
    let second = input(&text);
    assert_eq!(first.stdout, second.stdout);
    assert_result(
        first,
        0,
        json!({"status":"pass","fixture_id":"synthetic/equal"}),
    );
    let mut value = fixture();
    value["fixture_id"] = json!("another/id");
    value["expected"]["life"]["p2"] = json!(-4);
    value["actual"]["life"]["p2"] = json!(-4);
    value["expected"]["stack"] = json!([]);
    value["actual"]["stack"] = json!([]);
    assert_result(
        input(&value.to_string()),
        0,
        json!({"status":"pass","fixture_id":"another/id"}),
    );
}

#[test]
fn first_difference_and_stack_length_are_explicit() {
    for (actual, field, expected, observed) in [
        (
            json!({"life":{"p1":0,"p2":1},"active_player":"p2","stack":[]}),
            "life.p1",
            json!(20),
            json!(0),
        ),
        (
            json!({"life":{"p1":20,"p2":1},"active_player":"p2","stack":[]}),
            "life.p2",
            json!(17),
            json!(1),
        ),
        (
            json!({"life":{"p1":20,"p2":17},"active_player":"p1","stack":["spell-a"]}),
            "stack[1]",
            json!("ability-b"),
            Value::Null,
        ),
        (
            json!({"life":{"p1":20,"p2":17},"active_player":"p1","stack":["spell-a","ability-b","extra"]}),
            "stack[2]",
            Value::Null,
            json!("extra"),
        ),
    ] {
        let mut value = fixture();
        value["actual"] = actual;
        assert_result(
            input(&value.to_string()),
            1,
            json!({
                "status":"mismatch","fixture_id":"synthetic/equal",
                "field":field,"expected":expected,"actual":observed
            }),
        );
    }
}

#[test]
fn invalid_inputs_are_structured_and_deterministic() {
    let mut cases = vec![
        "{".to_owned(),
        "{}".to_owned(),
        "null".to_owned(),
        format!("{} false", fixture()),
        fixture().to_string().replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        ),
    ];
    for pointer in [
        "/schema_version",
        "/fixture_id",
        "/expected",
        "/actual",
        "/expected/life",
        "/actual/life/p1",
        "/expected/life/p2",
        "/actual/active_player",
        "/expected/stack",
    ] {
        let mut v = fixture();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        v.pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        cases.push(v.to_string());
    }
    for (pointer, value) in [
        ("/schema_version", json!(2)),
        ("/schema_version", json!("1")),
        ("/fixture_id", json!(" ")),
        ("/actual/active_player", json!("p3")),
        ("/expected/active_player", json!("P1")),
        ("/actual/life/p1", json!(1.5)),
        ("/actual/life/p2", json!(2147483648_i64)),
        ("/expected/stack", json!([null])),
        ("/actual/life", json!({"p1":20,"p3":17})),
    ] {
        let mut v = fixture();
        *v.pointer_mut(pointer).unwrap() = value;
        cases.push(v.to_string());
    }
    for pointer in ["", "/expected", "/actual", "/expected/life", "/actual/life"] {
        let mut v = fixture();
        v.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(0));
        cases.push(v.to_string());
    }
    for text in cases {
        let first = input(&text);
        let second = input(&text);
        assert_eq!(first.status.code(), Some(2), "{text}: {first:?}");
        assert!(first.stderr.is_empty());
        assert_eq!(first.stdout, second.stdout);
        let diagnostic: Value = serde_json::from_slice(&first.stdout).unwrap();
        assert_eq!(diagnostic["status"], "error");
        assert!(diagnostic["code"].as_str().is_some());
        assert!(!diagnostic["message"].as_str().unwrap().is_empty());
    }
}

#[test]
fn usage_and_io_errors_are_json() {
    for args in [vec![], vec!["one".as_ref(), "two".as_ref()]] {
        assert_result(
            run(&args),
            2,
            json!({"status":"error","code":"usage",
            "message":"expected exactly one fixture file path"}),
        );
    }
    let missing = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("does-not-exist.json");
    assert_result(
        run(&[missing.as_os_str()]),
        2,
        json!({"status":"error","code":"io",
        "message":"could not read fixture file"}),
    );
}
