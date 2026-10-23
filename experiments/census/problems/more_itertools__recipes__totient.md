more_itertools.recipes.totient (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `totient` takes the required argument `n`.

The docstring:

Return the count of natural numbers up to *n* that are coprime with *n*.

Euler's totient function φ(n) gives the number of totatives.
Totative are integers k in the range 1 ≤ k ≤ n such that gcd(n, k) = 1.

>>> n = 9
>>> totient(n)
6

>>> totatives = [x for x in range(1, n) if gcd(n, x) == 1]
>>> totatives
[1, 2, 4, 5, 7, 8]
>>> len(totatives)
6

Reference:  https://en.wikipedia.org/wiki/Euler%27s_totient_function
