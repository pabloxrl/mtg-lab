# Reproduction against actual native observations, before checker hardening.
import copy,json,sys,unittest
from pathlib import Path
sys.path.insert(0,'scripts')
import played_combat_reference as combat
class EarlyLifeRed(unittest.TestCase):
 def test_mutated_life_before_combat_is_rejected(self):
  r=json.loads(Path(sys.argv[1]).read_text())
  r['runs']['shivan-unblocked']['points'][0]['life'][0]=19
  r['checkpoints']['shivan-unblocked'][0]['life'][0]=19
  with self.assertRaisesRegex(ValueError,'life'):combat.check_run(r,json.loads(combat.FIXTURE.read_text()),'native')
unittest.main(argv=[sys.argv[0]])
