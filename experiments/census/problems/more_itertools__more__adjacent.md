more_itertools.more.adjacent (from more-itertools 11.2.1)

Signature, in words: `adjacent` takes the required arguments `predicate`, `iterable`; the optional argument `distance` (default `1`).

The docstring:

Return an iterable over `(bool, item)` tuples where the `item` is
drawn from *iterable* and the `bool` indicates whether
that item satisfies the *predicate* or is adjacent to an item that does.

For example, to find whether items are adjacent to a ``3``::

    >>> list(adjacent(lambda x: x == 3, range(6)))
    [(False, 0), (False, 1), (True, 2), (True, 3), (True, 4), (False, 5)]

Set *distance* to change what counts as adjacent. For example, to find
whether items are two places away from a ``3``:

    >>> list(adjacent(lambda x: x == 3, range(6), distance=2))
    [(False, 0), (True, 1), (True, 2), (True, 3), (True, 4), (True, 5)]

This is useful for contextualizing the results of a search function.
For example, a code comparison tool might want to identify lines that
have changed, but also surrounding lines to give the viewer of the diff
context.

The predicate function will only be called once for each item in the
iterable.

See also :func:`groupby_transform`, which can be used with this function
to group ranges of items with the same `bool` value.
