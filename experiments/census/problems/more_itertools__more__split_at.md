more_itertools.more.split_at (from more-itertools 11.2.1)

Signature, in words: `split_at` takes the required arguments `iterable`, `pred`; the optional arguments `maxsplit` (default `-1`), `keep_separator` (default `False`).

The docstring:

Yield lists of items from *iterable*, where each list is delimited by
an item where callable *pred* returns ``True``.

    >>> list(split_at('abcdcba', lambda x: x == 'b'))
    [['a'], ['c', 'd', 'c'], ['a']]

    >>> list(split_at(range(10), lambda n: n % 2 == 1))
    [[0], [2], [4], [6], [8], []]

At most *maxsplit* splits are done. If *maxsplit* is not specified or -1,
then there is no limit on the number of splits:

    >>> list(split_at(range(10), lambda n: n % 2 == 1, maxsplit=2))
    [[0], [2], [4, 5, 6, 7, 8, 9]]

By default, the delimiting items are not included in the output.
To include them, set *keep_separator* to ``True``.

    >>> list(split_at('abcdcba', lambda x: x == 'b', keep_separator=True))
    [['a'], ['b'], ['c', 'd', 'c'], ['b'], ['a']]
