"""CR603.3b: a recorded trigger selection must bind its actual placement."""
import copy
import sys
import unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
import xmage_ai_triggers
from xmage_ai_normalize import need


class TriggerSelectionBinding(unittest.TestCase):
    def witness(self):
        # Small authored parser control; full immutable games exercise both
        # actual engines in test_xmage_ai_pilot and the qualification command.
        a=dict(id='ability-a',source='card-a',controller='player',type='Triggered')
        b=dict(id='ability-b',source='card-b',controller='player',type='Triggered')
        def row(i,kind,detail,pending):
            return dict(sequence=i,kind=kind,detail=detail,
                        state=dict(pending_triggers=pending,stack=[]),
                        incarnations={'card-a':3,'card-b':3})
        return [row(0,'event/after',{'type':'SPELL_CAST'},[a,b]),
                row(1,'return/chooseTriggeredAbility',{'result':a,'arguments':[[a,b]]},[a,b]),
                row(2,'event/before',{'type':'TRIGGERED_ABILITY','target':'ability-a'},[b]),
                row(3,'event/before',{'type':'TRIGGERED_ABILITY','target':'ability-b'},[])]

    def index(self,rows):
        return xmage_ai_triggers.index(rows,{'card-a':'0/firebrand-archer/0',
            'card-b':'0/crackling-cyclops/0'},{'player':0},need)

    def test_selection_matches_order(self):
        result=self.index(self.witness())
        self.assertEqual([k['ability'] for k in result[0]['order']],['archer','cyclops'])

    def test_changed_returned_choice_cannot_be_ignored(self):
        rows=self.witness()
        rows[1]['detail']['result']=copy.deepcopy(rows[1]['detail']['arguments'][0][1])
        with self.assertRaisesRegex(ValueError,'selected trigger placement'):
            self.index(rows)
