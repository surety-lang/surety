more_itertools.more.windowed (from more-itertools 11.2.1)

Signature, in words: `windowed` takes the required arguments `seq`, `n`; the optional arguments `fillvalue` (default none), `step` (default `1`).

The docstring:

Return a sliding window of width *n* over the given iterable.

    >>> all_windows = windowed([1, 2, 3, 4, 5], 3)
    >>> list(all_windows)
    [(1, 2, 3), (2, 3, 4), (3, 4, 5)]

When the window is larger than the iterable, *fillvalue* is used in place
of missing values:

    >>> list(windowed([1, 2, 3], 4))
    [(1, 2, 3, None)]

Each window will advance in increments of *step*:

    >>> list(windowed([1, 2, 3, 4, 5, 6], 3, fillvalue='!', step=2))
    [(1, 2, 3), (3, 4, 5), (5, 6, '!')]

To slide into the iterable's items, use :func:`chain` to add filler items
to the left:

    >>> iterable = [1, 2, 3, 4]
    >>> n = 3
    >>> padding = [None] * (n - 1)
    >>> list(windowed(chain(padding, iterable), 3))
    [(None, None, 1), (None, 1, 2), (1, 2, 3), (2, 3, 4)]
