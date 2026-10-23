more_itertools.more.duplicates_justseen (from more-itertools 11.2.1)

Signature, in words: `duplicates_justseen` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Yields serially-duplicate elements after their first appearance.

>>> list(duplicates_justseen('mississippi'))
['s', 's', 'p']
>>> list(duplicates_justseen('AaaBbbCccAaa', str.lower))
['a', 'a', 'b', 'b', 'c', 'c', 'a', 'a']

This function is analogous to :func:`unique_justseen`.
