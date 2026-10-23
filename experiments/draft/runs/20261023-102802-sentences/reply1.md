```surety
// draft

ilen(iterable: Int[]) -> Int
	note "return the number of items in the iterable; that it consumes the iterable is about execution and is not claimed"
	# The output is the number of items in the iterable, counting every occurrence of a repeated item.

	// example wanted: an empty iterable, the boundary where the loop never runs
	// example wanted: an iterable with one value repeated several times, decided by counting each occurrence rather than each distinct value
	// example wanted: the docstring's case, the primes below 1,000 as a list, decided by counting them
```