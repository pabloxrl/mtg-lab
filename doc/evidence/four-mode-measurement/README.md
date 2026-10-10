# Four-mode measurement campaign (GH-281)

Work in progress; no campaign or M2 pass is claimed. The [frozen plan](plan.md)
defines the experiment, statistics and its limits. The campaign consumes the
existing collectors and diagnostic adapters; it adds no production engine or
benchmark capability.

Independent accounting: 60 natural completions and one unfinished game per 30s
means 2 completed games/s and all 61 attempts in the ledger. Sixteen configurations
with five windows therefore mean 4,800 completions, 60 unfinished attempts and
2,400 measured seconds, excluding separately reported warmup. The four full-replay
configurations require complete recordings, with no unfinished replay. A failed attempt's
29 units of work after a successful attempt's 17 gives denominator 46, not 17.
Full replay distinguishes verified natural/concession endings from incomplete
work. Diagnostic capacity drops are not incomplete replay. RFC B020/B021 supply
these expectations; they are not derived from the engine's output.

[Behavioral red](red.txt) records five intended `ValueError not raised` failures
when only the existing per-report validator was applied to literal small data:
missing modes, missing policies, a changed binary, an undeclared extra run and
changed resources. Other literal controls already rejected malformed rows,
windows, horizon, failed work, denominator, replay, trace and capacity/persistence.
An earlier module-existence probe failed before an adapter existed; that is not
behavioral red evidence and is not used as the acceptance basis.

Normal discovery includes `test_m2_repair_measurement.py`: literal campaign
arithmetic and corruption controls plus actual exported mode/capture, native
latency/semantic and profile evidence. New Rust tests exercise the real native
policy/encoding/mode matrix and a literal injected-clock failed-work ledger.
Tests never assert a measured speed as a required engine outcome.

The benchmark itself honestly exports sampled timing as `not_measured`. The
separate fixed-episode sampler supplies bucket bounds, which are not window
histograms. Its native test executable is separately pinned. The eight-row
benchmark cycles rows and does not export per-row elapsed denominators; no
per-row throughput is inferred. Capture equality and complete replay verification
are checked on fixed episodes, never inferred from timed-window totals.

Historical [GH-217 resident/stress evidence](../full-pool-baseline/README.md) remains
unchanged. This task changes no production buffer or game representation, so its
core-state capacity evidence is retained with its original binary and limits;
it is not a claim about concurrent replay/capture buffer capacity. No rules or
reference adapter changes require new XMage execution. Reuse the delivered
[policy reference basis](../full-pool-policy/README.md),
[trace/replay receipt](https://github.com/pabloxrl/mtg-lab/issues/215#issuecomment-6079735470)
and fixed-profile oracle links.

The [precollection review](precollection-review.json) found no removed/weakened
tests and confirmed the root README remained appropriately limited. It correctly
requested changes because the campaign results were not yet present. This is a
preparation review, not delivery approval; final review must cover the completed
report and exact candidate.

A subsequent [literal metadata red](resolved-pins-red.txt) exposed three additional
validator gaps before collection: configuration policy/mode could disagree with
the resolved native input, and an encoded track could report zero encoded bytes.
The added controls now reject those artifacts; no engine or runner changed.

The native Config schema intentionally omits `instrumentation` when it is `off`
(`skip_serializing_if = Mode::is_off`). The [positive omission regression](off-omission-red.txt)
prevents the new consistency check from rejecting that documented representation;
missing mode in enabled configurations still mismatches and rejects. This is the
existing versioned serialization rule, not a fallback for unsupported input.

A final [incomplete-replay red](incomplete-replay-red.txt) preserves an important
requirement correction for independent review: the initial new synthetic fixture
incorrectly labeled a full-replay window with one incomplete recording successful.
RFC B021 and the reviewed GH-279 contract require complete promised replay or
explicit failure. The campaign now rejects even **consistently counted** incomplete
replay, rather than detecting only a mismatched verified count. The synthetic
full-replay rows now contain 60 complete attempts; the other twelve configurations
still exercise unfinished-work accounting. Their expected unfinished total changes
from 80 to 60; the natural-completion numerator remains 4,800. This corrects the
new fixture against independent requirements, not engine output. Every pre-existing
regression/expectation remains unchanged. Final independent review must explicitly
check this correction and its stronger negative coverage.

Diagnosis after these validator repairs: provenance and recording availability
need checks against the frozen serializer and complete-replay contract in addition
to arithmetic identities. The revised approach retains positive omission controls,
consistent-but-invalid negative ledgers and real exported artifacts; it changes
no engine, collector or benchmark feature and weakens no acceptance requirement.

The [slow-warmup positive red](slow-warmup-red.txt) rejects an accidental extra
speed gate in this PR's first validator: B020 requires ten warmup seconds, not
eight warmup games. The corrected validator requires all eight rows across the
measured windows and preserves exact per-window row accounting. The frozen
10s/5×30s plan and original requirements are unchanged; slow or high-variance
results remain results rather than grounds for an invented minimum game rate.

Collection began at 11:00:55 UTC just before the last warmup-validator correction
was saved. The experimental plan, thresholds, native binary and existing collector
were unchanged. The manifest's original validator hash is preserved byte-exact in
[validator-at-start.txt](validator-at-start.txt). Keep the original collection status
and any validation failure; final analysis separately revalidates all raw data with
the corrected requirements. Do not overwrite the original manifest or call a
validation error a native game failure. No run is retried or selected for speed.
