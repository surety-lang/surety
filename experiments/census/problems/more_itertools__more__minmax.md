more_itertools.more.minmax (from more-itertools 11.2.1)

Signature, in words: `minmax` takes the required argument `iterable_or_value`; any number of positional arguments `others`; the keyword-only arguments `key` (default none), `default` (no default given: a sentinel meaning 'not supplied').

The docstring:

Returns both the smallest and largest items from an iterable
or from two or more arguments.

    >>> minmax([3, 1, 5])
    (1, 5)

    >>> minmax(4, 2, 6)
    (2, 6)

If a *key* function is provided, it will be used to transform the input
items for comparison.

    >>> minmax([5, 30], key=str)  # '30' sorts before '5'
    (30, 5)

If a *default* value is provided, it will be returned if there are no
input items.

    >>> minmax([], default=(0, 0))
    (0, 0)

Otherwise ``ValueError`` is raised.

This function makes a single pass over the input elements and takes care to
minimize the number of comparisons made during processing.

Note that unlike the builtin ``max`` function, which always returns the first
item with the maximum value, this function may return another item when there are
ties.

This function is based on the
`recipe <https://code.activestate.com/recipes/577916-fast-minmax-function>`__ by
Raymond Hettinger.
