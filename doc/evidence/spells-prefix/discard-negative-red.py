"""Assert the self-discard control reaches the discard rule, not stale identity."""
import json
from pathlib import Path
actual=json.loads(Path('.agent-artifacts/spells-xmage.json').read_text())
message=actual['rejections']['discard_spell_itself']
assert message.endswith('/discard source'), f'self-discard must reject at the discard rule, observed {message}'
