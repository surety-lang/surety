more_itertools.more.nth_prime (from more-itertools 11.2.1)

Signature, in words: `nth_prime` takes the required argument `n`; the keyword-only argument `approximate` (default `False`).

The docstring:

Return the nth prime (counting from 0).

>>> nth_prime(0)
2
>>> nth_prime(100)
547

If *approximate* is set to True, will return a prime close
to the nth prime.  The estimation is much faster than computing
an exact result.

>>> nth_prime(200_000_000, approximate=True)  # Exact result is 4222234763
4217820427
