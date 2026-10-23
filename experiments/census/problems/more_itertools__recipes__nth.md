more_itertools.recipes.nth (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `nth` takes the required arguments `iterable`, `n`; the optional argument `default` (default none).

The docstring:

Returns the nth item or a default value.

>>> l = range(10)
>>> nth(l, 3)
3
>>> nth(l, 20, "zebra")
'zebra'
