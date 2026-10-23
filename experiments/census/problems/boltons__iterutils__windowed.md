boltons.iterutils.windowed (from boltons 26.2.0)

Signature, in words: `windowed` takes the required arguments `src`, `size`; the optional argument `fill` (no default given: a sentinel meaning 'not supplied').

The docstring:

Returns tuples with exactly length *size*. If *fill* is unset 
and the iterable is too short to make a window of length *size*, 
no tuples are returned. See :func:`windowed_iter` for more.
