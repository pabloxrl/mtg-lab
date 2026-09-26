"""Validate the RFC delivery graph and lossless acceptance-requirement ledger.

This command only reads local files and Git objects; it never dispatches work.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


MILESTONES = tuple(f"M{index}" for index in range(6))


def require(condition, message):
    if not condition:
        raise ValueError(message)


def positive_int(value):
    return type(value) is int and value > 0


def heading_blocks(text):
    """Return heading-partitioned source slices, retaining original newlines."""
    lines = text.splitlines(keepends=True)
    headings = []
    fence = None
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        marker = re.match(r"(`{3,}|~{3,})", stripped)
        if marker:
            run = marker.group(1)
            if fence is None:
                fence = run
            elif run[0] == fence[0] and len(run) >= len(fence):
                fence = None
            continue
        if fence is None and re.match(r"^#{1,6} ", line):
            headings.append(index)
    require(headings and headings[0] == 0, "RFC must begin with a Markdown heading")
    return [
        (start + 1, end, lines[start].rstrip("\r\n"), "".join(lines[start:end]))
        for start, end in zip(headings, headings[1:] + [len(lines)])
    ]


def validate(program, requirements, source):
    """Raise ValueError on invalid metadata, coverage, or dependency ordering."""
    require(isinstance(source, bytes), "RFC source must be bytes")
    for document in (program, requirements):
        require(isinstance(document, dict), "program documents must be objects")
        require(type(document.get("schema_version")) is int and document["schema_version"] == 1,
                "unsupported schema_version")
        require(re.fullmatch(r"[0-9a-f]{40}", str(document.get("source_commit", ""))),
                "invalid source_commit pin")
        require(re.fullmatch(r"[0-9a-f]{64}", str(document.get("source_sha256", ""))),
                "invalid source_sha256 pin")
        require(document.get("rfc") == "doc/rfcs/0002-first-mvp.md", "unexpected RFC path")
    for field in ("source_commit", "source_sha256", "rfc"):
        require(program[field] == requirements[field], f"mismatched {field}")
    require(hashlib.sha256(source).hexdigest() == program["source_sha256"],
            "current RFC differs from pinned source_sha256")
    parent = program.get("parent_issue")
    require(positive_int(parent), "parent_issue must be a positive integer")
    authorized = program.get("authorized_milestones")
    require(isinstance(authorized, list) and all(item in MILESTONES for item in authorized),
            "unknown authorized milestone")
    require(len(set(authorized)) == len(authorized), "duplicate authorized milestone")
    tasks = program.get("tasks")
    require(isinstance(tasks, list) and tasks, "tasks must be a nonempty list")
    by_issue, keys = {}, set()
    for task in tasks:
        require(isinstance(task, dict), "task must be an object")
        issue, key = task.get("issue"), task.get("key")
        require(positive_int(issue), "task issue must be a positive integer")
        require(issue != parent, "tracking parent cannot be a task")
        require(issue not in by_issue, f"duplicate task issue {issue}")
        require(isinstance(key, str) and key.strip(), "task key must be nonempty")
        require(key not in keys, f"duplicate task key {key}")
        require(task.get("milestone") in MILESTONES, f"unknown milestone for issue {issue}")
        require(task.get("kind") in ("implementation", "gate"), f"unknown task kind for issue {issue}")
        dependencies = task.get("depends_on")
        require(isinstance(dependencies, list) and all(positive_int(dep) for dep in dependencies),
                f"invalid dependencies for issue {issue}")
        require(len(set(dependencies)) == len(dependencies), f"duplicate dependencies for issue {issue}")
        require(issue not in dependencies, f"self dependency for issue {issue}")
        refs = task.get("requirements")
        require(isinstance(refs, list) and refs and all(isinstance(ref, str) for ref in refs),
                f"missing requirements for issue {issue}")
        require(len(set(refs)) == len(refs), f"duplicate requirements for issue {issue}")
        by_issue[issue] = task
        keys.add(key)
    for issue, task in by_issue.items():
        require(all(dep in by_issue for dep in task["depends_on"]),
                f"unknown dependency for issue {issue}")

    ancestors, visiting = {}, set()

    def dependencies_of(issue):
        require(issue not in visiting, f"dependency cycle at issue {issue}")
        if issue not in ancestors:
            visiting.add(issue)
            result = set()
            for dependency in by_issue[issue]["depends_on"]:
                result.add(dependency)
                result.update(dependencies_of(dependency))
            visiting.remove(issue)
            ancestors[issue] = result
        return ancestors[issue]

    for issue in by_issue:
        dependencies_of(issue)
    previous_gate = None
    for milestone in MILESTONES:
        members = [task for task in tasks if task["milestone"] == milestone]
        gates = [task for task in members if task["kind"] == "gate"]
        require(len(gates) == 1, f"{milestone} must have exactly one gate")
        gate = gates[0]["issue"]
        implementation = {task["issue"] for task in members if task["kind"] == "implementation"}
        require(implementation, f"{milestone} has no implementation tasks")
        require(implementation <= ancestors[gate], f"{milestone} gate omits implementation dependencies")
        if previous_gate is not None:
            require(all(previous_gate in ancestors[task["issue"]] for task in members),
                    f"{milestone} task can bypass previous milestone gate")
        previous_gate = gate

    blocks = requirements.get("blocks")
    expected = heading_blocks(source.decode("utf-8"))
    require(isinstance(blocks, list) and len(blocks) == len(expected), "missing or extra RFC coverage blocks")
    reverse = {issue: set() for issue in by_issue}
    ids = set()
    for block, (start, end, heading, text) in zip(blocks, expected):
        require(isinstance(block, dict), "requirement block must be an object")
        identifier = block.get("id")
        require(isinstance(identifier, str) and re.fullmatch(r"R0002-B[0-9]{3,}", identifier),
                "invalid requirement block id")
        require(identifier not in ids, f"duplicate requirement block {identifier}")
        ids.add(identifier)
        require(type(block.get("start_line")) is int and type(block.get("end_line")) is int
                and (block["start_line"], block["end_line"]) == (start, end),
                f"incorrect source line range for {identifier}")
        require(block.get("heading") == heading, f"incorrect heading for {identifier}")
        require(block.get("text") == text, f"source text changed or missing in {identifier}")
        owners = block.get("owners")
        require(isinstance(owners, list) and owners and all(positive_int(owner) for owner in owners),
                f"unowned or invalid owners for {identifier}")
        require(len(set(owners)) == len(owners), f"duplicate owners for {identifier}")
        require(all(owner in by_issue for owner in owners), f"unknown owner for {identifier}")
        for owner in owners:
            reverse[owner].add(identifier)
    for issue, task in by_issue.items():
        require(set(task["requirements"]) == reverse[issue],
                f"requirements/owners disagree for issue {issue}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    try:
        program = json.loads((args.root / "doc/programs/rfc-0002.json").read_text())
        requirements = json.loads((args.root / "doc/programs/rfc-0002-requirements.json").read_text())
        source = (args.root / "doc/rfcs/0002-first-mvp.md").read_bytes()
        validate(program, requirements, source)
        pinned = subprocess.run(
            ["git", "show", f"{program['source_commit']}:{program['rfc']}"],
            cwd=args.root, capture_output=True, check=True,
        ).stdout
        require(pinned == source, "source_commit does not contain the pinned current RFC")
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Program validation failed: {error}\n")
    print("RFC 0002 program coverage and dependency gates passed.")


if __name__ == "__main__":
    main()
