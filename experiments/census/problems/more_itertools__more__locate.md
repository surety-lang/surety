more_itertools.more.locate (from more-itertools 11.2.1)

Signature, in words: `locate` takes the required argument `iterable`; the optional arguments `pred` (default the `bool` built-in), `window_size` (default none).

The docstring:

Yield the index of each item in *iterable* for which *pred* returns
``True``.

*pred* defaults to :func:`bool`, which will select truthy items:

    >>> list(locate([0, 1, 1, 0, 1, 0, 0]))
    [1, 2, 4]

Set *pred* to a custom function to, e.g., find the indexes for a particular
item.

    >>> list(locate(['a', 'b', 'c', 'b'], lambda x: x == 'b'))
    [1, 3]

If *window_size* is given, then the *pred* function will be called with
the values in each window. This enables searching for sub-sequences.
Note that *pred* may receive fewer than *window_size* arguments at the end of
the iterable.

    >>> iterable = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]
    >>> pred = lambda *args: args == (1, 2, 3)
    >>> list(locate(iterable, pred=pred, window_size=3))
    [1, 5, 9]

Use with :func:`seekable` to find indexes and then retrieve the associated
items:

    >>> from itertools import count
    >>> from more_itertools import seekable
    >>> source = (3 * n + 1 if (n % 2) else n // 2 for n in count())
    >>> it = seekable(source)
    >>> pred = lambda x: x > 100
    >>> indexes = locate(it, pred=pred)
    >>> i = next(indexes)
    >>> it.seek(i)
    >>> next(it)
    106
