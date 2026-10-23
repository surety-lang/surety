more_itertools.recipes.running_mean (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `running_mean` takes the required argument `iterable`; the keyword-only argument `maxlen` (default none).

The docstring:

Cumulative mean of values seen so far or values in a sliding window.

Set *maxlen* to a positive integer to specify the maximum size
of the sliding window.  The default of *None* is equivalent to
an unbounded window.

For example:

    >>> list(running_mean([40, 30, 50, 46, 39, 44]))
    [40.0, 35.0, 40.0, 41.5, 41.0, 41.5]

    >>> list(running_mean([40, 30, 50, 46, 39, 44], maxlen=3))
    [40.0, 35.0, 40.0, 42.0, 45.0, 43.0]

Supports numeric types such as int, float, complex, Decimal, and Fraction.

No extra effort is made to reduce round-off errors for float inputs.
So the results may be slightly different from `statistics.mean`.
