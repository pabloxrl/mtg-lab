# Local collector publication

`mtg_recorder::publication::publish` composes the actual scalar collector's
sealed bundle, existing v2 file writer/manifest validator, and the separately
authorized `mtg_core::episode::replay::Registry`. It introduces no CLI, rules,
sampling, sharding or alternative replay format. This component does not certify
full collector integration or M1; #154/#117 retain their complete audits.

## Use

1. Declare `collector::Run` and `Storage`, derive each capture header with
   `run.header(ordinal)`, and use the real driver's `reset_captured` path.
2. Finish every started result, retaining the owned results. Register each
   completed result in the replay registry with a distinct opaque UUID. Registration
   verifies its actual config, accepted history and final state/RNG. Do not register
   invented completed artifacts for truncated/failed/incomplete results.
3. Create two separately managed filesystem roots. Call `publication::publish`
   with the run, borrowed results, registry, explicit per-artifact authorization
   callback, `Destination { datasets, replays }`, and storage bounds. Use a deny-all
   callback when no replay grant exists. Completed results require their registered
   references; diagnostic-only runs need no replay grant.
4. Reopen `datasets/<run UUID>/manifest.json` with `Manifest::parse`, then its
   `episodes.jsonl` with `Manifest::load_v2`, supplying the consumer's expected
   versions and byte bound. Default to `LoadMode::CompletedOnly`. Use explicit
   `Diagnostic` mode for truncated/failed/incomplete accounting.
5. Separately call `publication::read_replay(replay_root, id, result, authorize,
   max_bytes)`. The trusted owner retains the actual owned result. This API checks
   authorization before filesystem access, reconstructs its expected replay using
   the existing registry, and compares actual disk bytes. The registry itself need
   not survive publication. Knowing the ID grants no access. Reconstruct the game
   with the existing `opening::replay::played::verify`.

The policy manifest and JSONL contain only opaque replay references, never replay
payloads, seeds or accepted privileged history. Per-seat policy readers expose no
replay reference. Both-seat JSONL still contains each seat's legitimate private
observation and requires authorized offline access; it is not a live policy feed.
Dataset loading checks dataset integrity; it does not read privileged replay
storage or certify its continued availability. Authorized replay reads separately
reject missing, changed, wrong-bound or oversized artifacts.

## Publication and recovery contract

Validation and authorization happen before any directory is reserved. Actual
config/version/deck/seat/limit and episode bindings use `Run::persist`. Its returned
sealed bytes are validated, rewritten through the existing file writer, and
compared exactly to bytes reopened from disk. Every required replay is resolved
against its actual result and reread byte-for-byte before the manifest is written.
The manifest itself is encoded/validated, written, flushed, synced and reread.

Roots must already exist, resolve to directories, and be disjoint (neither nested
inside the other, including through symlinks). Each run UUID reserves a new directory
in each root with `create_dir`. Linux/Unix run directories use mode 0700 and replay
files 0600 at creation. Root ownership, access grants and filesystem management
belong to the trusted local application. There is no hostile-host security claim,
remote authentication, encryption, or protection against an authorized owner
concurrently mutating storage. Copies already released cannot be revoked.

The existing dataset writer uses a sibling `.partial`, file sync and atomic
no-replace hard link, then directory sync. Replay artifacts use the same
no-replace link mechanism, under the private root. Parents are synced after
exclusive directory creation. The manifest's final hard link is the **sole run
advertisement**, after all required artifacts have been synced and validated.
No directory name, JSONL file, or `.partial` file alone signifies a published run.

All errors propagate. Before the manifest link, failures retain the reserved
folders, partial files and any valid finished artifacts but advertise no run.
A returned manifest-directory-sync error withdraws the just-created final link
and syncs that withdrawal. If withdrawal or its sync also fails,
`Error::PublicationUncertain` explicitly reports an uncertain commit: do not call
it successful or automatically retry. A crash after the manifest link can likewise
leave a visible, fully validated run whose final directory sync was not confirmed.
No filesystem protocol can report a completed acknowledgment after an abrupt
process death. Readers must validate actual bytes; uncertainty never makes partial
artifacts valid. Hardware/filesystem durability guarantees remain those of sync
and atomic hard links on the local filesystem.

Manifest and replay `.partial` hard links are intentionally retained as recovery
aliases even on success; there is no fallible cleanup after the manifest commit.
They share the final file inode, not an independent backup; do not edit them.
They are not separately advertised artifacts. The dataset writer retains partial
bytes on its prepublication errors, according to its existing contract. Diagnostics
never load unsealed or corrupt prefixes as valid training rows. Truncated runs keep
valid sealed rows and real zero-terminal-reward boundaries; failed/incomplete
entries retain accounting without fabricated rows or completed replays.

**No automatic retry or overwrite:** once either run directory exists, that run ID
is reserved regardless of whether a manifest exists. Repeating even identical
input fails. Existing datasets/replays/manifests and recovery files are not removed
or reused. Inspect/quarantine failed attempts manually. A new attempt needs a new
run identity, fresh correctly bound captures and replay registration; relabeling
old bytes is not recovery. Pre-reservation validation failures leave roots unchanged.

`Storage` bounds the dataset/manifest and each replay file, not aggregate RSS or
all replay storage. Registry memory and replay reconstruction cost grow with
history; I/O deadlines are caller-owned. Checksums and equality establish integrity,
not authenticity. Filesystem or application mutation after publication can destroy
availability; both readers reject affected bytes without inventing completion.

[Executable acceptance, independent oracle and failure evidence](evidence/collector-publication/README.md).
