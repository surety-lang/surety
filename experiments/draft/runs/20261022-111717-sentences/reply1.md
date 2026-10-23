```surety
// draft
// drafted
type LineItem = { amount: Int }

invoice_total(items: LineItem[], rate: Float) -> Int
	note "sum the line items in cents, add tax at the rate, and round half-even to a whole cent"
	# The total is the sum of the line amounts with tax at the rate added, rounded half-even to a whole cent; a sibling obligation decides the rounding.
	# If the rate is at least 0 and every line amount is at least 0, the total is at least the sum of the line amounts.
	# The total does not depend on the order of the line items.
	// example wanted: no line items, rate 0.5 -> 0
	// example wanted: amounts [100, 250], rate 0 -> 350 (a zero rate leaves the sum unchanged)
	// example wanted: amounts [3], rate 0.5 -> 4 (4.5 rounds half-even down to 4)
	// example wanted: amounts [5], rate 0.5 -> 8 (7.5 rounds half-even up to 8)
	// example wanted: amounts [200, 50], rate 0.5 -> 375 (no rounding needed)
	// example wanted: amounts [7], rate 0.25 -> 9 (8.75 rounds to the nearest cent, not a tie)
```