more_itertools.recipes.take (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `take` takes the required arguments `n`, `iterable`.

The docstring:

Return first *n* items of the *iterable* as a list.

    >>> take(3, range(10))
    [0, 1, 2]

If there are fewer than *n* items in the iterable, all of them are
returned.

    >>> take(10, range(3))
    [0, 1, 2]
