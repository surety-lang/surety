```surety
# If n is at least 0, the output has n times as many elements as the iterable.
	n >= 0 -> len(out) == n * len(iterable)
# If n is less than 0, the output is empty.
	n < 0 -> len(out) == 0
# Every output element's cycle is at least 0 and less than n.
	all o in out: o.cycle >= 0 and o.cycle < n
# For every output element, the output's elements from position its cycle times the iterable's length up to but not including the next cycle's start have, in order, the iterable's items.
	all o in out: out[o.cycle * len(iterable) .. (o.cycle + 1) * len(iterable)].item == iterable
# For every output element, every output element from position its cycle times the iterable's length up to but not including the next cycle's start carries that same cycle.
	all o in out: all p in out[o.cycle * len(iterable) .. (o.cycle + 1) * len(iterable)]: p.cycle == o.cycle
```