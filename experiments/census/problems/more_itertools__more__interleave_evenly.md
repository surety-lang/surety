more_itertools.more.interleave_evenly (from more-itertools 11.2.1)

Signature, in words: `interleave_evenly` takes the required argument `iterables`; the optional argument `lengths` (default none).

The docstring:

Interleave multiple iterables so that their elements are evenly distributed
throughout the output sequence.

>>> iterables = [1, 2, 3, 4, 5], ['a', 'b']
>>> list(interleave_evenly(iterables))
[1, 2, 'a', 3, 4, 'b', 5]

>>> iterables = [[1, 2, 3], [4, 5], [6, 7, 8]]
>>> list(interleave_evenly(iterables))
[1, 6, 4, 2, 7, 3, 8, 5]

This function requires iterables of known length. Iterables without
``__len__()`` can be used by manually specifying lengths with *lengths*:

>>> from itertools import combinations, repeat
>>> iterables = [combinations(range(4), 2), ['a', 'b', 'c']]
>>> lengths = [4 * (4 - 1) // 2, 3]
>>> list(interleave_evenly(iterables, lengths=lengths))
[(0, 1), (0, 2), 'a', (0, 3), (1, 2), 'b', (1, 3), (2, 3), 'c']

Based on Bresenham's algorithm.
