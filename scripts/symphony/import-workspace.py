"""Copy a stopped native worker checkout into the Docker volume, preserving the source."""
import argparse
from pathlib import Path
import re
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("workspace", type=Path)
args = parser.parse_args()
source = args.workspace.resolve()
if not re.fullmatch(r"GH-[0-9]+", source.name) or not (source / ".git").is_dir():
    parser.error("Expected a standalone GH-N worker clone")
root = Path(__file__).resolve().parents[2]
# No host directory is mounted into the runtime, even during migration.
script = '''import pathlib, subprocess, sys
root = pathlib.Path.home() / '.local/share/mtg-lab-symphony/workspaces'
root.mkdir(parents=True, exist_ok=True)
if (root / sys.argv[1]).exists():
    raise SystemExit('Destination already exists; refusing to overwrite work')
subprocess.run(['tar', '--no-same-owner', '-xf', '-', '-C', str(root)], check=True)
'''
with tempfile.TemporaryFile() as archive:
    subprocess.run(["tar", "--exclude=target", "--exclude=__pycache__", "--exclude=.DS_Store",
                    "-cf", "-", "-C", str(source.parent), source.name], stdout=archive, check=True)
    archive.seek(0)
    subprocess.run(["docker", "compose", "run", "--rm", "-T", "--no-deps", "symphony",
                    "python3", "-c", script, source.name], cwd=root, stdin=archive, check=True)
print(f"Imported {source.name}; native source preserved. Keep delivery paused until cutover checks pass.")
