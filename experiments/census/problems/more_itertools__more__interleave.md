more_itertools.more.interleave (from more-itertools 11.2.1)

Signature, in words: `interleave` takes any number of positional arguments `iterables`.

The docstring:

Return a new iterable yielding from each iterable in turn,
until the shortest is exhausted.

    >>> list(interleave([1, 2, 3], [4, 5], [6, 7, 8]))
    [1, 4, 6, 2, 5, 7]

For a version that doesn't terminate after the shortest iterable is
exhausted, see :func:`interleave_longest`.
