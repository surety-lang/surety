```surety
# The output is a prime, as a sibling obligation on primality decides.
	is_prime(out)
# If approximate is false, the number of primes less than the output is n, a count a sibling obligation decides.
	not approximate -> prime_count_below(out) == n
# If approximate is false, the output for n is less than the output for n plus one, also with approximate false.
	not approximate -> out < nth_prime(n + 1, false)
# If approximate is false, the output is more than twice n.
	not approximate -> out > 2 * n
```