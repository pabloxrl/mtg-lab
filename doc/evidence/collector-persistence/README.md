# Actual-result persistence acceptance

Related to #163; bounded component acceptance, not M1/full collector completion.
Normal discovery: `cargo test -p mtg-recorder --test collector`. No existing tests
were removed, skipped or weakened. No rules changes or new reference-engine claim.

Independent oracle: normal frozen red/green reset, seed **163**, ordinals **7/8**.
CR 103.5 keeps require exactly one accepted Keep from each seat: decisions `(0,0)`
and `(1,1)`. Advance the opening-complete internal boundary without a policy action,
then P1 concedes (CR 104.3a): returns/boundary reward `[1,-1]`, exactly two policy
decisions. Two completed episodes therefore store exactly four decisions. An
external decision limit of two instead ends immediately after the two keeps:
Truncated(Decisions), zero returns, no invented concession or rules loss.

Mixed run ordinals **7/8/9**: completed script; record ceiling one rejects the
second Keep and yields Failed(RecordCapacity); unfinished opening yields Incomplete.
Only ordinal 7 may be loaded, only with explicit diagnostic mode. These are actual
normal-reset driver results, not fabricated episodes. Original histories, snapshots
and trajectories remain available after persistence errors.

The first fixture attempt incorrectly conceded before advancing the existing
OpeningComplete internal boundary; it failed with SettlementPending and was not
counted as evidence. After correcting that script against the delivered driver
contract, the compiling persistence scaffold returned Incomplete. The retained
[behavioral red](behavioral-red.txt) has four intended failures: completed,
truncated and mixed persistence plus missing I/O propagation; one rejection test
already passed. No compile/import failure is claimed as a behavioral red.

An additional [compiled red](internal-work-red.txt) demonstrates that a real
injected-clock expiry during internal reset initially returned an error rather
than preserving explicit Incomplete recording accounting. The regression now
checks the diagnostic-only zero-row bundle and the unchanged Truncated result.

The eleven normal-discovery tests cover:

- Literal completed/truncated decisions/statuses/rewards plus exact equality to
  the original canonical capture for every stored field, and full sealed-byte hash/counts.
- Mixed failure/incomplete and zero-row sealed accounting; default versus diagnostic loading.
- Missing/duplicate/reordered results, config/policy/limit and capture-provenance mismatch.
- Independent Python hashlib constants for compact sorted frozen deck JSON,
  exact compiled rules/cards pins, actual snapshot engine identity and Config hash.
- Existing writer fail-on-overflow and block/drain behavior, row capacity, write
  failures during append/drain/seal, final flush failure, total-byte limits,
  successful metrics and preservation of borrowed results.
- Declared synthetic broken sinks mutate actual played-game output after flush:
  corrupt JSON/checksum, missing required field, unsupported version, missing seal,
  and re-sealed unrelated policy provenance. No invalid output yields a bundle.

README and API documentation distinguish owned sealed bytes from durable run
publication. CLI/quickstart commands and milestone verdicts remain unchanged;
existing command regression checks run in the full torture suite. Full suite and
candidate review/delivery receipts are recorded in the issue workpad and PR;
verification hashes accompany the committed evidence. #164 owns publication;
#154/#117/#20/#120/#21/#22 and later gates retain their original obligations.
