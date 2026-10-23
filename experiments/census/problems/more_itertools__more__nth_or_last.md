more_itertools.more.nth_or_last (from more-itertools 11.2.1)

Signature, in words: `nth_or_last` takes the required arguments `iterable`, `n`; the optional argument `default` (no default given: a sentinel meaning 'not supplied').

The docstring:

Return the nth or the last item of *iterable*,
or *default* if *iterable* is empty.

    >>> nth_or_last([0, 1, 2, 3], 2)
    2
    >>> nth_or_last([0, 1], 2)
    1
    >>> nth_or_last([], 0, 'some default')
    'some default'

If *default* is not provided and there are no items in the iterable,
raise ``ValueError``.
