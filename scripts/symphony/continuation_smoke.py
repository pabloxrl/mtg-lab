"""Bounded credential-free checks against the image's pinned Symphony package.

Unpack into a disposable home, then execute only controller test entry points.
No controller supervisor, network client, Codex process or live workspace runs.
"""
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def main():
    with tempfile.TemporaryDirectory(prefix="symphony-continuation-") as directory:
        home = Path(directory)
        # Deliberately do not inherit credentials, Erlang flags or user config.
        env = {"HOME": directory, "PATH": "/usr/local/bin:/usr/bin:/bin",
               "LANG": "C.UTF-8", "GITHUB_TOKEN": "smoke-no-credential"}
        unpacked = subprocess.run(["/usr/local/bin/symphony", "--help"], env=env, cwd=home,
                                  capture_output=True, text=True, timeout=60)
        # v0.0.3 deliberately returns 1 for its usage-only invocation.
        if unpacked.returncode != 1 or "Usage: symphony" not in unpacked.stderr:
            raise RuntimeError("Pinned Symphony usage/unpack check failed")
        releases = list(home.glob(".local/share/.burrito/symphony_erts-*_0.0.3"))
        if len(releases) != 1:
            raise RuntimeError("Expected the image's pinned Symphony v0.0.3 release")
        release = releases[0]
        executables = list(release.glob("erts-*/bin/erlexec"))
        if len(executables) != 1:
            raise RuntimeError("Pinned Symphony runtime is incomplete")
        executable = executables[0]
        env.update(ROOTDIR=str(release), BINDIR=str(executable.parent),
                   EMU="beam", PROGNAME="erl")
        command = [str(executable), "-boot", str(release / "releases/0.0.3/start_clean"),
                   "-boot_var", "RELEASE_LIB", str(release / "lib"), "-pa"]
        command += [str(path) for path in sorted(release.glob("lib/*/ebin"))]
        command += ["-noshell", "-s", "elixir", "start_cli", "-extra",
                    str(ROOT / "tests/fixtures/symphony/continuation.exs"),
                    str(ROOT / "WORKFLOW.md")]
        result = subprocess.run(command, env=env, cwd=home, check=True,
                                capture_output=True, text=True, timeout=30)
        if "credential-free-controller-continuation-ok" not in result.stdout:
            raise RuntimeError("Controller smoke did not complete")
        print(result.stdout.strip())


if __name__ == "__main__":
    main()
