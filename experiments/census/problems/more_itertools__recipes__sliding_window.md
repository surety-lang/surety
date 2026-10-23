more_itertools.recipes.sliding_window (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `sliding_window` takes the required arguments `iterable`, `n`.

The docstring:

Return a sliding window of width *n* over *iterable*.

    >>> list(sliding_window(range(6), 4))
    [(0, 1, 2, 3), (1, 2, 3, 4), (2, 3, 4, 5)]

If *iterable* has fewer than *n* items, then nothing is yielded:

    >>> list(sliding_window(range(3), 4))
    []

For a variant with more features, see :func:`windowed`.
