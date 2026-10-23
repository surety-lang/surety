more_itertools.recipes.running_median (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `running_median` takes the required argument `iterable`; the keyword-only argument `maxlen` (default none).

The docstring:

Cumulative median of values seen so far or values in a sliding window.

Set *maxlen* to a positive integer to specify the maximum size
of the sliding window.  The default of *None* is equivalent to
an unbounded window.

For example:

    >>> list(running_median([5.0, 9.0, 4.0, 12.0, 8.0, 9.0]))
    [5.0, 7.0, 5.0, 7.0, 8.0, 8.5]
    >>> list(running_median([5.0, 9.0, 4.0, 12.0, 8.0, 9.0], maxlen=3))
    [5.0, 7.0, 5.0, 9.0, 8.0, 9.0]

Supports numeric types such as int, float, Decimal, and Fraction,
but not complex numbers which are unorderable.

On version Python 3.13 and prior, max-heaps are simulated with
negative values. The negation causes Decimal inputs to apply context
rounding, making the results slightly different than that obtained
by statistics.median().
