more_itertools.recipes.quantify (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `quantify` takes the required argument `iterable`; the optional argument `pred` (default the `bool` built-in).

The docstring:

Return the how many times the predicate is true.

>>> quantify([True, False, True])
2
>>> quantify(range(100, 200), pred=lambda x: x<120)
20
