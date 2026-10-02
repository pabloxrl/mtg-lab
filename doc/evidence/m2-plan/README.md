# M2 planning amendment evidence

This is coordinator planning acceptance for #80, not new engine behavior or an M2
milestone verdict. The [27 contracts](../../programs/m2-atomic-delivery.md) and
[175-case crosswalk](../../programs/m2-test-crosswalk.md) retain the complete component
acceptance and gates while sizing future work. The source baseline is verified main
`503ab665372122fd7a8c08b7ba8e6ba77ce525c2`.

## Reproduce planning checks

From the repository root at the amendment candidate:

```sh
python3 scripts/check_program.py
python3 scripts/test_plan.py
python3 scripts/check_docs.py
python3 doc/evidence/m2-plan/check_preservation.py
./scripts/torture.sh
```

[Preservation results](preservation.json) assert all original tasks/owners remain,
only M2 component dependency lists change, all new tasks reach their gate and retain
M1, all RFC block content/source pins are unchanged, the entire catalog is byte-identical,
and every M2 case appears exactly once in the new execution crosswalk. Graph/ledger
validation independently checks schema, known owners, acyclicity and all stage gates.
The catalog validator retains all 320 designs (80 capabilities), without claiming
new execution. These scripts are an audit reproducer, not new engine regressions;
no rules/behavior or production/test code is changed by this amendment.

[Native link receipt](native-links.json) records re-fetched child blocked-by sets and
sub-issue parents. Every child remains unready during planning. The receipt explicitly
lists component dependency replacements to apply only after protected merge and
successful CI on its exact main commit. Final GitHub reconciliation, review JSON,
PR/merge/CI and bounded successor receipts belong in the
[single issue workpad](https://github.com/pabloxrl/mtg-lab/issues/80#issuecomment-5947085966).

README impact was checked: no usable command, setup, implemented architecture,
supported capability or verified milestone changes. The six-card scalar limitation,
M1 gate verdict and M2 planned status remain accurate. An unchanged root README is
intentional; the atomic delivery guide links the new plan. Independent review must
check this conclusion. No quickstart command is changed.
