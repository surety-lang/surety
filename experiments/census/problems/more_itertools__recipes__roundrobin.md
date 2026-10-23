more_itertools.recipes.roundrobin (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `roundrobin` takes any number of positional arguments `iterables`.

The docstring:

Visit input iterables in a cycle until each is exhausted.

    >>> list(roundrobin('ABC', 'D', 'EF'))
    ['A', 'D', 'E', 'B', 'F', 'C']

This function produces the same output as :func:`interleave_longest`, but
may perform better for some inputs (in particular when the number of
iterables is small).
