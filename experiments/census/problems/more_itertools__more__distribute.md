more_itertools.more.distribute (from more-itertools 11.2.1)

Signature, in words: `distribute` takes the required arguments `n`, `iterable`.

The docstring:

Distribute the items from *iterable* among *n* smaller iterables.

    >>> group_1, group_2 = distribute(2, [1, 2, 3, 4, 5, 6])
    >>> list(group_1)
    [1, 3, 5]
    >>> list(group_2)
    [2, 4, 6]

If the length of *iterable* is not evenly divisible by *n*, then the
length of the returned iterables will not be identical:

    >>> children = distribute(3, [1, 2, 3, 4, 5, 6, 7])
    >>> [list(c) for c in children]
    [[1, 4, 7], [2, 5], [3, 6]]

If the length of *iterable* is smaller than *n*, then the last returned
iterables will be empty:

    >>> children = distribute(5, [1, 2, 3])
    >>> [list(c) for c in children]
    [[1], [2], [3], [], []]

This function uses :func:`itertools.tee` and may require significant
storage.

If you need the order items in the smaller iterables to match the
original iterable, see :func:`divide`.
