# Native full-pool policy acceptance — GH-208

The existing `LegalRandom` and `Heuristic` now handle all twenty frozen cards
and every delivered decision through the authoritative Game, Driver, Run and
native CLI. This is partial R0002-B010/B011/B020/B029/B039 delivery. Original
component #23, reference composition #209–#212 and gate #26 retain full acceptance;
this report does not certify M2, playing strength, training or performance.
Protected delivery and exact-main CI are recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/208#issuecomment-6074112807).

## Contract and independent expectations

Policy IDs are `legal-random-full-pool-v1` and `heuristic-full-pool-v1`.
Previous `*-surprise-v1` IDs reject. The RNG stays `legal-random-rng-v1` with
unchanged SplitMix64 primitive, seed domains and unbiased bounded sampling.
The three newly admitted casts are Firebrand Archer, Crackling Cyclops and
Viashino Pyromancer, whose entire rules paths were delivered by #205/#206.
No rules implementation, policy observation schema or collector is replaced.

Random trigger ordering samples a complete permutation without replacement;
heuristic ordering preserves candidate order (bottom to top). Random player
targets include both players; heuristic prefers the opponent. Both reject missing,
duplicate or incomplete order domains and enabled unknown content without pass
fallback. Other strategic preferences remain unchanged. In particular, heuristic
never voluntarily mulligans or floats mana at priority; supporting a continuation
is not a promise that its strategy visits every continuation in every match.

Expectations come from RFC0002 §§3/8/9, the pinned card manifest, CR603 and
CR704.5a/b, and the existing policy RNG/privacy contract. The short ordered-deck
script uses normal reset and literal checkpoints: Pyromancer deals two, then
Archer deals one; Cyclops is 3/4 and Fodder creates two 1/1 Goblins. It is separate
from the reproducibility oracle. Fixed-seed agreement alone is not rules proof.

## Executable acceptance

All tests below are in normal discovery and run in both torture profiles.

| Requirement | Executable evidence |
| --- | --- |
| Twenty-card admission and strict unknown-content errors | `mtg-policy/tests/full_pool.rs::full_pool_trigger_creatures_are_supported_without_pass_fallback`; prior per-card policy tests retained |
| Full trigger permutation / literal RNG vector / fixed heuristic strategy | `full_pool_trigger_permutation_uses_existing_rng_and_stable_heuristic_order`; seat0 published words yield rows 2,0,1 |
| Player targeting, missing choice, malformed order and version errors | `full_pool_player_targets_and_missing_choices_are_explicit`, `full_pool_trigger_order_rejects_incomplete_or_duplicate_permutations`, `full_pool_policy_versions_reject_previous_support_contract` |
| Hand-authored normal-reset checkpoints and hidden hand/library twins | `full_pool_scripted_trigger_checkpoints_and_hidden_twins`; both policies receive identical full permitted observations and produce identical proposals, including trigger order/target; opponent hands actually differ |
| Actual games, all 18 decision kinds, capture/replay/quantum and rejection invariance | `mtg-recorder/tests/full_pool_policy.rs::full_pool_native_games_replay_capture_quantum_and_rejections` |
| Existing CLI / no TTY / reproducible games | `mtg-cli/tests/native_simulate.rs::full_pool_cli_all_matchups_both_policies_and_starting_seats` |
| Limits stay distinct from rules outcomes; missing policies never default | Retained `native_one_decision_cannot_be_a_rules_outcome`, `native_explicit_work_and_record_limits_account_exactly`, `native_bad_versions_policies_bounds_and_overflow_fail_without_output` |
| Modal/discard/ability/combat distributions and literal strategy | Retained `mtg-policy/tests/selection.rs`, `heuristic.rs`, `games.rs`, `heuristic_games.rs`; prior normal-reset Shivan/Invoker/Surprise/Thrill recorder tests retained |

The game matrix uses environment master 42, policy master 42, ordinal 7; both
policies separately in red/green, red/red and green/green, with starting seats
0 and 1. Every game must terminate by rules within 20,000 decisions; declared
losses independently satisfy life ≤ 0 or attempted empty-library draw. Each
configuration repeats with capture enabled at quantum MAX and 1, then disabled
at MAX. Histories, final permitted observations and captured episodes agree.
Canonical `Run::persist` JSONL reload equals typed conversion; the restricted
semantic replay reconstructs the game. Twelve CLI configurations repeat exactly.

Every decision kind reached by this matrix is also consumed by both policies
on separately restored authoritative Game snapshots, using fresh revision tokens;
those proposals must apply successfully. This covers heuristic responses to
randomly reached bottoming/cleanup/activation-payment choices without changing
its strategy. Empty, illegal, wrong-seat and stale submissions preserve state;
first-per-family rejection also preserves history and capture. Snapshot restoration
is supplemental validation, not a substitute for the ordinary-reset games.

Compiled preimplementation failures are retained in [red.log](red.log) and
[order-red.log](order-red.log). Initial test-harness compile/status/replay-ID mistakes
were repaired and are not behavioral-red evidence. The existing two policy tests
that called Pyromancer unsupported now use `unknown-content`: #206 delivered that
card, and this issue admits it to the native policies. Every prior rejection
assertion is retained, supplemented by positive card and new decision tests;
independent review must assess this explicit requirement correction.

## Reference boundary

No new mechanic or reference bridge is implemented. The unchanged catalog
crosswalk remains authoritative, with aggregate/composition execution assigned
to #209–#212. This change re-executes the references for the newly admitted trigger
cards: [22 cast-trigger cases](cast-reference.json) and
[14 ETB cases](etb-reference.json), twice native and pinned XMage
`000d8a7abc0ac31cc24af08691423e0c24dc59e7`, with comparator negative controls.
These are selected synthetic rule checkpoints, not native-policy games in XMage,
full-game reference agreement or a new Forge claim. Normal-reset game evidence
is supplied separately above. Existing reference expectations and source pins
are unchanged; receipts include fixture, bridge and native-source hashes.

## Reproduce in the managed container

```sh
cargo test --locked -p mtg-policy
cargo test --locked -p mtg-recorder --test full_pool_policy -- --nocapture
cargo test --locked -p mtg-cli --test native_simulate
python3 scripts/etb_trigger_reference.py --cache /home/agent/.cache/xmage --output /tmp/full-pool-etb
python3 scripts/cast_trigger_reference.py --cache /home/agent/.cache/xmage --output /tmp/full-pool-cast
./scripts/torture.sh
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/native-v2.json </dev/null
```

README, capability/policy contracts and the native quickstart now describe the
full supported pool and new policy versions. No stage-table verdict, command
schema, setup, CI, workflow, authorization or budget changes are made.

The updated native quickstart was executed headlessly: two completed games,
zero truncated/failed/incomplete episodes. Full torture and independent review
results, exact candidate SHA, protected PR and exact-main CI are recorded in the
linked issue workpad rather than inferred from this report's existence.
