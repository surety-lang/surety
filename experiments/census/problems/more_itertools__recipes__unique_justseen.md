more_itertools.recipes.unique_justseen (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `unique_justseen` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Yields elements in order, ignoring serial duplicates

>>> list(unique_justseen('AAAABBBCCDAABBB'))
['A', 'B', 'C', 'D', 'A', 'B']
>>> list(unique_justseen('ABBCcAD', str.lower))
['A', 'B', 'C', 'A', 'D']
