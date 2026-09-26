"""Reject evidence loss and graphs that could authorize premature delivery."""
import copy
import hashlib
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from check_program import heading_blocks, validate  # noqa: E402


def fixture():
    source = b"# MVP\nEvery requirement survives.\n\n## Acceptance\nNo skipped gates.\n"
    pins = dict(schema_version=1, source_commit="a" * 40,
                source_sha256=hashlib.sha256(source).hexdigest(),
                rfc="doc/rfcs/0002-first-mvp.md")
    tasks = []
    for number in range(6):
        implementation, gate = number * 2 + 2, number * 2 + 3
        for issue, kind, dependencies in (
            (implementation, "implementation", [] if number == 0 else [implementation - 1]),
            (gate, "gate", [implementation]),
        ):
            tasks.append(dict(key=f"task-{issue}", issue=issue, kind=kind,
                              milestone=f"M{number}", depends_on=dependencies,
                              requirements=["R0002-B001", "R0002-B002"]))
    program = dict(pins, parent_issue=1, authorized_milestones=["M0"], tasks=tasks)
    requirements = dict(pins, blocks=[
        dict(id=f"R0002-B{index:03}", heading=heading, start_line=start,
             end_line=end, text=text, owners=[task["issue"] for task in tasks])
        for index, (start, end, heading, text) in enumerate(heading_blocks(source.decode()), 1)
    ])
    return program, requirements, source


class ProgramTests(unittest.TestCase):
    def setUp(self):
        self.program, self.requirements, self.source = fixture()

    def reject(self, message):
        with self.assertRaisesRegex(ValueError, message):
            validate(self.program, self.requirements, self.source)

    def test_valid_transitive_gates_and_no_mutation(self):
        before = copy.deepcopy((self.program, self.requirements))
        validate(self.program, self.requirements, self.source)
        self.assertEqual(before, (self.program, self.requirements))

    def test_deleted_block_is_missing_coverage(self):
        self.requirements["blocks"].pop()
        self.reject("coverage blocks")

    def test_unowned_block_is_rejected(self):
        self.requirements["blocks"][0]["owners"] = []
        self.reject("unowned")

    def test_cycle_is_rejected(self):
        self.program["tasks"][0]["depends_on"] = [3]
        self.reject("cycle")

    def test_unknown_and_self_dependencies(self):
        for dependency, message in ((999, "unknown dependency"), (2, "self dependency")):
            with self.subTest(dependency=dependency):
                self.program["tasks"][0]["depends_on"] = [dependency]
                self.reject(message)

    def test_gate_must_cover_all_implementation(self):
        self.program["tasks"][1]["depends_on"] = []
        self.reject("gate omits")

    def test_later_work_cannot_bypass_previous_gate(self):
        self.program["tasks"][2]["depends_on"] = [2]
        self.reject("bypass previous")

    def test_parent_cannot_be_dispatched(self):
        self.program["parent_issue"] = 2
        self.reject("tracking parent")

    def test_requirements_and_reverse_owners_must_match(self):
        self.program["tasks"][0]["requirements"].pop()
        self.reject("requirements/owners disagree")

    def test_source_text_cannot_be_rewritten(self):
        self.requirements["blocks"][0]["text"] += "Changed expectation.\n"
        self.reject("source text")

    def test_line_ranges_and_headings_are_validated(self):
        self.requirements["blocks"][0]["end_line"] += 1
        self.reject("line range")
        self.requirements["blocks"][0]["end_line"] -= 1
        self.requirements["blocks"][0]["heading"] = "# A different contract"
        self.reject("incorrect heading")

    def test_current_rfc_and_pins_cannot_drift(self):
        self.source += b"An untracked new acceptance requirement.\n"
        self.reject("current RFC differs")

    def test_invalid_or_mismatched_pins(self):
        self.program["source_commit"] = "main"
        self.reject("invalid source_commit")
        self.program["source_commit"] = "b" * 40
        self.reject("mismatched source_commit")

    def test_duplicates_rejected(self):
        cases = ("issue", "key", "requirements", "depends_on")
        for field in cases:
            with self.subTest(field=field):
                self.program, self.requirements, self.source = fixture()
                if field in ("issue", "key"):
                    self.program["tasks"][1][field] = self.program["tasks"][0][field]
                else:
                    self.program["tasks"][1][field] *= 2
                self.reject("duplicate")

    def test_unknown_milestone_is_not_authorized(self):
        self.program["authorized_milestones"] = ["M6"]
        self.reject("unknown authorized milestone")
        self.program["authorized_milestones"] = ["M0"]
        self.program["tasks"][0]["milestone"] = "M6"
        self.reject("unknown milestone")

    def test_gate_cannot_be_missing_or_duplicated(self):
        self.program["tasks"][1]["kind"] = "implementation"
        self.reject("exactly one gate")

    def test_markdown_examples_are_not_requirement_headings(self):
        blocks = heading_blocks("# Actual\n```text\n# Example\n```\n## Next\nEnd.\n")
        self.assertEqual([(block[0], block[1]) for block in blocks], [(1, 4), (5, 6)])


if __name__ == "__main__":
    unittest.main()
