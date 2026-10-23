more_itertools.more.combination_index (from more-itertools 11.2.1)

Signature, in words: `combination_index` takes the required arguments `element`, `iterable`.

The docstring:

Equivalent to ``list(combinations(iterable, r)).index(element)``

The subsequences of *iterable* that are of length *r* can be ordered
lexicographically. :func:`combination_index` computes the index of the
first *element*, without computing the previous combinations.

    >>> combination_index('adf', 'abcdefg')
    10

``ValueError`` will be raised if the given *element* isn't one of the
combinations of *iterable*.
