more_itertools.more.distinct_combinations (from more-itertools 11.2.1)

Signature, in words: `distinct_combinations` takes the required arguments `iterable`, `r`.

The docstring:

Yield the distinct combinations of *r* items taken from *iterable*.

    >>> list(distinct_combinations([0, 0, 1], 2))
    [(0, 0), (0, 1)]

Equivalent to ``set(combinations(iterable))``, except duplicates are not
generated and thrown away. For larger input sequences this is much more
efficient.
