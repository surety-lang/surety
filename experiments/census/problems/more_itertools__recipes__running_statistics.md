more_itertools.recipes.running_statistics (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `running_statistics` takes the required argument `iterable`; the keyword-only argument `maxlen` (default none).

The docstring:

Statistics for values seen so far or values in a sliding window.

Set *maxlen* to a positive integer to specify the maximum size
of the sliding window.  The default of *None* is equivalent to
an unbounded window.

Yields instances of a ``Stats`` dataclass with fields for the dataset *size*,
*minimum* value, *median* value, *maximum* value, and the arithmetic *mean*.

Supports numeric types such as int, float, Decimal, and Fraction,
but not complex numbers which are unorderable.
