more_itertools.more.only (from more-itertools 11.2.1)

Signature, in words: `only` takes the required argument `iterable`; the optional arguments `default` (default none), `too_long` (default none).

The docstring:

If *iterable* has only one item, return it.
If it has zero items, return *default*.
If it has more than one item, raise the exception given by *too_long*,
which is ``ValueError`` by default.

>>> only([], default='missing')
'missing'
>>> only([1])
1
>>> only([1, 2])  # doctest: +IGNORE_EXCEPTION_DETAIL
Traceback (most recent call last):
...
ValueError: Expected exactly one item in iterable, but got 1, 2,
 and perhaps more.'
>>> only([1, 2], too_long=TypeError)  # doctest: +IGNORE_EXCEPTION_DETAIL
Traceback (most recent call last):
...
TypeError

Note that :func:`only` attempts to advance *iterable* twice to ensure there
is only one item.  See :func:`spy` or :func:`peekable` to check
iterable contents less destructively.
