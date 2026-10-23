more_itertools.recipes.powerset (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `powerset` takes the required argument `iterable`.

The docstring:

Yields all possible subsets of the iterable.

    >>> list(powerset([1, 2, 3]))
    [(), (1,), (2,), (3,), (1, 2), (1, 3), (2, 3), (1, 2, 3)]

:func:`powerset` will operate on iterables that aren't :class:`set`
instances, so repeated elements in the input will produce repeated elements
in the output.

    >>> seq = [1, 1, 0]
    >>> list(powerset(seq))
    [(), (1,), (1,), (0,), (1, 1), (1, 0), (1, 0), (1, 1, 0)]

For a variant that efficiently yields actual :class:`set` instances, see
:func:`powerset_of_sets`.
