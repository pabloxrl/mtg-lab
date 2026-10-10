# Four-mode measurement campaign (GH-281)

Work in progress; no campaign or M2 pass is claimed. The [frozen plan](plan.md)
defines the experiment, statistics and its limits. The campaign consumes the
existing collectors and diagnostic adapters; it adds no production engine or
benchmark capability.

Independent accounting: 60 natural completions and one unfinished game per 30s
means 2 completed games/s and all 61 attempts in the ledger. Sixteen configurations
with five windows therefore mean 4,800 completions, 80 unfinished attempts and
2,400 measured seconds, excluding separately reported warmup. A failed attempt's
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
