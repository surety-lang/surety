more_itertools.more.repeat_last (from more-itertools 11.2.1)

Signature, in words: `repeat_last` takes the required argument `iterable`; the optional argument `default` (default none).

The docstring:

After the *iterable* is exhausted, keep yielding its last element.

    >>> list(islice(repeat_last(range(3)), 5))
    [0, 1, 2, 2, 2]

If the iterable is empty, yield *default* forever::

    >>> list(islice(repeat_last(range(0), 42), 5))
    [42, 42, 42, 42, 42]
