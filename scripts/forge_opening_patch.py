"""Pinned test-only controller replacement; no AI fallback during opening replay."""
import forge_capture_patch as patch


def controller(source):
    result, inventory = patch.instrument_controller(source)
    # Retain original bodies as uncalled helpers, preserving the pinned source
    # and its import/checkstyle contract. Every public choice wrapper calls only
    # the strict consumer; no replay path calls an original AI helper.
    replaced=set()
    for m in inventory:
        parameters=m['signature'].split('(',1)[1][:-1]
        args=', '.join(p.split()[-1] for p in patch.split_parameters(parameters))
        before=f'mtgRawOriginal_{m["name"]}({args});'
        after=f'forge.game.MtgOpeningReplay.choose(player, "{m["name"]}", new Object[] {{{args}}});'
        if before in replaced: continue
        if before not in result: raise ValueError('controller invocation seam changed: '+before)
        replaced.add(before)
        result=result.replace(before,after)
    return result
