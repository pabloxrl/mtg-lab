# Combat acceptance conflict — not a delivery report

GH-275 and the registered acceptance table require a positive
"Pathwarden haste/vigilance/trample" played prefix while limiting the path to
nontriggering creature casts and excluding activated abilities and spell
response implementation.

The unchanged frozen manifest defines Tajuru Pathwarden as a 5/4 costing
{4}{G}, with **Vigilance and Trample only**. Its pinned printing is
`da20a0d3-2022-4dea-84c8-85adc5a974f8`, and its content hash is
`c0763ac465569fdc180ede4e67c6e2a06157345d5bc223d2825483100f685d15`.
No frozen creature has printed haste. Cavalry grants haste through an activated
ability; Surprise grants it through a noncreature spell. Both require expanding
the stated creature-cast-only acceptance path. CR 302.6/702.10 does not allow
vigilance to substitute for haste.

An essential scope decision is pending: verify Pathwarden's sickness and retain
the existing #210 haste evidence, or explicitly authorize a legal haste source
and its played-prefix dependency. The feature worker cannot rewrite the
registered acceptance, invent a keyword, inject a haste effect, or silently
count a prior synthetic case as new played-prefix evidence.

Independent preparation completed: current-main program validation; prerequisite
completion/review/merge/exact-main CI checks; real native baseline test; pinned
issue-local XMage source preparation and callback inspection. The baseline
compiled and executed, then failed `precombat_main != combat_damage` as intended.
`baseline-test.py` and `combat-red.log` preserve the assertion and execution
summary. This demonstrates the missing consumer, not an implemented combat
adapter. The pending normal-discovery test is intentionally red and uncommitted.

No production behavior, fixture pin, existing test or requirement changed.
No full torture, combat reference comparison, independent candidate review,
PR, merge, or acceptance success is claimed. README remains unchanged because
no usable command or supported behavior was delivered. Preserve this checkout
and its draft when the coordinator resolves the contract and reactivates it.
