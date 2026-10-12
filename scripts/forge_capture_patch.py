"""Build fail-closed observational patches against the exact pinned Forge source.

This does not select AI actions. Original method bodies are moved verbatim to
private helpers; wrappers record entry/return values and invoke each body once.
"""
import hashlib
import re

CONTROLLER = 'forge-ai/src/main/java/forge/ai/PlayerControllerAi.java'
CONTROLLER_SHA = 'a8c0efea0a55b46416fbaccbcf7aa075f5f2ff13e579fa051f5a3334bb381206'


def matching_brace(text, start):
    """Java brace matching with string/character literals and comments excluded."""
    depth = 0
    token = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|[{}]')
    for match in token.finditer(text, start):
        if match.group() == '{':
            depth += 1
        elif match.group() == '}':
            depth -= 1
            if depth == 0:
                return match.start()
    raise ValueError('unterminated Java method')


def split_parameters(parameters):
    parameters = re.sub(r'/\*[\s\S]*?\*/', '', parameters)
    result, start, depth = [], 0, 0
    for i, char in enumerate(parameters):
        depth += (char == '<') - (char == '>')
        if char == ',' and depth == 0:
            result.append(parameters[start:i].strip())
            start = i + 1
    if parameters[start:].strip():
        result.append(parameters[start:].strip())
    return result


def instrument_controller(source, expected_sha=CONTROLLER_SHA, expected_methods=115, label=""):
    if hashlib.sha256(source.encode()).hexdigest() != expected_sha:
        raise ValueError('pinned PlayerControllerAi source mismatch')
    pattern = re.compile(r'^    public ([^\n{;]+?)\s+(\w+)\(([^)]*)\)\s*\{', re.M)
    replacements, inventory = [], []
    metadata = {'getAi', 'isAI', 'pilotsNonAggroDeck', 'setupAutoProfile'}
    for match in pattern.finditer(source):
        returns, name, parameters = match.groups()
        if name in metadata:
            continue
        end = matching_brace(source, match.end()-1)
        arguments = [p.split()[-1] for p in split_parameters(parameters)]
        if not all(re.fullmatch(r'\w+', a) for a in arguments):
            raise ValueError('unsupported parameter declaration: ' + name)
        signature = name + '(' + ','.join(split_parameters(parameters)) + ')'
        method_id = hashlib.sha256(signature.encode()).hexdigest()[:16]
        helper = 'mtgRawOriginal_' + name
        return_type = returns
        if returns.startswith('<'):
            depth = 0
            for i, c in enumerate(returns):
                depth += (c == '<') - (c == '>')
                if depth == 0:
                    return_type = returns[i+1:].strip()
                    break
        args = ', '.join(arguments)
        observed_args = args + (', ability' if label == 'cost.' and args else 'ability' if label == 'cost.' else '')
        call = helper + '(' + args + ')'
        is_void = return_type == 'void'
        body = (call + ';\n            Object rawResult = null;' if is_void else
                return_type + ' rawResult = ' + call + ';')
        wrapper = match.group() + '\n' + f'''        long rawCall = forge.game.MtgRawCapture.enter(player, "{method_id}", "{label}{name}", new Object[] {{{observed_args}}});
        try {{
            {body}
            forge.game.MtgRawCapture.leave(player, rawCall, rawResult, new Object[] {{{observed_args}}});
            {'return;' if is_void else 'return rawResult;'}
        }} catch (RuntimeException | Error rawFailure) {{
            forge.game.MtgRawCapture.failure(player, rawCall, rawFailure);
            throw rawFailure;
        }}
    }}

    private {returns} {helper}({parameters}) {{''' + source[match.end():end+1]
        replacements.append((match.start(), end+1, wrapper))
        inventory.append(dict(id=method_id, name=name, signature=signature))
    if len(inventory) != expected_methods:
        raise ValueError(f'controller method inventory changed: {len(inventory)}')
    for start, end, value in reversed(replacements):
        source = source[:start] + value + source[end:]
    return source, inventory


def instrument_game(source):
    before = '    public void fireEvent(final Event event) {\n'
    if source.count(before) != 1:
        raise ValueError('Game event seam changed')
    return source.replace(before, before + '        MtgRawCapture.event(this, event);\n')


def instrument_shuffle(source):
    before = '    public final void shuffle(final SpellAbility sa) {\n'
    after = '        getZone(ZoneType.Library).setCards(getController().cheatShuffle(list));\n'
    if source.count(before) != 1 or source.count(after) != 1:
        raise ValueError('Player shuffle seam changed')
    return source.replace(before, before + '        forge.game.MtgRawCapture.shuffle(this, "before");\n').replace(
        after, after + '        forge.game.MtgRawCapture.shuffle(this, "after");\n')


def instrument_cost_decisions(source):
    return instrument_controller(source,
        '165a618bfd0ba99dbb9d5d2e8b9b30024f461317f95f2bb8f33fee414d80293b', 41, 'cost.')


def instrument_sba(source):
    signature = '    public boolean checkStateEffects(final boolean runEvents, final Set<Card> affectedCards) {'
    if source.count(signature) != 1:
        raise ValueError('SBA seam changed')
    wrapper = '''    public boolean checkStateEffects(final boolean runEvents, final Set<Card> affectedCards) {
        MtgRawCapture.boundary(game, "sba-before");
        try {
            return mtgRawOriginal_checkStateEffects(runEvents, affectedCards);
        } finally {
            MtgRawCapture.boundary(game, "sba-after");
        }
    }

    private boolean mtgRawOriginal_checkStateEffects(final boolean runEvents, final Set<Card> affectedCards) {'''
    return source.replace(signature, wrapper)
