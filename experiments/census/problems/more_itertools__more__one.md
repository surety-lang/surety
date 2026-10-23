more_itertools.more.one (from more-itertools 11.2.1)

Signature, in words: `one` takes the required argument `iterable`; the optional arguments `too_short` (default none), `too_long` (default none).

The docstring:

Return the first item from *iterable*, which is expected to contain only
that item. Raise an exception if *iterable* is empty or has more than one
item.

:func:`one` is useful for ensuring that an iterable contains only one item.
For example, it can be used to retrieve the result of a database query
that is expected to return a single row.

If *iterable* is empty, ``ValueError`` will be raised. You may specify a
different exception with the *too_short* keyword:

    >>> it = []
    >>> one(it)  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    ValueError: too few items in iterable (expected 1)'
    >>> too_short = IndexError('too few items')
    >>> one(it, too_short=too_short)  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    IndexError: too few items

Similarly, if *iterable* contains more than one item, ``ValueError`` will
be raised. You may specify a different exception with the *too_long*
keyword:

    >>> it = ['too', 'many']
    >>> one(it)  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    ValueError: Expected exactly one item in iterable, but got 'too',
    'many', and perhaps more.
    >>> too_long = RuntimeError
    >>> one(it, too_long=too_long)  # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    RuntimeError

Note that :func:`one` attempts to advance *iterable* twice to ensure there
is only one item. See :func:`spy` or :func:`peekable` to check iterable
contents less destructively.
