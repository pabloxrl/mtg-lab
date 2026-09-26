# Runtime security and provenance

The multi-stage Dockerfile pins multi-architecture Rust and Maven/Temurin base
image digests plus SHA-256 digests of upstream Symphony and Codex release assets.
System packages are resolved by apt at build time. Record the resulting image
ID/digest for an exact deployment; rebuilds may incorporate system updates.

`seccomp.json` derives from the [Moby default profile](https://github.com/moby/profiles/blob/85e237f1fe229a0c61c9c7d8e743fa780d3b97ca/seccomp/default.json),
under [Apache 2.0](SECCOMP-LICENSE). The only policy additions allow `clone`,
`unshare`, `mount`, `umount2`, `pivot_root`, and `setns` for Codex's nested
bubblewrap sandbox. Docker's remaining syscall filtering is retained. The
container runs as UID 1001 with no capabilities and no-new-privileges; those
calls cannot grant host privileges. No Docker socket or host home is mounted.

Ubuntu 24+ can deny user-namespace capabilities even to unconfined applications.
The named `mtg-lab-codex.apparmor` profile permits `userns` for this deployment.
It otherwise uses AppArmor's unconfined mode; it does not claim a second
AppArmor filesystem boundary. Bootstrap installs it under `/etc/apparmor.d`, enables the boot-time AppArmor
loader (ordered before Docker by systemd), loads it on AppArmor hosts and
selects it through ignored `.env`. It never disables the host's global userns
restriction. A remote Docker host requires operator provisioning of that profile.
On Docker hosts without AppArmor the Compose default is `unconfined`.

Agents cannot install host packages. Runtime filesystem writes are confined to
the named home volume and disposable executable `/tmp`; package tooling and the
controller are on the read-only image filesystem. Authentication belongs to the
trusted agent user and is not isolated from that user's tasks. Normal Codex
sandbox and automatic approval review remain active inside the container.
