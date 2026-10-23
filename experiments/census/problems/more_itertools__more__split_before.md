more_itertools.more.split_before (from more-itertools 11.2.1)

Signature, in words: `split_before` takes the required arguments `iterable`, `pred`; the optional argument `maxsplit` (default `-1`).

The docstring:

Yield lists of items from *iterable*, where each list ends just before
an item for which callable *pred* returns ``True``:

    >>> list(split_before('OneTwo', lambda s: s.isupper()))
    [['O', 'n', 'e'], ['T', 'w', 'o']]

    >>> list(split_before(range(10), lambda n: n % 3 == 0))
    [[0, 1, 2], [3, 4, 5], [6, 7, 8], [9]]

At most *maxsplit* splits are done. If *maxsplit* is not specified or -1,
then there is no limit on the number of splits:

    >>> list(split_before(range(10), lambda n: n % 3 == 0, maxsplit=2))
    [[0, 1, 2], [3, 4, 5], [6, 7, 8, 9]]
