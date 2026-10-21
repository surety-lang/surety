Obligation `p5b.sur`:

```
type Order = { amount: Int, customer_email: Text }
get_first_occurrence_per_email(orders: Order[]) -> Order[]
	# Every customer email in the input orders appears exactly once in the output orders
	all o in orders: o.customer_email in out.customer_email
	unique out.customer_email
	
	# Every same email leftover is later in the orders list
	all o in out:
		all l leftover in orders where o.customer_email == l.customer_email:
			o before l in orders
			
	# The output is in the original input order
	out same_order_as orders
	# Running the original function on its own output results in the same output
	get_first_occurrence_per_email(out) == out
	
	e:
		in: [
			{amount: 2, customer_email: "j@test.com"},
			{amount: 4, customer_email: "j@test.com"}
		]
		out: [
			{amount: 2, customer_email: "j@test.com"}
		]
	e:
		in: []
		out: []
	e:
		in: [
			{amount: 2, customer_email: "j@test.com"},
			{amount: 4, customer_email: "j@test.com"},
			{amount: 1, customer_email: "q@abctest.com"},
			{amount: 51, customer_email: "q@abctest.com"},
			{amount: 33, customer_email: "q@abctest.com"},
			{amount: 44, customer_email: "j@test.com"},
			{amount: 55, customer_email: "q@abctest.com"}
		]
		out: [
			{amount: 2, customer_email: "j@test.com"},
			{amount: 1, customer_email: "q@abctest.com"},
		]
```

Your module must define: `get_first_occurrence_per_email`.