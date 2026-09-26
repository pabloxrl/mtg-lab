# Synthetic checkpoint comparator v1

This is a verifier self-test foundation for [GH-1](https://github.com/pabloxrl/mtg-lab/issues/1),
following [RFC 0001](rfcs/0001-project-charter.md) and
[RFC 0002](rfcs/0002-first-mvp.md). Both checkpoints are supplied by the author.
No game simulation, reachable-state validation, card semantics, reference-engine
execution, or Magic rules conformance is implemented.

## Run

```sh
cargo build -p mtg-fixture --locked
./target/debug/mtg-fixture fixtures/comparator/equal.json </dev/null
```

The binary accepts exactly one file path (including paths beginning with a dash).
It never reads stdin, uses a display, or prompts. Input must be a finite UTF-8 JSON
file; do not supply a FIFO or device that waits for a producer. The whole file is
loaded into memory, so this initial tool is intended for small fixtures.

## Schema version 1

All fields below are required. Unknown and duplicate fields are rejected at every
object level. No null values or type coercions are accepted.

```json
{
  "schema_version": 1,
  "fixture_id": "synthetic/example",
  "expected": {
    "life": {"p1": 20, "p2": 17},
    "active_player": "p1",
    "stack": ["spell-a", "ability-b"]
  },
  "actual": {
    "life": {"p1": 20, "p2": 17},
    "active_player": "p1",
    "stack": ["spell-a", "ability-b"]
  }
}
```

- `schema_version`: integer 1. Other unsigned 32-bit versions produce
  `unsupported_version` once the fixture shape is valid; invalid types/shapes
  produce `invalid_fixture`.
- `fixture_id`: stable, author-assigned, nonblank string; preserved verbatim.
  Keep it unchanged when rerunning the same case. No registry or cross-file
  uniqueness check exists.
- `expected`, `actual`: checkpoint objects with identical required structure.
- `life`: exactly the player keys `p1` and `p2`, each a signed 32-bit integer.
  Negative and zero life are valid synthetic data.
- `active_player`: exactly `"p1"` or `"p2"`.
- `stack`: array of opaque strings, in bottom-to-top order. Empty arrays,
  repeated entries, and empty strings are permitted. Strings have no inferred
  card meaning and are compared exactly, without Unicode normalization.

Object key order does not affect comparison. The first difference is selected
in this fixed order: `life.p1`, `life.p2`, `active_player`, then stack indices
from zero. A missing entry on the shorter side is reported as JSON `null`;
null cannot be an input stack entry, so absence is unambiguous.

## Output and exit codes

Exactly one compact JSON object plus newline is written to stdout for all normal
results and diagnosed input failures. Stderr is empty. There are no timestamps,
paths, terminal escapes, or runtime-generated IDs in output. With the locked
dependencies, identical input yields identical bytes. JSON key ordering is not
an API promise; consumers should parse JSON.

| Exit | Status | Fields |
| --- | --- | --- |
| 0 | `pass` | `fixture_id` |
| 1 | `mismatch` | `fixture_id`, `field`, `expected`, `actual` |
| 2 | `error` | `code`, `message` |

Error codes are `usage` (wrong argument count), `io` (file cannot be read),
`invalid_fixture` (JSON/schema/identity failure), and `unsupported_version`.
Parser messages include details and line/column when available. Invalid inputs
do not report a fixture ID because it has not been validated. A write failure,
such as a closed output pipe, also exits 2 but cannot guarantee delivery of JSON.

Example mismatch:

```json
{"actual":19,"expected":20,"field":"life.p1","fixture_id":"synthetic/life","status":"mismatch"}
```

## Independent evidence and scope

The five examples in `fixtures/comparator/` are original, manually specified
synthetic comparisons. Their oracle is GH-1's equality requirements, not game
rules or an engine's generated output:

| Fixture | Independent expected result |
| --- | --- |
| `equal.json` | Identical checkpoint fields pass |
| `life.json` | 20 versus 19 differs at `life.p1` |
| `active-player.json` | p1 versus p2 differs at `active_player` |
| `stack-entry.json` | ability-b versus ability-c differs at `stack[1]` |
| `stack-order.json` | Swapping spell-a and ability-b differs at `stack[0]` |

The integration tests execute the actual binary with stdin closed and display
variables removed, enforce a five-second timeout, and independently assert exit
codes and JSON values. They cover first-difference precedence, both life totals,
stack length, invalid versions/types/identities, missing/unknown/duplicate fields,
positional arrays and object-form player values, trailing data, usage, I/O failure, and repeated-run determinism. Unit tests check
typed comparison and duplicate nested fields.

Red evidence before implementation: all five integration tests compiled and
failed against a no-output scaffold. Mismatches returned 0 instead of 1, invalid
inputs returned 0 instead of 2, and the equal case lacked JSON. After implementation,
`./scripts/verify.sh` runs these same tests plus unit tests, formatting, linting,
and documentation checks. The issue workpad records delivery and review evidence.

Independent review found that derived Serde structs/enums accepted positional
arrays and object-form player values. A regression first reproduced an incorrect
pass for an array fixture; explicit shape validation now rejects these forms
while the original-byte typed parse retains duplicate-field detection.

Future integration remains [backlog #4](https://github.com/pabloxrl/mtg-lab/issues/4): a separately versioned scenario schema with
rules/card pins, provenance, choices, named checkpoints and capability coverage;
engine-produced checkpoints; and pinned, headless XMage/Forge bridges. None is
advertised by this comparator or counted as a skipped supported test.
