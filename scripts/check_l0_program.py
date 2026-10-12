"""Validate capture-first L0 registration and plan one read-only bounded handoff."""
import argparse
import json
from pathlib import Path
import subprocess

from check_engine_program import select_successor, validate as validate_registry
from check_program import require

PATH = 'doc/programs/reference-ai-l0.json'
CONTRACT = 'doc/programs/reference-ai-l0.md'
PROFILE = dict(namespace='reference-ai-l0', coordinator=318, milestone=8, stage='L0',
               prerequisites=[26, 277, 318], source=CONTRACT, prefix='L0',
               kinds=('implementation', 'operations', 'gate'))


def validate(program, source, legacy, m3):
    tasks = validate_registry(program, source, legacy, profile=PROFILE)
    m3_ids = {t['issue'] for t in m3['tasks']} | {m3['coordinator_issue']}
    require(not (set(tasks) | set(program['prerequisites'])) & m3_ids,
            'M3 cannot enter L0 or become a prerequisite')
    require(program['tasks'][0]['key'] == 'xmage_capture'
            and program['tasks'][0]['kind'] == 'implementation'
            and program['tasks'][0]['depends_on'] == [],
            'first delivery must be concrete capture without architecture prerequisite')
    return tasks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ref', help='fresh main ref; candidate only for coordinator registration')
    parser.add_argument('--snapshot', type=Path, help='verified live inputs; never mutates GitHub')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]

    def read(path):
        if args.ref:
            return subprocess.check_output(['git', 'show', f'{args.ref}:{path}'], cwd=root)
        return (root / path).read_bytes()

    try:
        program = json.loads(read(PATH))
        validate(program, read(CONTRACT), json.loads(read('doc/programs/rfc-0002.json')),
                 json.loads(read('doc/programs/engine-validation-m3.json')))
        if args.snapshot:
            print(json.dumps(select_successor(program, json.loads(args.snapshot.read_text()))))
        else:
            print('L0 registry, capture-first ownership and complete integrated gate passed.')
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'L0 registry validation failed: {error}\n')


if __name__ == '__main__':
    main()
