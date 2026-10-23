more_itertools.recipes.subslices (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `subslices` takes the required argument `iterable`.

The docstring:

Return all contiguous non-empty subslices of *iterable*.

    >>> list(subslices('ABC'))
    [['A'], ['A', 'B'], ['A', 'B', 'C'], ['B'], ['B', 'C'], ['C']]

This is similar to :func:`substrings`, but emits items in a different
order.
