```surety
// draft
// drafted
type Order = { amount: Int, customer_email: Text }

drop_orders_over_100(orders: Order[]) -> Order[]
	note "filter out any order whose amount is greater than 100, keeping the rest"
	# Every output order has an amount of at most 100.
	# Every input order left out of the output has an amount greater than 100.
	# The output keeps the input orders in their original order.
	// example wanted: no orders in, no orders out
	// example wanted: an order with amount exactly 100 is kept; one with amount 101 is dropped
	// example wanted: every order over 100, so the output is empty
	// example wanted: two identical orders with amount 50 are both kept, in place
	// example wanted: orders of 40, 150 and 60 give 40 then 60
```