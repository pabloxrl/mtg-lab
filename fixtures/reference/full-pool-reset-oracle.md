# Reset prefix oracle (envelope version 1)

Authored for GH-269 from CR 103 and the frozen deck JSON, independently of
native/XMage observations. GR-001/011/012/020. The expectations JSON is literal
acceptance data, never regenerated from either adapter's output.

Creation order is the frozen deck entry order, then zero-based copy index.
An occurrence is `seat/card-key/copy-index`. The two authored top-first index
permutations are respectively `[39,0,16,22,28,34,38,1,17,23,29,35,2,18,24,30,36,
3,19,25,31,37,4,20,26,32,5,21,27,33,6,7,8,9,10,11,12,13,14,15]` and 39 down to 0.
The swap variant exchanges indices 0 and 1 before the initial shuffle. Both
are copies of the same basic; seat 0 moves one between hand and library and
seat 1 changes two library positions. Card counts cannot detect these swaps.

CR 103 gives each player 20 life and seven cards, with the starting player
making the first mulligan declaration. Thus the first seven permutation entries
are hand membership (sorted for canonical comparison); the remaining 33 are the
ordered library, top first. No declaration is submitted. Initial shuffle callback
order is seat 0 then seat 1 in both adapters; it is a bridge protocol constraint,
not a claim that the CR mandates ordering these independent shuffles.

All sixteen cases cover the eight deck/starter rows and their copy-swap controls.
Output is privileged verification data, never a policy observation. Starting
seat is observed as starting seat: there is no turn-active player before turns
begin. No turn/priority field is guessed. `completion: prefix` means stopped at
the first declaration, not completed opening, mulligan, turn or game.
