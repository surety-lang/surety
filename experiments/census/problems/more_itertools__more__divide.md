more_itertools.more.divide (from more-itertools 11.2.1)

Signature, in words: `divide` takes the required arguments `n`, `iterable`.

The docstring:

Divide the elements from *iterable* into *n* parts, maintaining
order.

    >>> group_1, group_2 = divide(2, [1, 2, 3, 4, 5, 6])
    >>> list(group_1)
    [1, 2, 3]
    >>> list(group_2)
    [4, 5, 6]

If the length of *iterable* is not evenly divisible by *n*, then the
length of the returned iterables will not be identical:

    >>> children = divide(3, [1, 2, 3, 4, 5, 6, 7])
    >>> [list(c) for c in children]
    [[1, 2, 3], [4, 5], [6, 7]]

If the length of the iterable is smaller than n, then the last returned
iterables will be empty:

    >>> children = divide(5, [1, 2, 3])
    >>> [list(c) for c in children]
    [[1], [2], [3], [], []]

This function will exhaust the iterable before returning.
If order is not important, see :func:`distribute`, which does not first
pull the iterable into memory.
