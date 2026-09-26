"""Provision the named AppArmor profile only when the Docker host supports it."""
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[2]
# Ubuntu 24+ restricts nested user namespaces even for unconfined processes.
# Load a named profile in the Docker host, not a global sysctl relaxation.
security = subprocess.check_output(["docker", "info", "--format", "{{json .SecurityOptions}}"], text=True)
if "apparmor" in security:
    import platform
    context = subprocess.check_output(["docker", "context", "show"], text=True).strip()
    profile = root / "docker/mtg-lab-codex.apparmor"
    if platform.system() == "Darwin" and context == "colima":
        # apparmor.service loads /etc/apparmor.d before sysinit.target; Docker
        # starts after sysinit.target. Persist the policy as well as loading it.
        subprocess.run(["colima", "ssh", "--", "sudo", "tee", "/etc/apparmor.d/mtg-lab-codex"],
                       input=profile.read_text(), text=True, stdout=subprocess.DEVNULL, check=True)
        subprocess.run(["colima", "ssh", "--", "sudo", "chmod", "644", "/etc/apparmor.d/mtg-lab-codex"], check=True)
        subprocess.run(["colima", "ssh", "--", "sudo", "systemctl", "enable", "apparmor"], check=True)
        subprocess.run(["colima", "ssh", "--", "sudo", "apparmor_parser", "-r", "/etc/apparmor.d/mtg-lab-codex"], check=True)
    elif platform.system() == "Linux":
        subprocess.run(["sudo", "install", "-m", "644", str(profile), "/etc/apparmor.d/mtg-lab-codex"], check=True)
        subprocess.run(["sudo", "systemctl", "enable", "apparmor"], check=True)
        subprocess.run(["sudo", "apparmor_parser", "-r", "/etc/apparmor.d/mtg-lab-codex"], check=True)
    else:
        raise SystemExit("Load docker/mtg-lab-codex.apparmor on the remote Docker host first; "
                         "then provision credentials from that host.")
    env = root / ".env"
    lines = env.read_text().splitlines() if env.exists() else []
    lines = [line for line in lines if not line.startswith("MTG_APPARMOR_PROFILE=")]
    env.write_text("\n".join(lines + ["MTG_APPARMOR_PROFILE=mtg-lab-codex"]) + "\n")
