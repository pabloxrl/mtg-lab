import json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'scripts'))
import admission_reference as reference
class IdentityFields(unittest.TestCase):
 def test_required_creation_and_stack_provenance(self):
  case=next(c for c in json.load(open(reference.FIXTURE))['cases'] if c['id']=='GR-040-RG-S1-heuristic')
  for field in ('birth','definition_hash'):
   actual=json.load(open('.agent-artifacts/admission/native-full-2/'+case['id']+'.native-1.json'))
   for point in actual['points']:
    for creation in point['creations']:creation.pop(field)
   with self.subTest(field=field),self.assertRaisesRegex(ValueError,'creation'):
    reference.check_game(actual,case,'native')
  actual=json.load(open('.agent-artifacts/admission/native-full-2/'+case['id']+'.native-1.json'))
  point=next(p for p in actual['points'] if any('raw_birth' in s for s in p['stack']))
  next(s for s in point['stack'] if 'raw_birth' in s).pop('raw_birth')
  with self.assertRaisesRegex(ValueError,'stack'):
   reference.check_game(actual,case,'native')
unittest.main()
