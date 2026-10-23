```surety
# If no item satisfies the predicate, the result is true exactly when n is 0.
	not (out <-> n == 0) -> any i in items: satisfies(i)
# If every item satisfies the predicate, the result is true exactly when n equals the number of items.
	not (out <-> n == len(items)) -> any i in items: not satisfies(i)
# If n is negative, the result is false.
	n < 0 -> not out
# If n is more than the number of items, the result is false.
	n > len(items) -> not out
# If the last item satisfies the predicate, the result is the result on the items without the last one with n reduced by one.
	all last in items[len(items) - 1 .. len(items)] where satisfies(last): out <-> exactly_n(items[0 .. len(items) - 1], n - 1)
# If the last item does not satisfy the predicate, the result is the result on the items without the last one with the same n.
	all last in items[len(items) - 1 .. len(items)] where not satisfies(last): out <-> exactly_n(items[0 .. len(items) - 1], n)
```