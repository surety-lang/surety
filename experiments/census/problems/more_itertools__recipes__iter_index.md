more_itertools.recipes.iter_index (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `iter_index` takes the required arguments `iterable`, `value`; the optional arguments `start` (default `0`), `stop` (default none).

The docstring:

Yield the index of each place in *iterable* that *value* occurs,
beginning with index *start* and ending before index *stop*.

>>> list(iter_index('AABCADEAF', 'A'))
[0, 1, 4, 7]
>>> list(iter_index('AABCADEAF', 'A', 1))  # start index is inclusive
[1, 4, 7]
>>> list(iter_index('AABCADEAF', 'A', 1, 7))  # stop index is not inclusive
[1, 4]

The behavior for non-scalar *value* arguments matches the built-in Python types.

>>> list(iter_index('ABCDABCD', 'AB'))
[0, 4]
>>> list(iter_index([0, 1, 2, 3, 0, 1, 2, 3], [0, 1]))
[]
>>> list(iter_index([[0, 1], [2, 3], [0, 1], [2, 3]], [0, 1]))
[0, 2]

For ``range`` objects (and other objects whose ``index`` method's behavior doesn't
match that of ``list``), wrap *iterable* with ``iter``:

>>> list(iter_index(iter(range(5)), 2))
[2]

See :func:`locate` for a more general means of finding the indexes
associated with particular values.
