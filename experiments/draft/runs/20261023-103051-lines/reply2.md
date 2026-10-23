```surety
# The output is Ok exactly when n is at least 1, and otherwise it is n_error.
	(n < 1 and out == Err(n_error)) or n >= 1 and exactly_one Ok(_) in out
# The Ok output's first n elements are the iterable's first n elements.
	all Ok(res) in out: res[0 .. n] == iterable[0 .. n]
# If the iterable has more than n elements, the Ok output's element at position n is the filler e.
	all Ok(res) in out: len(iterable) > n -> res[n .. n + 1] == [e]
# If the iterable has more than n elements, the Ok output from position n plus 1 on is the interspersal of e among the iterable's elements from position n on, with the same n.
	all Ok(res) in out: len(iterable) > n -> intersperse(e, iterable[n .. len(iterable)], n) == Ok(res[n + 1 .. len(res)])
# The iterable's elements appear in the Ok output in the iterable's order.
	all Ok(res) in out: iterable same_order_as res
# The number of fillers is the Ok output's length minus the iterable's length, and either n times the number of fillers is less than the iterable's length or there are no fillers.
	all Ok(res) in out: n * (len(res) - len(iterable)) < len(iterable) or len(res) - len(iterable) == 0
# The iterable's length is at most n times one more than the number of fillers in the Ok output.
	all Ok(res) in out: len(iterable) <= n * (len(res) - len(iterable) + 1)
```