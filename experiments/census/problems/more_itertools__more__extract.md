more_itertools.more.extract (from more-itertools 11.2.1)

Signature, in words: `extract` takes the required arguments `iterable`, `indices`; the keyword-only argument `monotonic` (default `False`).

The docstring:

Yield values at the specified indices.

Example:

    >>> data = 'abcdefghijklmnopqrstuvwxyz'
    >>> list(extract(data, [7, 4, 11, 11, 14]))
    ['h', 'e', 'l', 'l', 'o']

The *iterable* is consumed lazily and can be infinite.

When *monotonic* is false, the *indices* are consumed immediately
and must be finite. When *monotonic* is true, *indices* are consumed
lazily and can be infinite but must be non-decreasing.

Raises ``IndexError`` if an index lies beyond the iterable.
Raises ``ValueError`` for a negative index or for a decreasing
index when *monotonic* is true.
