more_itertools.more.intersperse (from more-itertools 11.2.1)

Signature, in words: `intersperse` takes the required arguments `e`, `iterable`; the optional argument `n` (default `1`).

The docstring:

Intersperse filler element *e* among the items in *iterable*, leaving
*n* items between each filler element.

    >>> list(intersperse('!', [1, 2, 3, 4, 5]))
    [1, '!', 2, '!', 3, '!', 4, '!', 5]

    >>> list(intersperse(None, [1, 2, 3, 4, 5], n=2))
    [1, 2, None, 3, 4, None, 5]
