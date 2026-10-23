more_itertools.more.replace (from more-itertools 11.2.1)

Signature, in words: `replace` takes the required arguments `iterable`, `pred`, `substitutes`; the optional arguments `count` (default none), `window_size` (default `1`).

The docstring:

Yield the items from *iterable*, replacing the items for which *pred*
returns ``True`` with the items from the iterable *substitutes*.

    >>> iterable = [1, 1, 0, 1, 1, 0, 1, 1]
    >>> pred = lambda x: x == 0
    >>> substitutes = (2, 3)
    >>> list(replace(iterable, pred, substitutes))
    [1, 1, 2, 3, 1, 1, 2, 3, 1, 1]

If *count* is given, the number of replacements will be limited:

    >>> iterable = [1, 1, 0, 1, 1, 0, 1, 1, 0]
    >>> pred = lambda x: x == 0
    >>> substitutes = [None]
    >>> list(replace(iterable, pred, substitutes, count=2))
    [1, 1, None, 1, 1, None, 1, 1, 0]

Use *window_size* to control the number of items passed as arguments to
*pred*. This allows for locating and replacing subsequences.

    >>> iterable = [0, 1, 2, 5, 0, 1, 2, 5]
    >>> window_size = 3
    >>> pred = lambda *args: args == (0, 1, 2)  # 3 items passed to pred
    >>> substitutes = [3, 4] # Splice in these items
    >>> list(replace(iterable, pred, substitutes, window_size=window_size))
    [3, 4, 5, 3, 4, 5]

*pred* may receive fewer than *window_size* arguments at the end of
the iterable and should be able to handle this.
