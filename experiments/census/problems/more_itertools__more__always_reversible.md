more_itertools.more.always_reversible (from more-itertools 11.2.1)

Signature, in words: `always_reversible` takes the required argument `iterable`.

The docstring:

An extension of :func:`reversed` that supports all iterables, not
just those which implement the ``Reversible`` or ``Sequence`` protocols.

    >>> print(*always_reversible(x for x in range(3)))
    2 1 0

If the iterable is already reversible, this function returns the
result of :func:`reversed()`. If the iterable is not reversible,
this function will cache the remaining items in the iterable and
yield them in reverse order, which may require significant storage.
