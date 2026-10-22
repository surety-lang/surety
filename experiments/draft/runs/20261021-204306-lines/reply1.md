```surety
# Every input order with an amount of 100 or less is in the output.
	all o in orders where o.amount <= 100: o in out
# No output order has an amount greater than 100.
	none o in out: o.amount > 100
# An order with an amount of exactly 100 is kept.
	all o in orders where o.amount == 100: o in out
# Every output order is an input order, unchanged.
	out drawn_from orders
# A kept order appears in the output as many times as it appears in the input.
	reaching for: for each order o in out, the number of occurrences of o's value in out equals the number of occurrences of o's value in orders. That needs a count over a filtered list (occurrences equal to o), and the notation has no filtered-list or count-where form. `all l leftover in orders: l not in out` gives only "at least as many", not "exactly as many".
# The output is in the same order as the input.
	out same_order_as orders
```