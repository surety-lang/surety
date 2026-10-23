more_itertools.recipes.polynomial_from_roots (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `polynomial_from_roots` takes the required argument `roots`.

The docstring:

Compute a polynomial's coefficients from its roots.

>>> roots = [5, -4, 3]            # (x - 5) * (x + 4) * (x - 3)
>>> polynomial_from_roots(roots)  # x³ - 4 x² - 17 x + 60
[1, -4, -17, 60]

Note that polynomial coefficients are specified in descending power order.

Supports all numeric types: int, float, complex, Decimal, Fraction.
