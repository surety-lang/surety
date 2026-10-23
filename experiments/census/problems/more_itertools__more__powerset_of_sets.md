more_itertools.more.powerset_of_sets (from more-itertools 11.2.1)

Signature, in words: `powerset_of_sets` takes the required argument `iterable`; the keyword-only argument `baseset` (default the `set` built-in).

The docstring:

Yields all possible subsets of the iterable.

    >>> list(powerset_of_sets([1, 2, 3]))  # doctest: +SKIP
    [set(), {1}, {2}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}]
    >>> list(powerset_of_sets([1, 1, 0]))  # doctest: +SKIP
    [set(), {1}, {0}, {0, 1}]

:func:`powerset_of_sets` takes care to minimize the number
of hash operations performed.

The *baseset* parameter determines what kind of sets are
constructed, either *set* or *frozenset*.
