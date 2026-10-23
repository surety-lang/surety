more_itertools.more.combination_with_replacement_index (from more-itertools 11.2.1)

Signature, in words: `combination_with_replacement_index` takes the required arguments `element`, `iterable`.

The docstring:

Equivalent to
``list(combinations_with_replacement(iterable, r)).index(element)``

The subsequences with repetition of *iterable* that are of length *r* can
be ordered lexicographically. :func:`combination_with_replacement_index`
computes the index of the first *element*, without computing the previous
combinations with replacement.

    >>> combination_with_replacement_index('adf', 'abcdefg')
    20

``ValueError`` will be raised if the given *element* isn't one of the
combinations with replacement of *iterable*.
