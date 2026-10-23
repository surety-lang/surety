more_itertools.recipes.unique (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `unique` takes the required argument `iterable`; the optional arguments `key` (default none), `reverse` (default `False`).

The docstring:

Yields unique elements in sorted order.

>>> list(unique([[1, 2], [3, 4], [1, 2]]))
[[1, 2], [3, 4]]

*key* and *reverse* are passed to :func:`sorted`.

>>> list(unique('ABBcCAD', str.casefold))
['A', 'B', 'c', 'D']
>>> list(unique('ABBcCAD', str.casefold, reverse=True))
['D', 'c', 'B', 'A']

The elements in *iterable* need not be hashable, but they must be
comparable for sorting to work.
