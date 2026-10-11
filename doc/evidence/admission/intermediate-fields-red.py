import copy,json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'scripts'))
import admission_reference as reference
receipt=json.loads(Path('.agent-artifacts/admission-run-2.json').read_text())
folder=Path('.agent-artifacts')/receipt['privileged_directory']
case=json.loads(reference.FIXTURE.read_text())['cases'][0]
class RequiredIntermediate(unittest.TestCase):
 def test_actual_pre_sba_damage_is_required(self):
  run=json.loads((folder/(case['id']+'.native-1.json')).read_text())
  self.assertTrue('marked' in run,'The composed client must retain delivered pre-SBA simultaneous-damage observations')
 def test_missing_opening_checkpoint_blocks_admission(self):
  run=json.loads((folder/(case['id']+'.native-1.json')).read_text());run['opening']['points'].pop()
  with self.assertRaisesRegex(ValueError,'opening'):
   reference.check_game(run,case,'native')
unittest.main()
