more_itertools.more.consecutive_groups (from more-itertools 11.2.1)

Signature, in words: `consecutive_groups` takes the required argument `iterable`; the optional argument `ordering` (default none).

The docstring:

Yield groups of consecutive items using :func:`itertools.groupby`.
The *ordering* function determines whether two items are adjacent by
returning their position.

By default, the ordering function is the identity function. This is
suitable for finding runs of numbers:

    >>> iterable = [1, 10, 11, 12, 20, 30, 31, 32, 33, 40]
    >>> for group in consecutive_groups(iterable):
    ...     print(list(group))
    [1]
    [10, 11, 12]
    [20]
    [30, 31, 32, 33]
    [40]

To find runs of adjacent letters, apply :func:`ord` function
to convert letters to ordinals.

    >>> iterable = 'abcdfgilmnop'
    >>> ordering = ord
    >>> for group in consecutive_groups(iterable, ordering):
    ...     print(list(group))
    ['a', 'b', 'c', 'd']
    ['f', 'g']
    ['i']
    ['l', 'm', 'n', 'o', 'p']

Each group of consecutive items is an iterator that shares its source with
*iterable*. When an output group is advanced, the previous group is
no longer available unless its elements are copied (e.g., into a ``list``).

    >>> iterable = [1, 2, 11, 12, 21, 22]
    >>> saved_groups = []
    >>> for group in consecutive_groups(iterable):
    ...     saved_groups.append(list(group))  # Copy group elements
    >>> saved_groups
    [[1, 2], [11, 12], [21, 22]]
