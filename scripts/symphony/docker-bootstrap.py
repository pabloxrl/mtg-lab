"""Provision only runtime credentials into a Docker volume, without logging secrets."""
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[2]
subprocess.run(["python3", str(root / "scripts/symphony/prepare-docker-host.py")], check=True)
auth = Path.home() / ".codex/auth.json"
if not auth.is_file():
    raise SystemExit("Missing local Codex auth cache. Authenticate Codex first.")
credential = subprocess.check_output(["gh", "auth", "token", "--hostname", "github.com"], text=True).strip()
payload = json.dumps({"codex": json.loads(auth.read_text()), "github": credential})
script = r'''import json, os, pathlib, subprocess, sys
os.umask(0o077)
data = json.load(sys.stdin)
home = pathlib.Path.home()
(home / ".codex").mkdir(exist_ok=True)
auth = home / ".codex/auth.json"
# Preserve refreshed container tokens across repeat bootstrap calls.
if not auth.exists():
    auth.write_text(json.dumps(data["codex"]) + "\n")
(home / ".codex/config.toml").write_text('cli_auth_credentials_store = "file"\n')
subprocess.run(["gh", "auth", "login", "--hostname", "github.com", "--git-protocol", "https", "--with-token"], input=data["github"], text=True, check=True, stdout=subprocess.DEVNULL)
subprocess.run(["gh", "auth", "setup-git"], check=True)
for key, value in [("user.name", "pablo ribalta"), ("user.email", "pabloxrl@gmail.com")]:
    subprocess.run(["git", "config", "--global", key, value], check=True)
print("Container authentication provisioned; credentials were not printed.")
'''
subprocess.run(["docker", "compose", "run", "--rm", "-T", "--no-deps", "symphony", "python3", "-c", script],
               cwd=root, input=payload, text=True, check=True)
