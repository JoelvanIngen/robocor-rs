# Attempts

Documents ideas I have tried that did not result in a net performance gain.

## Unmake move

Profiling suggests memmove overhead is responsible for the majority of compute time.
Preventing copying the board, but rather mutating a single instance in-place could reduce this overhead.
An `unmake move` function would take in information necessary to restore the board to its previous state.
However, whilst this prevents excessive board cloning, the performance penalty incurred from this restoration process
was worse than the original copy overhead.
