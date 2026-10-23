more_itertools.more.split_after (from more-itertools 11.2.1)

Signature, in words: `split_after` takes the required arguments `iterable`, `pred`; the optional argument `maxsplit` (default `-1`).

The docstring:

Yield lists of items from *iterable*, where each list ends with an
item where callable *pred* returns ``True``:

    >>> list(split_after('one1two2', lambda s: s.isdigit()))
    [['o', 'n', 'e', '1'], ['t', 'w', 'o', '2']]

    >>> list(split_after(range(10), lambda n: n % 3 == 0))
    [[0], [1, 2, 3], [4, 5, 6], [7, 8, 9]]

At most *maxsplit* splits are done. If *maxsplit* is not specified or -1,
then there is no limit on the number of splits:

    >>> list(split_after(range(10), lambda n: n % 3 == 0, maxsplit=2))
    [[0], [1, 2, 3], [4, 5, 6, 7, 8, 9]]
