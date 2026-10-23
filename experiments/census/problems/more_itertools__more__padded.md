more_itertools.more.padded (from more-itertools 11.2.1)

Signature, in words: `padded` takes the required argument `iterable`; the optional arguments `fillvalue` (default none), `n` (default none), `next_multiple` (default `False`).

The docstring:

Yield the elements from *iterable*, followed by *fillvalue*, such that
at least *n* items are emitted.

    >>> list(padded([1, 2, 3], '?', 5))
    [1, 2, 3, '?', '?']

If *next_multiple* is ``True``, *fillvalue* will be emitted until the
number of items emitted is a multiple of *n*:

    >>> list(padded([1, 2, 3, 4], n=3, next_multiple=True))
    [1, 2, 3, 4, None, None]

If *n* is ``None``, *fillvalue* will be emitted indefinitely.

To create an *iterable* of exactly size *n*, you can truncate with
:func:`islice`.

    >>> list(islice(padded([1, 2, 3], '?'), 5))
    [1, 2, 3, '?', '?']
    >>> list(islice(padded([1, 2, 3, 4, 5, 6, 7, 8], '?'), 5))
    [1, 2, 3, 4, 5]
