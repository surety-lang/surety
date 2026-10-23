boltons.iterutils.pairwise (from boltons 26.2.0)

Signature, in words: `pairwise` takes the required argument `src`; the optional argument `end` (no default given: a sentinel meaning 'not supplied').

The docstring:

Convenience function for calling :func:`windowed` on *src*, with
*size* set to 2.

>>> pairwise(range(5))
[(0, 1), (1, 2), (2, 3), (3, 4)]
>>> pairwise([])
[]

Unless *end* is set, the number of pairs is always one less than 
the number of elements in the iterable passed in, except on an empty input, 
which will return an empty list.

With *end* set, a number of pairs equal to the length of *src* is returned,
with the last item of the last pair being equal to *end*.

>>> list(pairwise(range(3), end=None))
[(0, 1), (1, 2), (2, None)]

This way, *end* values can be useful as sentinels to signal the end of the iterable.
