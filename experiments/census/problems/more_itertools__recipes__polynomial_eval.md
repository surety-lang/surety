more_itertools.recipes.polynomial_eval (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `polynomial_eval` takes the required arguments `coefficients`, `x`.

The docstring:

Evaluate a polynomial at a specific value.

Computes with better numeric stability than Horner's method.

Evaluate ``x^3 - 4 * x^2 - 17 * x + 60`` at ``x = 2.5``:

>>> coefficients = [1, -4, -17, 60]
>>> x = 2.5
>>> polynomial_eval(coefficients, x)
8.125

Note that polynomial coefficients are specified in descending power order.

Supports all numeric types: int, float, complex, Decimal, Fraction.
