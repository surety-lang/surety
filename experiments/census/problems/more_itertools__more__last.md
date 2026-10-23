more_itertools.more.last (from more-itertools 11.2.1)

Signature, in words: `last` takes the required argument `iterable`; the optional argument `default` (no default given: a sentinel meaning 'not supplied').

The docstring:

Return the last item of *iterable*, or *default* if *iterable* is
empty.

    >>> last([0, 1, 2, 3])
    3
    >>> last([], 'some default')
    'some default'

If *default* is not provided and there are no items in the iterable,
raise ``ValueError``.
