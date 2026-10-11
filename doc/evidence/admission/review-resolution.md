# Independent review repair

[Review 1](review-1.json) rejected candidate
`10a271c88e68dc36af54af840ae574469fef1bce` with two blocking findings.
Its local passing results are retained as candidate-v4 and are not final
admission evidence. The [regression red](review-regressions-red.log) records
three intended assertion failures before either repair; no import or compile
failure is counted as behavioral evidence.

The omitted-effect finding is addressed by `xmage_effect_lifecycle` in
`scripts/admission_observations.py`. An independent ledger reads witnessed
stack resolutions and uses the pinned card/effect implementations to require
effect presence through its lifetime. It checks primary, pre-SBA, cleanup and
terminal observations. CR608.2b controls legal targets and CR514.2 controls
cleanup expiry. The consulted effect-source hashes are in
`references/xmage/admission-provenance.json`. Tests delete every effect and
reintroduce expired effects; both must fail even with unchanged outcomes.

The trigger finding is addressed by `xmage_trigger_provenance`. Each trigger
must have a callback trace, a frozen source/event key, a witnessed occurrence
UUID and matching zone-change counter. Stack witnesses must match the trace
and raw stack index. Tests omit the trace and independently rebind raw source
identities and counters. Token counters come from actual token-creation
callbacks; they are never guessed from a missing deck-construction counter.

All five added regressions run in ordinary Python discovery. A hash-bound
historical XMage payload is used solely as a corruption witness; it is not a
golden expected state. The same five mutations also run against fresh XMage
observations in each complete admission invocation. Positive acceptance still
requires real native replay in focused discovery and two complete fresh
native/XMage invocations, followed by full torture and another independent
exact-candidate review. No test, authored checkpoint or source pin was weakened.
