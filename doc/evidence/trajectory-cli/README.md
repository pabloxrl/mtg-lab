# Structured dataset CLI acceptance — GH-181

Partial R0002-B020/B039/B041. #120 retains EVERY original composed CLI clause;
#21 retains integration/catalog acceptance; #22 remains the M1 gate. All earlier
owners, RFC blocks, catalog cases and assertions remain intact. This child does
not complete M1 or certify whole requirement blocks.

## Executable independent expectations

`cargo test --locked -p mtg-cli --test trajectory_commands` runs in ordinary
workspace discovery, including full debug/release torture. Every command uses
closed stdin, unset displays, parsed JSON stdout/stderr and a 15-second external
watchdog which kills/reaps hangs.

Real played acceptance uses only delivered Driver/canonical capture/Run persistence
libraries, seed 181, ordinals 7/8, normal reset with the complete frozen green deck
ordered lands first. No simulate subprocess or future sibling is needed. CR103
keeps both sevens; CR117 two upkeep passes; CR103.8a skips the starting draw;
CR305 plays one Forest; CR104.3a P1 concedes. Literal expectations: five policy
decisions per completed episode, ten for two; concession is not a policy decision,
life [20,20], P0 hand six, terminal returns [1,-1]. The writer produces actual v2
JSONL and manifests; the CLI validates those bytes in a separate process. Opaque
replay references deliberately have no corresponding file or authorization.

An explicit six-decision limit produces a real truncated second episode, making
11 stored decisions total with the completed first episode. A record budget of
one causes an actual capture failure on the second Keep; an untouched reset is
Incomplete. Those runs retain the five decisions of the completed episode and
account for the failed/incomplete ordinal separately. An entirely failed run has
zero stored episodes/decisions but one failed declaration. Default manifest loads
reject all three noncompleted forms. Diagnostics retain all declarations and
report `valid_noncompleted`, never a misleading completed prefix. Reasons remain
private. The legacy handwritten v1 pair is separately labeled synthetic input;
its established one episode/one decision expectations and commands are unchanged.

## Synthetic corruption and boundary tests

Declared fault injection supplements real play. Mutations independently re-seal
JSONL with SHA256 and literal row/count rules, or update whole-file manifest hashes,
to prove semantic checking is not merely checksum detection. Cases include absent,
duplicate, mixed and unsupported versions; duplicate/missing declared episodes;
wrong ordinals, counts, checksums, provenance and status; missing footer/seal and
corrupt trailing fragments, including diagnostic manifests with matching file
hashes; private reason/payload/path redaction; explicit basename/path traversal
and partial-name rejection; symlink/missing/directory/oversized inputs; exact and
exceeded combined byte budgets; unknown/duplicate/conflicting options. V1 defaults,
explicit v1, both manifest versions and no-replace output routing remain covered.

[Compiled behavioral failures](red.log) preceded production changes: six command
contracts rejected the unsupported CLI options; one independent library-input
accounting test passed. Initial test construction needed a Debug derive and an
explicit third-choice land selection (upkeep candidate lists also include masked
land choices). Those setup errors are not behavioral-red evidence. All literal
expectations were retained; no pre-existing test changed. Further corruption
coverage exercises already-delivered strict-reader behavior.

## Delivery checks and limits

[Verification receipt](verification.json) records focused tests and quickstarts.
Final full torture results and log hashes are recorded in the issue workpad and PR. Separate clean-candidate Codex review and protected merge /
exact-main CI receipts are preserved in the issue workpad and PR. README command,
behavior and limitation changes require independent review. Completion is
conditional on those delivery gates; local acceptance alone is not integration.

The CLI is a thin caller of strict v1/v2 readers and manifest loaders. It adds no
rules, schema, collector, publisher or replay access. The libc dependency was
already locked transitively; its direct Unix use supplies portable no-follow and
nonblocking open flags for explicit input selection. No workflow/CI/review policy
changes. Historical provenance strings are cross-checked, not authenticated or
required to equal the current executable's engine build. V1 bare-file behavior
is preserved, including stored truncations; completed-only enforcement applies
to manifests and the explicit v2 command. See the [command contract](../../headless-commands.md)
for summary fields, exit codes, combined byte budget and filesystem limitations.

The first full run caught the existing v1 error-category assertions; the CLI now
retains static Invalid/Incomplete/Integrity categories without private I/O details.
All existing assertions remain unchanged.
