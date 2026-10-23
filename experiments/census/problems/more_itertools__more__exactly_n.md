more_itertools.more.exactly_n (from more-itertools 11.2.1)

Signature, in words: `exactly_n` takes the required arguments `iterable`, `n`; the optional argument `predicate` (default the `bool` built-in).

The docstring:

Return ``True`` if exactly ``n`` items in the iterable are ``True``
according to the *predicate* function.

    >>> exactly_n([True, True, False], 2)
    True
    >>> exactly_n([True, True, False], 1)
    False
    >>> exactly_n([0, 1, 2, 3, 4, 5], 3, lambda x: x < 3)
    True

The iterable will be advanced until ``n + 1`` truthy items are encountered,
so avoid calling it on infinite iterables.
