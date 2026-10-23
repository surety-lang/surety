more_itertools.more.nth_combination_with_replacement (from more-itertools 11.2.1)

Signature, in words: `nth_combination_with_replacement` takes the required arguments `iterable`, `r`, `index`.

The docstring:

Equivalent to
``list(combinations_with_replacement(iterable, r))[index]``.


The subsequences with repetition of *iterable* that are of length *r* can
be ordered lexicographically. :func:`nth_combination_with_replacement`
computes the subsequence at sort position *index* directly, without
computing the previous subsequences with replacement.

    >>> nth_combination_with_replacement(range(5), 3, 5)
    (0, 1, 1)

``ValueError`` will be raised If *r* is negative.
``IndexError`` will be raised if the given *index* is invalid.
