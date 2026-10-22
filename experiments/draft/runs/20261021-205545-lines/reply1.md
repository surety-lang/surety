```surety
# Every output order has an amount of at most 100.
	all o in out: o.amount <= 100
# No input order with an amount greater than 100 is in the output.
	none o in orders where o.amount > 100: o in out
# Every input order with an amount of at most 100 is in the output.
	all o in orders where o.amount <= 100: o in out
# An order with an amount of exactly 100 is kept.
	all o in orders where o.amount == 100: o in out
# Every output order is an input order, unchanged.
	out drawn_from orders
# An order that appears more than once in the input with an amount of at most 100 appears in the output exactly as many times.
	reaching for: for each input order with an amount of at most 100 whose value occurs more than once in orders, the number of occurrences of that value in out equals the number of occurrences of it in orders; the notation has no way to count the occurrences of one value in a list
# The output has as many orders as the input has orders with an amount of at most 100.
	reaching for: len(out) == the number of occurrences in orders whose amount is at most 100; the notation has no filtered count (a count or len over `orders where o.amount <= 100`)
# The output keeps the orders in the same order as the input.
	out same_order_as orders
```