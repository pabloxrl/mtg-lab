"""Launch upstream Codex app-server with explicit repo-worker configuration."""
import os
from pathlib import Path
import re
import tomllib

# app-server does not accept --profile in CLI 0.157.1. Use ordinary config
# overrides and disable every host MCP connector rather than copying credentials.
config_path = Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex"))) / "config.toml"
config = tomllib.loads(config_path.read_text()) if config_path.exists() else {}
args = [os.environ.get("SYMPHONY_CODEX", "codex"), "-c", 'model="gpt-6-astra"', "-c", 'model_reasoning_effort="medium"',
        "-c", 'web_search="disabled"', "-c", 'approvals_reviewer="auto_review"']
for name in config.get("mcp_servers", {}):
    if not re.fullmatch(r"[A-Za-z0-9_-]+", name):
        raise SystemExit("Unsupported MCP name in host config; cannot safely disable it")
    args.extend(["-c", f"mcp_servers.{name}.enabled=false"])
# Each dispatched checkout owns its writable reference tree; Maven mutates it.
issue_directory = Path.cwd().name
if not re.fullmatch(r"GH-[1-9][0-9]*", issue_directory):
    raise SystemExit("Expected an issue workspace for managed delivery")
os.environ["MTG_REFERENCE_CACHE"] = str(Path.home() / ".cache" / ("xmage-" + issue_directory))
os.environ["CARGO_TARGET_DIR"] = str(Path.cwd() / "target")
args.append("app-server")
os.execvp(args[0], args)
