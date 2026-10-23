more_itertools.recipes.matmul (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `matmul` takes the required arguments `m1`, `m2`.

The docstring:

Multiply two matrices.

>>> list(matmul([(7, 5), (3, 5)], [(2, 5), (7, 9)]))
[(49, 80), (41, 60)]

The caller should ensure that the dimensions of the input matrices are
compatible with each other.

Supports all numeric types: int, float, complex, Decimal, Fraction.
