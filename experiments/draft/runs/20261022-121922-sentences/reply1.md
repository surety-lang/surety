```surety
// draft
// drafted
type LineItem = { amount: Int }

invoice_total(items: LineItem[], rate_bps: Int) -> Int
	note "sum the line items' amounts in cents, apply the tax rate in basis points to that sum, and round the taxed total half-even to a whole cent"
	# The total depends on the items only through the sum of their amounts: the total of the items is the total of a single item whose amount is that sum.
	# If the sum of the amounts is at least 0 and the rate is at least 0, the total is at least the sum of the amounts.
	# If the sum of the amounts is at most 0 and the rate is at least 0, the total is at most the sum of the amounts.
	// example wanted: no items at a nonzero rate; the sum is 0, so nothing is taxed
	// example wanted: amounts [125, 300] at 0 basis points; no tax, so the total is the plain sum
	// example wanted: amounts [125] at 1000 basis points; the taxed total is exactly 137.5, a half-even tie, and this decides whether the tie is taken on the total or on the tax alone
	// example wanted: amounts [135] at 1000 basis points; the taxed total is exactly 148.5, a tie whose even neighbour is the lower one
	// example wanted: amounts [126] at 1000 basis points; the taxed total is 138.6, not a tie, so ordinary nearest-cent rounding decides
	// example wanted: amounts [5, 5] at 1000 basis points; each item taxed alone is a 5.5 tie, but the summed total is exact, and this decides sum-then-round over round-per-item
	// example wanted: amounts [-125] at 1000 basis points; a negative taxed total of exactly -137.5, and this decides whether half-even is symmetric about zero
```