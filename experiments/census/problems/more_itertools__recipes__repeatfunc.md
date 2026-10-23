more_itertools.recipes.repeatfunc (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `repeatfunc` takes the required argument `function`; any number of positional arguments `args`; the optional argument `times` (default none).

The docstring:

Call *function* with *args* repeatedly, returning an iterable over the
results.

If *times* is specified, the iterable will terminate after that many
repetitions:

    >>> from operator import add
    >>> times = 4
    >>> args = 3, 5
    >>> list(repeatfunc(add, times, *args))
    [8, 8, 8, 8]

If *times* is ``None`` the iterable will not terminate:

    >>> from random import randrange
    >>> times = None
    >>> args = 1, 11
    >>> take(6, repeatfunc(randrange, times, *args))  # doctest:+SKIP
    [2, 4, 8, 1, 8, 4]
