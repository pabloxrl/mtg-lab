import copy,json,sys,unittest
from pathlib import Path
sys.path.insert(0,'scripts');import admission_reference as a
root=Path('.agent-artifacts');receipt=json.loads((root/'admission-run-1.json').read_text());folder=root/receipt['privileged_directory'];doc=json.loads(a.FIXTURE.read_text());case=next(c for c in doc['cases'] if c['id']=='GR-040-RG-S1-heuristic')
class RequiredAuxiliaryFields(unittest.TestCase):
 def test_native_cleanup(self):
  index=json.loads((folder/'native-index.json').read_text());run=json.loads((folder/index[case['id']][0]).read_text())
  next(p for p in run['cleanup_checkpoints'] if p['creations'])['creations'][0].pop('birth')
  with self.assertRaises(ValueError):a.check_game(run,case,'native')
 def test_xmage_predeparture(self):
  index=json.loads((folder/'xmage-index.json').read_text());run=json.loads((folder/index[case['id']][0]).read_text())
  run['before_departure']['creations'][0].pop('raw_uuid')
  with self.assertRaises(ValueError):a.check_game(run,a.reference_case(case)[0],'xmage')
unittest.main()
