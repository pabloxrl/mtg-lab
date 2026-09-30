# Authorized local episode replays

`mtg_core::episode::replay::Registry` binds a completed `EpisodeResult` from
`Driver::reset_captured` to the existing privileged played-replay format. It
records the actual reset config/master seed/ordinal and accepted semantic
history, verifies the artifact, and compares the reconstructed final state and
RNG to the owned snapshot. Only process-local store/decision namespaces are
normalized. No rules or replay format changes are introduced.

The application owner supplies an opaque artifact ID (for example a fresh UUID,
never a seed). Call `registry.register(id, &mut result)`. Successful registration
returns `Availability::Available(id)` and puts that reference in the owned
trajectory header. IDs cannot be rebound within the registry, including after
removal. Blank or conflicting IDs fail. Registration is atomic on failure and
confers no permission to read.

Call `registry.resolve(id, &result, authorize)` to borrow the replay bytes.
The mandatory authorization callback receives the stored artifact ID and episode
key; it must check the application's explicitly granted per-artifact access.
Use `|_, _| false` when no grant exists. Knowing an ID, including a valid UUID,
is not authorization. A grant for one ID must not authorize another. Resolution
also checks the complete owned-result binding and re-verifies the replay before
release. Wrong run/ordinal/config/history/result and corrupt payloads fail.
Use `opening::replay::played::verify(bytes)` to reconstruct the game.

This is a local application boundary: the owner and callback are trusted, and
there is no authentication service or protection against a hostile host process.
Do not expose the owner, its privileged inputs/history, registry, or replay bytes
to a policy. Policy observations and per-seat readers contain none of the replay
payload or seed; the trajectory header contains only the opaque reference.
Caller-declared header provenance remains the capture contract; deriving run
provenance and filesystem publication are separate deliveries.

`Unavailable(status)` covers incomplete, truncated and failed results; a
completed result without capture returns `NotCaptured`. None inserts an artifact
or fabricates completion. A nonterminal semantic history remains rejected by
the existing played recorder. A caller-provided preexisting header reference is
not proof of replay availability; always inspect registration/resolution results.

`remove(id)` discards an artifact and denies future resolution. Dropping the
registry drops all its artifacts; a new empty registry cannot resolve old IDs.
Returned slices borrow the registry, so removal/drop cannot race their use in
safe Rust. Copies previously released to authorized readers cannot be recalled.
The registry is in-memory only, owns replay buffers, retains retired IDs, and
must be managed by the local run owner; memory and replay validation cost grow
with captured history. There is no disk persistence or automatic grant store.

[Executable acceptance and independent checkpoints](evidence/episode-replay/README.md).

[Local publication](collector-publication.md) composes this registry with the
sealed dataset adapter and separate disk storage. The registry remains in-memory;
the authorized disk reader requires the retained actual owned result.
