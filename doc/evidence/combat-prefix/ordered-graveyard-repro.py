"""Compare preserved real engine checkpoints without erasing zone order."""
import json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[3]/'scripts'))
import played_combat_reference as combat
folder=Path(sys.argv[1])
native=json.loads((folder/'native.json').read_text())
xmage=json.loads((folder/'xmage.json').read_text())
name='shivan-multiple'
left=native['checkpoints'][name][-1]
right=xmage['checkpoints'][name][-1]
diff=combat.base.instant_reference.difference(combat.semantic(left),combat.semantic(right))
print(json.dumps({'case':name,'boundary':left['boundary'],'native_graveyard':left['graveyard'],'xmage_graveyard':right['graveyard'],'first_divergence':diff},indent=2))
assert diff is None, 'Same explicit tape must preserve ordered graveyard checkpoints; CR 404.3 owner choice is missing'
