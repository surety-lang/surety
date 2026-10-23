more_itertools.more.sliced (from more-itertools 11.2.1)

Signature, in words: `sliced` takes the required arguments `seq`, `n`; the optional argument `strict` (default `False`).

The docstring:

Yield slices of length *n* from the sequence *seq*.

>>> list(sliced((1, 2, 3, 4, 5, 6), 3))
[(1, 2, 3), (4, 5, 6)]

By the default, the last yielded slice will have fewer than *n* elements
if the length of *seq* is not divisible by *n*:

>>> list(sliced((1, 2, 3, 4, 5, 6, 7, 8), 3))
[(1, 2, 3), (4, 5, 6), (7, 8)]

If the length of *seq* is not divisible by *n* and *strict* is
``True``, then ``ValueError`` will be raised before the last
slice is yielded.

This function will only work for iterables that support slicing.
For non-sliceable iterables, see :func:`chunked`.
