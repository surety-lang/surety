more_itertools.recipes.nth_combination (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `nth_combination` takes the required arguments `iterable`, `r`, `index`.

The docstring:

Equivalent to ``list(combinations(iterable, r))[index]``.

The subsequences of *iterable* that are of length *r* can be ordered
lexicographically. :func:`nth_combination` computes the subsequence at
sort position *index* directly, without computing the previous
subsequences.

    >>> nth_combination(range(5), 3, 5)
    (0, 3, 4)

``ValueError`` will be raised If *r* is negative.
``IndexError`` will be raised if the given *index* is invalid.
