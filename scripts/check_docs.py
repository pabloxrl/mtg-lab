"""Check local Markdown file links in tracked documentation without dependencies."""
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote

root = Path(__file__).resolve().parent.parent
paths = subprocess.check_output(
    ["git", "ls-files", "--cached", "--others", "--exclude-standard", "*.md"],
    cwd=root, text=True,
).splitlines()
errors = []
for name in sorted(set(paths)):
    path = root / name
    if not path.is_file():
        continue
    for target in re.findall(r"\]\(([^\s)]+)(?:\s+[^)]*)?\)", path.read_text()):
        if "://" in target or target.startswith(("#", "mailto:")):
            continue
        local = unquote(target.split("#", 1)[0])
        if local and not (path.parent / local).exists():
            errors.append(f"{name}: missing link target {target}")
if errors:
    raise SystemExit("\n".join(errors))
print("Local documentation links passed.")
