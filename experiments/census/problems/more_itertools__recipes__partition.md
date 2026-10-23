more_itertools.recipes.partition (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `partition` takes the required arguments `pred`, `iterable`.

The docstring:

Returns a 2-tuple of iterables derived from the input iterable.
The first yields the items that have ``pred(item) == False``.
The second yields the items that have ``pred(item) == True``.

    >>> is_odd = lambda x: x % 2 != 0
    >>> iterable = range(10)
    >>> even_items, odd_items = partition(is_odd, iterable)
    >>> list(even_items), list(odd_items)
    ([0, 2, 4, 6, 8], [1, 3, 5, 7, 9])

If *pred* is None, :func:`bool` is used.

    >>> iterable = [0, 1, False, True, '', ' ']
    >>> false_items, true_items = partition(None, iterable)
    >>> list(false_items), list(true_items)
    ([0, False, ''], [1, True, ' '])
