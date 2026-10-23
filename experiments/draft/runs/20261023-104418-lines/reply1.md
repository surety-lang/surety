```surety
# If initial is absent, the output has as many elements as the iterable.
	initial == None -> len(out) == len(iterable)

# If initial is absent, the output starts with the iterable's first element.
	initial == None and len(iterable) > 0 -> out[0 .. 1] == iterable[0 .. 1]

# If initial is absent, the output sums to the iterable's last element.
	initial == None -> sum(out) == sum(iterable[len(iterable) - 1 .. len(iterable)])

# If the iterable has at least two elements, the output's last element is the iterable's last element minus the element before it.
	len(iterable) >= 2 -> len(out) >= 1 and sum(out[len(out) - 1 .. len(out)]) == sum(iterable[len(iterable) - 1 .. len(iterable)]) - sum(iterable[len(iterable) - 2 .. len(iterable) - 1])

# The output for the iterable without its last element, with the same initial, is the output without its last element.
	difference(iterable[0 .. len(iterable) - 1], initial) == out[0 .. len(out) - 1]

# If initial is present, the output is the output for the same iterable with initial absent, with its first element left out.
	initial != None -> out == difference(iterable, None)[1 .. len(iterable)]
```