more_itertools.more.distinct_permutations (from more-itertools 11.2.1)

Signature, in words: `distinct_permutations` takes the required argument `iterable`; the optional argument `r` (default none).

The docstring:

Yield successive distinct permutations of the elements in *iterable*.

    >>> sorted(distinct_permutations([1, 0, 1]))
    [(0, 1, 1), (1, 0, 1), (1, 1, 0)]

Equivalent to yielding from ``set(permutations(iterable))``, except
duplicates are not generated and thrown away. For larger input sequences
this is much more efficient.

If the elements of the input iterable are sortable, the output tuples are
produced in sorted order.

Duplicate permutations arise when there are duplicated elements in the
input iterable. The number of items returned is
`n! / (x_1! * x_2! * ... * x_n!)`, where `n` is the total number of
items input, and each `x_i` is the count of a distinct item in the input
sequence. The function :func:`multinomial` computes this directly.

If *r* is given, only the *r*-length permutations are yielded.

    >>> sorted(distinct_permutations([1, 0, 1], r=2))
    [(0, 1), (1, 0), (1, 1)]
    >>> sorted(distinct_permutations(range(3), r=2))
    [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)]

*iterable* need not be sortable, but note that using equal (``x == y``)
but non-identical (``id(x) != id(y)``) elements may produce surprising
behavior. For example, ``1`` and ``True`` are equal but non-identical:

    >>> list(distinct_permutations([1, True, '3']))  # doctest: +SKIP
    [
        (1, True, '3'),
        (1, '3', True),
        ('3', 1, True)
    ]
    >>> list(distinct_permutations([1, 2, '3']))  # doctest: +SKIP
    [
        (1, 2, '3'),
        (1, '3', 2),
        (2, 1, '3'),
        (2, '3', 1),
        ('3', 1, 2),
        ('3', 2, 1)
    ]
