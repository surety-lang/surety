more_itertools.more.first (from more-itertools 11.2.1)

Signature, in words: `first` takes the required argument `iterable`; the optional argument `default` (no default given: a sentinel meaning 'not supplied').

The docstring:

Return the first item of *iterable*, or *default* if *iterable* is
empty.

    >>> first([0, 1, 2, 3])
    0
    >>> first([], 'some default')
    'some default'

If *default* is not provided and there are no items in the iterable,
raise ``ValueError``.

:func:`first` is useful when you have a generator of expensive-to-retrieve
values and want any arbitrary one. It is marginally shorter than
``next(iter(iterable), default)``.
