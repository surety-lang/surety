more_itertools.recipes.first_true (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `first_true` takes the required argument `iterable`; the optional arguments `default` (default none), `pred` (default none).

The docstring:

Returns the first true value in the iterable.

If no true value is found, returns *default*

If *pred* is not None, returns the first item for which
``pred(item) == True`` .

    >>> first_true(range(10))
    1
    >>> first_true(range(10), pred=lambda x: x > 5)
    6
    >>> first_true(range(10), default='missing', pred=lambda x: x > 9)
    'missing'
