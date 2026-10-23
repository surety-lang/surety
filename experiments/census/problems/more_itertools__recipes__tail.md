more_itertools.recipes.tail (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `tail` takes the required arguments `n`, `iterable`.

The docstring:

Return an iterator over the last *n* items of *iterable*.

>>> t = tail(3, 'ABCDEFG')
>>> list(t)
['E', 'F', 'G']
