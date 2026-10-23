```surety
# The output is drawn from the input.
	out drawn_from iterable

# The output keeps the input's order.
	out same_order_as iterable

# Every output item whose pred is true comes after some output item whose pred is false and before some output item whose pred is false.
	all o in out where o.pred: any a in out where not a.pred: any b in out where not b.pred: a before o in out and o before b in out

# Every input item left out of the output comes before every input item whose pred is false, or after every one of them.
	all l leftover in iterable: l.pred and none a in iterable where not a.pred: any b in iterable where not b.pred: a before l in iterable and l before b in iterable
```