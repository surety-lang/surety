more_itertools.more.permutation_index (from more-itertools 11.2.1)

Signature, in words: `permutation_index` takes the required arguments `element`, `iterable`.

The docstring:

Equivalent to ``list(permutations(iterable, r)).index(element)```

The subsequences of *iterable* that are of length *r* where order is
important can be ordered lexicographically. :func:`permutation_index`
computes the index of the first *element* directly, without computing
the previous permutations.

    >>> permutation_index([1, 3, 2], range(5))
    19

``ValueError`` will be raised if the given *element* isn't one of the
permutations of *iterable*.
