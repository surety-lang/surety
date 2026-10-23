more_itertools.recipes.flatten (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `flatten` takes the required argument `list_of_lists`.

The docstring:

Return an iterator flattening one level of nesting in a list of lists.

    >>> list(flatten([[0, 1], [2, 3]]))
    [0, 1, 2, 3]

See also :func:`collapse`, which can flatten multiple levels of nesting.
