Obligation `p5a.sur`:

```
type Order = { amount: Int, customer_email: Text, ordered_at: Time }

get_first_customer_orders(orders: Order[]) -> Order[]
	all o in orders: o.customer_email in out.customer_email
	unique out.customer_email

	all o in out:
		all l_o leftover in orders where o.customer_email == l_o.customer_email:
			o.ordered_at <= l_o.ordered_at 
			
	out drawn_from orders
	out unordered

	# Two orders with the same email and the same ordered_at: either may be kept. Decided, not forgotten.
	accept any a in orders: any b in orders where b != a and b.customer_email == a.customer_email: b.ordered_at == a.ordered_at
	
	
	// `unordered` isn’t saying something about the elements
	// it’s the explicit “no claim here”:
	// order is stated or stated not to matter.
	// Leave it out and the linter should ask you which.
	
	e:
		in: [
			{amount: 2, customer_email: "j@test.com", ordered_at: "2026-10-19T14:19:01Z"},
			{amount: 4, customer_email: "j@test.com", ordered_at: "2026-09-19T14:19:01Z"}
		]
		out: [
			{amount: 4, customer_email: "j@test.com", ordered_at: "2026-09-19T14:19:01Z"}
		]
	e:
		in: []
		out: []
	e:
		in: [
			{amount: 2, customer_email: "j@test.com", ordered_at: "2026-10-19T14:19:01Z"},
			{amount: 4, customer_email: "j@test.com", ordered_at: "2026-09-19T14:19:01Z"},
			{amount: 42, customer_email: "arj@abctest.com", ordered_at: "2026-09-19T14:19:01Z"},
			{amount: 22, customer_email: "arj@abctest.com", ordered_at: "2025-09-19T14:19:01Z"}
		]
		out: [
			{amount: 4, customer_email: "j@test.com", ordered_at: "2026-09-19T14:19:01Z"},
			{amount: 22, customer_email: "arj@abctest.com", ordered_at: "2025-09-19T14:19:01Z"}
		]
```

Your module must define: `get_first_customer_orders`.