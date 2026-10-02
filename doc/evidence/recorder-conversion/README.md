# Explicit recorder conversion compatibility

Issue #133 replaces intermediate JSON conversion at the existing v1/v2 and
collector boundaries with exhaustive typed field copies. Core retains authorized
view production; the recorder retains independent durable schemas and validation.
No wire version, game rule, supported card, fixture or failure expectation changes.

`crates/mtg-recorder/tests/core_conversion.rs` adds normal-discovery compatibility
checks. Both tests pass against the original JSON-mediated implementation
([baseline](baseline.log)) and the typed implementation. This is a behavior-
preserving refactor, not a new-rule red/green claim. Original handwritten
`episode.json[l]` / `structured.json[l]`, privacy tests and publication fault
injection tests remain unchanged and are exercised by full torture.

- `played_conversion_preserves_bytes_statistics_and_incomplete_error`: normal
  reset with seed 133, both seats Keep (CR 103.5), P1 concedes (CR 104.3a).
  Literal Keep submissions and returns `[1, -1]`; absent statistics stay absent;
  negative zero, minimum subnormal, a difficult decimal-roundtrip value and
  maximum finite f64 preserve bits through durable reload. Canonical bytes match
  the old conversion on identical core records. Unfinished capture is Incomplete.
- `legacy_conversion_preserves_statistics_truncations_and_quarantine`: normal
  reset and Keep; each external limit retains its literal reason and zero returns.
  Supplied metadata and float bits survive durable reload; missing footer and
  failed capture retain Incomplete. The comparison covers all serialized fields.

The old conversion is a compatibility comparator only, not an independent rules
oracle. Literal expectations above derive from the rules and existing trajectory
contract; unchanged handwritten fixtures and negative tests remain independent
wire/privacy/failure oracles. Applicable rules references are unchanged: this
change adds no mechanic or reference bridge behavior.

Reproduce inside the managed Linux container:

```sh
cargo test -p mtg-recorder --test core_conversion --locked
./scripts/torture.sh
```

README now describes the typed boundary. No setup or quickstart command changes.
Only partial R0002-B036/B037 ownership; original component and M2 gate acceptance
remain authoritative. Candidate review, full-suite results and protected delivery
receipts are recorded in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/133#issuecomment-5951166146).
