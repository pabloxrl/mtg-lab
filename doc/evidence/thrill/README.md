# Thrill discard and ordered draw acceptance

GH-202 enables pinned 1R Thrill of Possibility: discard another hand card as
an additional cost, then draw two on resolution. Independent expectations use
pinned Oracle and CR 601.2h, 608, 121.1–2 and 704.5b. [Compiled red](red.log)
precedes implementation. Delivery remains conditional on full torture,
separate candidate-bound review, protected merge and exact-main CI; the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/202#issuecomment-5968348622)
records these receipts. Partial R0002-B010/B011/B029 only, no M2 verdict.

Normal discovery includes core casting `thrill_*`, recorder
`thrill_normal_reset_discard_draw_capture_replay`, policy
`thrill_policy_cast_discard_payment_and_previous_version_rejection`, and Python
`test_thrill_reference.py`.

- Synthetic ordered Forest/Cub/Growth deck draws Forest then Cub and leaves
  Growth on top. Mountain reaches graveyard with paid mana before responses.
  Zero/one-card libraries lose at SBA only after the spell finishes; no refund.
- Thrill alone, wrong color, insufficient mana, missing/self/duplicate discard,
  wrong seat, stale input and unpaid finish reject without mutation. Snapshots
  preserve pending discard/payment and bounded resolution work; cancellation
  exposes no provisional discard and spends nothing.
- Real ordered red-deck reset plays two Mountains, casts Thrill on turn three,
  discards Mountain and draws known Swab Goblin then Dragon Fodder. Existing
  Driver/Run proves quantum 1/unbounded and capture on/off equality, semantic
  replay, typed JSONL round trips and once-only concession rewards. Rejected
  commands preserve state, history and capture. No synthetic hook substitutes
  for this played evidence.

The private cast-state boundary owns the provisional discard. `begin_cast`
starts the existing payment continuation; `choose_cast_discard` selects exactly
one other hand handle. Policy uses existing `Discard` semantic/typed commands
with the new `cast_discard` decision kind. `CancelPayment` cancels it; only
`FinishPayment` commits the logical action. Recording distinguishes this
continuation from a completed cleanup discard. Native policy IDs now end in
`thrill-v1`; older IDs reject. Source fingerprints reject old snapshots/replays.

## Reference checks

```sh
./scripts/torture.sh
python3 scripts/thrill_reference.py --cache /home/agent/.cache/xmage --output /tmp/thrill-reference
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/thrill-instants
```

Six shared native/XMage cases cover ordered draws, one/zero-card libraries,
Thrill alone, wrong color and insufficient mana. Both compare literal ordered
hand/library/graveyard/stack ledgers, mana and loss before/after actual casting
and resolution, twice, with comparator mutation controls. Setup is synthetic;
actual pinned spell/additional cost/draw/SBA rules execute in XMage. Terminal
zone ledgers are captured at completed resolution before losing-player cleanup,
with the loss flag captured after real SBA. Reference legal-cast rejections use
XMage's playable abilities; native tests separately submit invalid commands.
No Forge or full-game reference claim.

[Catalog allocation](catalog.json) preserves original owners and expectations.
Cast-trigger compositions wait for #205/#209 and cleanup exceptions for #207;
#23/#24/#26 keep their full acceptance. This adds no trigger implementation.

The old unsupported-card enumeration now includes Thrill among supported
identities with a literal pinned 1R expectation; all remaining unsupported
identities retain rejection coverage. Independent review must assess this
requirement correction, equivalent/stronger coverage, and README accuracy.
No tests are removed or skipped; no workflow/CI/auth/budget changes.

Executed reference receipts: [six Thrill cases](reference.json) and
[eleven retained Growth/Bite cases](instant-reference.json), each twice per
engine with negative controls. The Thrill bridge reverses XMage's add-on-top
setup insertion to preserve neutral top-to-bottom order and asserts that
translation before casting. XMage's playable enumeration may offer Thrill alone;
the actual cast rejects its unpaid additional cost and leaves the observed ledger
unchanged. That actual result, not candidate enumeration, is compared.

Final local validation after current-main integration: [full torture](torture.log)
PASS, 168 Python tests and 1152 Rust debug/release executions, zero failed or
ignored. The [updated native quickstart](quickstart.json) completes both games
with zero failed, incomplete or truncated episodes. Independent review and
protected delivery receipts remain recorded in the linked issue workpad/PR.
