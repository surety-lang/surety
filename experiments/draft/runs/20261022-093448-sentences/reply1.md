```surety
// draft
// drafted
type LineItem = { amount_cents: Int }

invoice_total(items: LineItem[], tax_rate_bps: Int) -> Int
	note "sum the line items' amounts in cents, apply the tax rate (in basis points), and round the result half-even to a whole cent"
	# The total is the items' taxed subtotal rounded half-even to the cent, which a sibling obligation decides.
	# If the length of items is 0, the total is 0.
	# If every item in items has an amount_cents of 0, the total is 0.
	# If the tax rate is 0 and the length of items is 1, the total is that item's amount_cents.
	# If the tax rate is at least 0, the length of items is 1, and that item's amount_cents is at least 0, the total is at least that item's amount_cents.
	# If every item in items has an amount_cents of at least 0 and the tax rate is at least 0, the total is at least 0.
	# If every item in items has an amount_cents of at least 0 and the tax rate is at least 0, every item in items has an amount_cents of at most the total.
```