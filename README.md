# 'Avocado': A UCI chess engine in rust, using bitboards

Can be used from any UCI compatible chess gui, just compile then link to the executable. The engine is currently incomplete and fairly easy to beat, but in the future things should improve.

## Features
 - Bitboards for move generation (magic bitboards)
 - UCI compliant
 - Negamax for search
 - Fairly simple evaluation / search system (for the time being)

 ## Improvements
 - Better evaluation: distinguishing between endgame, middlegame, opening
 - Iterative deepening
 - Better move ordering and move pruning for the search function
 - Opening database
 - Refactoring move generation functions to be more readable and efficient.