more_itertools.recipes.factor (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `factor` takes the required argument `n`.

The docstring:

Yield the prime factors of n.

>>> list(factor(360))
[2, 2, 2, 3, 3, 5]

Finds small factors with trial division.  Larger factors are
either verified as prime with ``is_prime`` or split into
smaller factors with Pollard's rho algorithm.
