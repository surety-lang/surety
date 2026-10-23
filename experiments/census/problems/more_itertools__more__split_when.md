more_itertools.more.split_when (from more-itertools 11.2.1)

Signature, in words: `split_when` takes the required arguments `iterable`, `pred`; the optional argument `maxsplit` (default `-1`).

The docstring:

Split *iterable* into pieces based on the output of *pred*.
*pred* should be a function that takes successive pairs of items and
returns ``True`` if the iterable should be split in between them.

For example, to find runs of increasing numbers, split the iterable when
element ``i`` is larger than element ``i + 1``:

    >>> list(split_when([1, 2, 3, 3, 2, 5, 2, 4, 2], lambda x, y: x > y))
    [[1, 2, 3, 3], [2, 5], [2, 4], [2]]

At most *maxsplit* splits are done. If *maxsplit* is not specified or -1,
then there is no limit on the number of splits:

    >>> list(split_when([1, 2, 3, 3, 2, 5, 2, 4, 2],
    ...                 lambda x, y: x > y, maxsplit=2))
    [[1, 2, 3, 3], [2, 5], [2, 4, 2]]
