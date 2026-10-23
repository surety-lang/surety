more_itertools.more.interleave_longest (from more-itertools 11.2.1)

Signature, in words: `interleave_longest` takes any number of positional arguments `iterables`.

The docstring:

Return a new iterable yielding from each iterable in turn,
skipping any that are exhausted.

    >>> list(interleave_longest([1, 2, 3], [4, 5], [6, 7, 8]))
    [1, 4, 6, 2, 5, 7, 3, 8]

This function produces the same output as :func:`roundrobin`, but may
perform better for some inputs (in particular when the number of iterables
is large).
