```surety
// draft
// drafted
type Order = { amount: Int, customer_email: Text }

filter_out_large_orders(orders: Order[]) -> Order[]
	note "drop every order whose amount is greater than 100 and keep the rest unchanged"
	# Every output order has an amount of at most 100.
	# No input order with an amount greater than 100 is in the output.
	# Every input order with an amount of at most 100 is in the output.
	# An order with an amount of exactly 100 is kept.
	# Every output order is an input order, unchanged.
	# An order that appears more than once in the input with an amount of at most 100 appears in the output exactly as many times.
	# The output has as many orders as the input has orders with an amount of at most 100.
	# The output keeps the orders in the same order as the input.
```