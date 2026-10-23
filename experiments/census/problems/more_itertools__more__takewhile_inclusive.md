more_itertools.more.takewhile_inclusive (from more-itertools 11.2.1)

Signature, in words: `takewhile_inclusive` takes the required arguments `predicate`, `iterable`.

The docstring:

A variant of :func:`takewhile` that yields one additional element.

    >>> list(takewhile_inclusive(lambda x: x < 5, [1, 4, 6, 4, 1]))
    [1, 4, 6]

:func:`takewhile` would return ``[1, 4]``.
