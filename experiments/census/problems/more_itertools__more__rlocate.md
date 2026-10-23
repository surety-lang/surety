more_itertools.more.rlocate (from more-itertools 11.2.1)

Signature, in words: `rlocate` takes the required argument `iterable`; the optional arguments `pred` (default the `bool` built-in), `window_size` (default none).

The docstring:

Yield the index of each item in *iterable* for which *pred* returns
``True``, starting from the right and moving left.

*pred* defaults to :func:`bool`, which will select truthy items:

    >>> list(rlocate([0, 1, 1, 0, 1, 0, 0]))  # Truthy at 1, 2, and 4
    [4, 2, 1]

Set *pred* to a custom function to, e.g., find the indexes for a particular
item:

    >>> iterator = iter('abcb')
    >>> pred = lambda x: x == 'b'
    >>> list(rlocate(iterator, pred))
    [3, 1]

If *window_size* is given, then the *pred* function will be called with
that many items. This enables searching for sub-sequences:

    >>> iterable = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]
    >>> pred = lambda *args: args == (1, 2, 3)
    >>> list(rlocate(iterable, pred=pred, window_size=3))
    [9, 5, 1]

Beware, this function won't return anything for infinite iterables.
If *iterable* is reversible, ``rlocate`` will reverse it and search from
the right. Otherwise, it will search from the left and return the results
in reverse order.

See :func:`locate` to for other example applications.
