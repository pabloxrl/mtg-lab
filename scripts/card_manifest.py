"""Offline validation of frozen Foundations metadata; no HTTP or game semantics."""
import argparse
from datetime import date
import hashlib
import json
from pathlib import Path
import re
import uuid

DEFAULT_MANIFEST = Path(__file__).resolve().parents[1] / 'data/cards/foundations_micro_v1.json'
MAX_BYTES = 256 * 1024
POOL = 'foundations_micro_v1'
# RFC 0002 §3. These are acceptance constraints, not a deck-construction API.
DECKS = {
    'red': {'Mountain':16, 'Swab Goblin':4, 'Axgard Cavalry':2, 'Crackling Cyclops':3,
            'Firebrand Archer':3, 'Dragon Fodder':4, 'Goblin Surprise':2,
            'Viashino Pyromancer':2, 'Thrill of Possibility':2, 'Shivan Dragon':2},
    'green': {'Forest':16, 'Bear Cub':4, 'Llanowar Elves':3, 'Druid of the Cowl':2,
              'Magnigoth Sentry':2, 'Tajuru Pathwarden':2, 'Thornweald Archer':3,
              'Giant Growth':3, 'Bite Down':3, 'Wildheart Invoker':2},
}
CHARACTERISTICS = {'mana_cost', 'cmc', 'type_line', 'colors', 'color_identity',
                   'keywords', 'power', 'toughness'}
PRINTING = {'id', 'set', 'collector_number', 'lang', 'layout', 'released_at'}
CARD = {'id', 'name', 'kind', 'oracle_id', 'printing', 'characteristics', 'behavior_id',
        'behavior_status', 'retrieval_date', 'source_url', 'raw_response_sha256',
        'oracle_text_sha256', 'source_projection_sha256', 'content_sha256'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fields(value, expected):
    require(isinstance(value, dict) and set(value) == expected,
            'object must contain exactly: ' + ', '.join(sorted(expected)))


def sequence(value):
    require(isinstance(value, list), 'expected an array')
    return value


def digest(value):
    """SHA-256 of UTF-8 sorted compact JSON, unescaped Unicode, no NaN."""
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                    ensure_ascii=False, allow_nan=False).encode()).hexdigest()


def sha(value):
    require(isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value), 'invalid sha256')


def identifier(value):
    require(isinstance(value, str), 'identifier must be a UUID string')
    try:
        require(str(uuid.UUID(value)) == value, 'UUID must be canonical lowercase')
    except (ValueError, AttributeError) as error:
        raise ValueError('invalid UUID') from error


def iso_date(value):
    require(isinstance(value, str) and re.fullmatch(r'\d{4}-\d{2}-\d{2}', value), 'invalid date')
    return date.fromisoformat(value)


def unique(values, label):
    require(len(set(values)) == len(values), 'duplicate ' + label)


def record_digest(record):
    sha(record['content_sha256'])
    require(digest({k:v for k,v in record.items() if k != 'content_sha256'})
            == record['content_sha256'], 'content digest mismatch')


def slug(name):
    return name.lower().replace(' ', '-')


def validate_card(card):
    fields(card, CARD)
    record_digest(card)
    names = set(DECKS['red']) | set(DECKS['green']) | {'Goblin'}
    require(isinstance(card['name'], str) and card['name'] in names, 'unsupported card name')
    token = card['name'] == 'Goblin'
    require(card['id'] == ('goblin-token' if token else slug(card['name'])), 'unsupported card id')
    require(card['kind'] == ('token' if token else 'card'), 'wrong card kind')
    identifier(card['oracle_id'])
    require(card['behavior_id'] == POOL + '/' + card['id'], 'unsupported behavior id')
    require(card['behavior_status'] == 'reserved-not-implemented', 'no implemented rules claimed')
    printing = card['printing']
    fields(printing, PRINTING)
    identifier(printing['id'])
    require(printing['set'] == ('tfdn' if token else 'fdn'), 'unsupported set')
    require(printing['lang'] == 'en', 'unsupported language')
    require(printing['layout'] == ('token' if token else 'normal'), 'unsupported layout')
    require(isinstance(printing['collector_number'], str)
            and re.fullmatch(r'[1-9][0-9]*', printing['collector_number']), 'invalid collector number')
    require(iso_date(card['retrieval_date']) >= iso_date(printing['released_at']),
            'retrieval date precedes printing')
    require(card['source_url'] == 'https://api.scryfall.com/cards/' + printing['id'], 'wrong source URL')
    for key in ['oracle_text_sha256','raw_response_sha256','source_projection_sha256']:
        sha(card[key])
    chars = card['characteristics']
    fields(chars, CHARACTERISTICS)
    require(isinstance(chars['mana_cost'], str) and
            re.fullmatch(r'(\{(?:[0-9]+|R|G)\})*', chars['mana_cost']), 'unsupported mana cost')
    require(type(chars['cmc']) in (int,float) and 0 <= chars['cmc'] <= 20, 'invalid mana value')
    for key in ['colors','color_identity','keywords']:
        sequence(chars[key])
        require(all(isinstance(v,str) for v in chars[key]), 'invalid characteristic array')
        unique(chars[key], key)
    require(set(chars['colors']) <= {'R','G'} and set(chars['color_identity']) <= {'R','G'},
            'unsupported colors')
    require(set(chars['keywords']) <= {'Flying','Reach','Haste','Vigilance','Trample','Deathtouch'},
            'unsupported keywords')
    line = chars['type_line']
    require(isinstance(line,str) and (line in ['Instant','Sorcery','Basic Land — Mountain',
            'Basic Land — Forest','Token Creature — Goblin'] or line.startswith('Creature — ')),
            'unsupported type line')
    creature = 'Creature' in line
    for key in ['power','toughness']:
        require((isinstance(chars[key],str) and re.fullmatch(r'[0-9]+',chars[key]))
                if creature else chars[key] is None, 'invalid power/toughness')
    if token:
        require(line == 'Token Creature — Goblin' and chars['power'] == chars['toughness'] == '1'
                and chars['colors'] == ['R'] and chars['mana_cost'] == '' and chars['keywords'] == [],
                'expected red 1/1 Goblin token')
    for name in ['Mountain','Forest']:
        if card['name'] == name:
            require(line == 'Basic Land — ' + name, 'wrong basic land type')


def validate_manifest(manifest):
    fields(manifest, {'schema_version','pool_id','revision','oracle_revision','rules_version',
                      'cards','decks','matchups'})
    require(type(manifest['schema_version']) is int and manifest['schema_version'] == 1,
            'unsupported schema version')
    require(manifest['pool_id'] == POOL and manifest['revision'] == '2026-09-26.1',
            'unsupported pool revision')
    require(manifest['oracle_revision'] == 'sha256-per-card-utf8-text', 'unsupported Oracle revision scheme')
    require(manifest['rules_version'] == 'cr-2026-09-25', 'unsupported rules revision')
    cards = sequence(manifest['cards'])
    require(len(cards) == 21, 'expected 20 cards plus one token')
    for card in cards:
        validate_card(card)
    for key in ['id','name','oracle_id','behavior_id']:
        unique([card[key] for card in cards], key)
    unique([card['printing']['id'] for card in cards], 'printing id')
    by_id = {card['id']:card for card in cards}
    decks = sequence(manifest['decks'])
    require(len(decks) == 2, 'expected exactly two decks')
    seen = []
    for deck in decks:
        fields(deck, {'id','total_cards','lands','spells','cards','content_sha256'})
        record_digest(deck)
        require(isinstance(deck['id'],str) and deck['id'] in DECKS, 'unsupported deck')
        seen.append(deck['id'])
        for key, expected in [('total_cards',40),('lands',16),('spells',24)]:
            require(type(deck[key]) is int and deck[key] == expected, 'wrong deck ' + key)
        entries = sequence(deck['cards'])
        actual, lands = {}, 0
        for entry in entries:
            fields(entry, {'card_id','copies'})
            cid, copies = entry['card_id'], entry['copies']
            require(isinstance(cid,str) and cid in by_id and by_id[cid]['kind'] == 'card',
                    'unsupported deck card')
            require(cid not in actual, 'duplicate deck card')
            require(type(copies) is int and copies > 0, 'copies must be positive integers')
            actual[cid] = copies
            if by_id[cid]['characteristics']['type_line'].startswith('Basic Land'):
                lands += copies
        require(actual == {slug(k):v for k,v in DECKS[deck['id']].items()}, 'wrong exact multiplicities')
        require(sum(actual.values()) == 40 and lands == 16, 'wrong computed deck totals')
    unique(seen, 'deck id')
    matchups = sequence(manifest['matchups'])
    require(len(matchups) == 8, 'expected all eight seat configurations')
    seen = []
    for matchup in matchups:
        fields(matchup, {'seats','starting_seat'})
        seats = sequence(matchup['seats'])
        require(len(seats) == 2 and all(isinstance(s,str) and s in DECKS for s in seats),
                'unsupported matchup')
        require(type(matchup['starting_seat']) is int and matchup['starting_seat'] in (0,1),
                'invalid starting seat')
        seen.append((tuple(seats),matchup['starting_seat']))
    unique(seen, 'matchup')


def parse_json(raw):
    require(0 < len(raw) <= MAX_BYTES, 'JSON is empty or exceeds 256 KiB')
    def pairs(items):
        result = {}
        for key,value in items:
            require(key not in result, 'duplicate JSON field: ' + key)
            result[key] = value
        return result
    def nonfinite(value):
        raise ValueError('nonfinite JSON number: ' + value)
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=nonfinite)


def load_manifest(path=DEFAULT_MANIFEST):
    path = Path(path)
    with path.open('rb') as stream:
        raw = stream.read(MAX_BYTES + 1)
    pin = path.with_suffix('.sha256').read_text().strip()
    sha(pin)
    require(hashlib.sha256(raw).hexdigest() == pin, 'manifest file digest mismatch')
    manifest = parse_json(raw)
    validate_manifest(manifest)
    return manifest


def verify_source(card, raw):
    """Verify stable source fields and exact text; ignore volatile API metadata.

Raw response SHA records historical acquisition bytes, which Scryfall does not
promise to serve again. No artwork/image URL is used or copied into the manifest.
"""
    validate_card(card)
    source = parse_json(raw)
    require(isinstance(source,dict), 'source must be an object')
    try:
        projection = {key:source[key] for key in PRINTING | {'oracle_id','name','oracle_text'}}
        projection['characteristics'] = {key:source.get(key) for key in CHARACTERISTICS}
        require(isinstance(source['oracle_text'],str), 'missing Oracle text')
        require(hashlib.sha256(source['oracle_text'].encode()).hexdigest() == card['oracle_text_sha256'],
                'Oracle text digest mismatch')
        expected = dict(card['printing'], oracle_id=card['oracle_id'], name=card['name'],
                        oracle_text=source['oracle_text'], characteristics=card['characteristics'])
        require(projection == expected, 'source identity/characteristics mismatch')
        require(digest(projection) == card['source_projection_sha256'], 'source projection digest mismatch')
    except KeyError as error:
        raise ValueError('missing source field: ' + str(error)) from error


def verify_cache(manifest, cache):
    validate_manifest(manifest)
    for card in manifest['cards']:
        with (Path(cache) / (card['printing']['id'] + '.json')).open('rb') as stream:
            verify_source(card,stream.read(MAX_BYTES + 1))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',type=Path,default=DEFAULT_MANIFEST)
    parser.add_argument('--cache',type=Path,help='also verify separately acquired raw source JSON offline')
    args = parser.parse_args()
    try:
        manifest = load_manifest(args.manifest)
        if args.cache is not None:
            verify_cache(manifest,args.cache)
    except (ValueError,OSError) as error:
        parser.exit(1,f'Card manifest validation failed: {error}\n')
    print(json.dumps({'validation':'offline-source-and-metadata' if args.cache else 'offline-metadata-only',
                      'pool_id':manifest['pool_id'],'cards':20,'tokens':1,'decks':2,'matchups':8}))


if __name__ == '__main__':
    main()
