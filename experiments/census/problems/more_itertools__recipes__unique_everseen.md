more_itertools.recipes.unique_everseen (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `unique_everseen` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Yield unique elements, preserving order. Remember all elements ever seen.

    >>> list(unique_everseen('AAAABBBCCDAABBB'))
    ['A', 'B', 'C', 'D']
    >>> list(unique_everseen('ABBCcAD', str.casefold))
    ['A', 'B', 'C', 'D']

Raises ``TypeError`` for unhashable items.

Some unhashable objects can be converted to hashable objects
using the *key* parameter:

* For ``list`` objects, try ``key=tuple``.
* For ``set`` objects, try ``key=frozenset``.
* For ``dict`` objects, try ``key=lambda x: frozenset(x.items())``
  or in Python 3.15 and later, set ``key=frozendict``.

Alternatively, consider the ``unique()`` itertool recipe.  It sorts
the data and then uses equality to eliminate duplicates.  Hashability
is not required.
