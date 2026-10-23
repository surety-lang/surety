```surety
# For every n at least 2, the output is n minus one times the sum of the subfactorial of n minus one and the subfactorial of n minus two.
	n >= 2 -> out == (n - 1) * (subfactorial(n - 1) + subfactorial(n - 2))
# For every n at least 1, the output differs from n times the subfactorial of n minus one by exactly one.
	n >= 1 -> abs(out - n * subfactorial(n - 1)) == 1
# The output is never negative.
	out >= 0
```