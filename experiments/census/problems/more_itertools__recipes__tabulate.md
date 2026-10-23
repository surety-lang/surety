more_itertools.recipes.tabulate (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `tabulate` takes the required argument `function`; the optional argument `start` (default `0`).

The docstring:

Return an iterator over the results of ``func(start)``,
``func(start + 1)``, ``func(start + 2)``...

*func* should be a function that accepts one integer argument.

If *start* is not specified it defaults to 0. It will be incremented each
time the iterator is advanced.

    >>> square = lambda x: x ** 2
    >>> iterator = tabulate(square, -3)
    >>> take(4, iterator)
    [9, 4, 1, 0]
