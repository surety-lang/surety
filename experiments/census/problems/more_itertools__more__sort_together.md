more_itertools.more.sort_together (from more-itertools 11.2.1)

Signature, in words: `sort_together` takes the required argument `iterables`; the optional arguments `key_list` (default `(0,)`), `key` (default none), `reverse` (default `False`), `strict` (default `False`).

The docstring:

Return the input iterables sorted together, with *key_list* as the
priority for sorting. All iterables are trimmed to the length of the
shortest one.

This can be used like the sorting function in a spreadsheet. If each
iterable represents a column of data, the key list determines which
columns are used for sorting.

By default, all iterables are sorted using the ``0``-th iterable::

    >>> iterables = [(4, 3, 2, 1), ('a', 'b', 'c', 'd')]
    >>> sort_together(iterables)
    [(1, 2, 3, 4), ('d', 'c', 'b', 'a')]

Set a different key list to sort according to another iterable.
Specifying multiple keys dictates how ties are broken::

    >>> iterables = [(3, 1, 2), (0, 1, 0), ('c', 'b', 'a')]
    >>> sort_together(iterables, key_list=(1, 2))
    [(2, 3, 1), (0, 0, 1), ('a', 'c', 'b')]

To sort by a function of the elements of the iterable, pass a *key*
function. Its arguments are the elements of the iterables corresponding to
the key list::

    >>> names = ('a', 'b', 'c')
    >>> lengths = (1, 2, 3)
    >>> widths = (5, 2, 1)
    >>> def area(length, width):
    ...     return length * width
    >>> sort_together([names, lengths, widths], key_list=(1, 2), key=area)
    [('c', 'b', 'a'), (3, 2, 1), (1, 2, 5)]

Set *reverse* to ``True`` to sort in descending order.

    >>> sort_together([(1, 2, 3), ('c', 'b', 'a')], reverse=True)
    [(3, 2, 1), ('a', 'b', 'c')]

If the *strict* keyword argument is ``True``, then
``ValueError`` will be raised if any of the iterables have
different lengths.
