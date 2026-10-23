more_itertools.more.partial_product (from more-itertools 11.2.1)

Signature, in words: `partial_product` takes any number of positional arguments `iterables`; the keyword-only argument `repeat` (default `1`).

The docstring:

Yields tuples containing one item from each iterator, with subsequent
tuples changing a single item at a time by advancing each iterator until it
is exhausted. This sequence guarantees every value in each iterable is
output at least once without generating all possible combinations.

This may be useful, for example, when testing an expensive function.

    >>> list(partial_product('AB', 'C', 'DEF'))
    [('A', 'C', 'D'), ('B', 'C', 'D'), ('B', 'C', 'E'), ('B', 'C', 'F')]

The *repeat* keyword argument specifies the number of repetitions
of the iterables.  For example, ``partial_product('AB', repeat=3)`` is
equivalent to ``partial_product('AB', 'AB', 'AB')``.
