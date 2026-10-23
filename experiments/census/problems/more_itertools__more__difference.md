more_itertools.more.difference (from more-itertools 11.2.1)

Signature, in words: `difference` takes the required argument `iterable`; the optional argument `func` (default the `sub` function); the keyword-only argument `initial` (default none).

The docstring:

This function is the inverse of :func:`itertools.accumulate`. By default
it will compute the first difference of *iterable* using
:func:`operator.sub`:

    >>> from itertools import accumulate
    >>> iterable = accumulate([0, 1, 2, 3, 4])  # produces 0, 1, 3, 6, 10
    >>> list(difference(iterable))
    [0, 1, 2, 3, 4]

*func* defaults to :func:`operator.sub`, but other functions can be
specified. They will be applied as follows::

    A, B, C, D, ... --> A, func(B, A), func(C, B), func(D, C), ...

For example, to do progressive division:

    >>> iterable = [1, 2, 6, 24, 120]
    >>> func = lambda x, y: x // y
    >>> list(difference(iterable, func))
    [1, 2, 3, 4, 5]

If the *initial* keyword is set, the first element will be skipped when
computing successive differences.

    >>> it = [10, 11, 13, 16]  # from accumulate([1, 2, 3], initial=10)
    >>> list(difference(it, initial=10))
    [1, 2, 3]
