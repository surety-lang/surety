more_itertools.recipes.running_min (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `running_min` takes the required argument `iterable`; the keyword-only argument `maxlen` (default none).

The docstring:

Smallest of values seen so far or values in a sliding window.

Set *maxlen* to a positive integer to specify the maximum size
of the sliding window.  The default of *None* is equivalent to
an unbounded window.

For example:

    >>> list(running_min([4, 3, 7, 0, 8, 1, 6, 2, 9, 5]))
    [4, 3, 3, 0, 0, 0, 0, 0, 0, 0]

    >>> list(running_min([4, 3, 7, 0, 8, 1, 6, 2, 9, 5], maxlen=3))
    [4, 3, 3, 0, 0, 0, 1, 1, 2, 2]

Supports numeric types such as int, float, Decimal, and Fraction,
but not complex numbers which are unorderable.
