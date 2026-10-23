more_itertools.more.nth_permutation (from more-itertools 11.2.1)

Signature, in words: `nth_permutation` takes the required arguments `iterable`, `r`, `index`.

The docstring:

Equivalent to ``list(permutations(iterable, r))[index]```

The subsequences of *iterable* that are of length *r* where order is
important can be ordered lexicographically. :func:`nth_permutation`
computes the subsequence at sort position *index* directly, without
computing the previous subsequences.

    >>> nth_permutation('ghijk', 2, 5)
    ('h', 'i')

``ValueError`` will be raised If *r* is negative.
``IndexError`` will be raised if the given *index* is invalid.
