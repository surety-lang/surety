more_itertools.more.strictly_n (from more-itertools 11.2.1)

Signature, in words: `strictly_n` takes the required arguments `iterable`, `n`; the optional arguments `too_short` (default none), `too_long` (default none).

The docstring:

Validate that *iterable* has exactly *n* items and return them if
it does. If it has fewer than *n* items, call function *too_short*
with the actual number of items. If it has more than *n* items, call function
*too_long* with the number ``n + 1``.

    >>> iterable = ['a', 'b', 'c', 'd']
    >>> n = 4
    >>> list(strictly_n(iterable, n))
    ['a', 'b', 'c', 'd']

Note that the returned iterable must be consumed in order for the check to
be made.

By default, *too_short* and *too_long* are functions that raise
``ValueError``.

    >>> list(strictly_n('ab', 3))  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    ValueError: too few items in iterable (got 2)

    >>> list(strictly_n('abc', 2))  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    ValueError: too many items in iterable (got at least 3)

You can instead supply functions that do something else.
*too_short* will be called with the number of items in *iterable*.
*too_long* will be called with `n + 1`.

    >>> def too_short(item_count):
    ...     raise RuntimeError
    >>> it = strictly_n('abcd', 6, too_short=too_short)
    >>> list(it)  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    RuntimeError

    >>> def too_long(item_count):
    ...     print('The boss is going to hear about this')
    >>> it = strictly_n('abcdef', 4, too_long=too_long)
    >>> list(it)
    The boss is going to hear about this
    ['a', 'b', 'c', 'd']
