| #   | Problem                    | Transformation                                                                                      | What it forces you to decide                                                |
| --- | -------------------------- | --------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| 1   | Reshape                    | `{first_name, last_name, email}` → `{name, email}` with the names joined                            | Record construction, string concat, the simplest map                        |
| 2   | Threshold filter           | Orders with `amount > 100`                                                                          | Predicate syntax, numeric comparison                                        |
| 3   | Total by customer          | Group orders by customer, sum amounts                                                               | What `group` returns, how keys are extracted                                |
| 4   | Top N                      | Highest 3 scores, ties broken by name                                                               | Sort ordering spec, stability. Property: output length ≤ N and sorted       |
| 5   | Dedupe by key              | Keep first occurrence per email                                                                     | What “first” means, ordering guarantees. Property: idempotent               |
| 6   | Flatten nested             | Orders with line items → flat rows carrying the order id                                            | Nested lists, flatMap, field hoisting                                       |
| 7   | Join                       | Orders enriched with user name by `user_id`; some orders have no user                               | Inner vs left join, optionals, whether the missing case is a sum type       |
| 8   | Validate with accumulation | Signup form: email shape, age 13–120, password length ≥ 12; return the clean record or _all_ errors | `Result` as a sum, refinement types, accumulate rather than short-circuit   |
| 9   | Parse a CSV line           | Split a line into fields, honoring quoted commas                                                    | String recursion, partial failure, whether chars are a type                 |
| 10  | Temperature conversion     | Celsius ↔ Fahrenheit                                                                                | Floats, tolerance. Property: round-trip within epsilon                      |
| 11  | Invoice total in cents     | Sum line items, apply tax rate, round half-even                                                     | Integer money, how rounding is specified, no floats                         |
| 12  | Date bucketing             | Group events by month from ISO-8601 strings                                                         | Whether dates are a primitive or a string you parse; library boundary       |
| 13  | Pagination                 | List + page + size → page contents, total pages, has_next                                           | Refinement (`page ≥ 1`, `size > 0`), empty and last-page edges              |
| 14  | Snapshot diff              | Old and new inventory by SKU → `{added, removed, changed}`                                          | Maps, set operations, sum of change kinds. Property: apply(old, diff) = new |
| 15  | Running balance            | Ledger entries → each entry with its running balance                                                | Scan (a fold that emits a list), order dependence                           |
| 16  | JSON path with defaults    | From a loose JSON value, pull `user.address.city` with a default                                    | A recursive `Json` sum type, optionals, default handling                    |
| 17  | Tree flatten               | Category tree → flat list with depth and path                                                       | Recursive record types, structural recursion, totality                      |
| 18  | Phone normalization        | Free-text phone → E.164 given a default country, or an error                                        | String ops, refined output type `Phone`. Property: idempotent               |
| 19  | Interval merge             | Overlapping `[start, end]` ranges → merged                                                          | Sort plus fold with lookahead. Property: output sorted and non-overlapping  |
| 20  | Histogram                  | Numbers into fixed-width buckets → bucket counts                                                    | Integer division, map keyed by bucket. Property: counts sum to input length |
| 21  | Pivot                      | Long rows `{month, product, amount}` → wide `{month, amounts by product}`                           | Maps of maps, missing cells default to 0, column order                      |
| 22  | Serialize and parse        | Record ↔ query string                                                                               | Both directions in one obligation. Property: parse(serialize(x)) = x        |

```
Problem: Take first and last name and combine into name, preserve email

type Customer = {first_name: Text, last_name: Text, email: Text}
combine(obj: Customer) -> {name: Text, email: Text}
	note "Combine customer first & last into single name, preserve rest"
	e: 
		in: {first_name: "Joe", last_name: "Shmoe", email: "test@test.com"}
		out: {name: "Joe Shmoe", email: "test@test.com"}
	e:
		in: {first_name: "", last_name: "Show", email: "test@gmail.com"}
		out: {name: "Show", email: "test@gmail.com"}
```


```
Problem: filter out any order with amount > 100

type Order = { amount: Int, customer_email: Text }
filter(o: Order[]) -> Order[]
	note "keep orders at amount 100 or lower"
	e:
		in: [{amount: 4, customer_email: "test@test.com"}, {amount: 42143, customer_email: "test@test.com"}, {amount: 52, customer_email: "test@test.com"}, {amount: 99, customer_email: "test@test.com"}, {amount: 100, customer_email: "test@test.com"}] 
		out: [{amount: 4, customer_email: "test@test.com"}, {amount: 52, customer_email: "test@test.com"}, {amount: 99, customer_email: "test@test.com"}, {amount: 100, customer_email: "test@test.com"}] 
	e:
		in: []
		out: []
```



```
Problem 3: group orders by customer,  sum amounts

type Customer = { first_name: Text, last_name: Text, email: Text }
type Order = { amount: Int, customer_email: Text }


group_by_customer(orders: Order[]) -> [{customer_email: Text, orders: Order[], total_amount: Int }]
	note "group orders by customer, sum amounts per customer"
	e:
		in: [{amount: 4, customer_email: "test@test.com"}, {amount: 42143, customer_email: "test@test.com"}, {amount: 52, customer_email: "jess@test.com"}, {amount: 99, customer_email: "jess@test.com"}, {amount: 100, customer_email: "joe@abe.com"}]
		out: [{
				customer_email: "test@test.com",
				orders: [
					{amount: 4, customer_email: "test@test.com"},
					{amount: 42143, customer_email: "test@test.com"}
				],
				total_amount: 42147
			}, 
			{
				customer_email: "jess@test.com"
				orders: [
					{amount: 52, customer_email: "jess@test.com"},
					{amount: 99, customer_email: "jess@test.com"}
				],
				total_amount: 151
			},
			{
				customer_email: "joe@abe.com"
				orders: [
					{amount: 100, customer_email: "joe@abe.com"},
				],
				total_amount: 100
			}]
	
```



### Re-do with properties

```
Problem 1: Take first and last name and combine into name, preserve email

type Customer = {first_name: Text, last_name: Text, email: Text}

combine(c: Customer) -> {name: Text, email: Text}
	note "combine a customers' `first_name` & `last_name` into a single `name`"
	out.email == c.email
	e: 
		in: { first_name: "Joe", last_name: "Shmoe", email: "test@test.com" }
		out: { name: "Joe Shmoe", ... }
	e:
		in: { first_name: "", last_name: "Show", email: "test@test.com"  }
		out: { name: "Show", ... }
	
```

```
Problem 2: filter out any order with amount > 100

type Order = {amount: Int, customer_email: Text}
get_small_orders(orders: Order[]) -> Order[]
	note "keep orders at amount 100 or lower"
	# Every output order is an input order
	# Every output order has an amount of 100 or lower
	# Every input order with an amount of 100 or lower is in the output
	# Output orders are in the same order as the input
	e:
		in: [{amount: 4, customer_email: "test@test.com"}, {amount: 42143, customer_email: "test@test.com"}, {amount: 52, customer_email: "test@test.com"}, {amount: 99, customer_email: "test@test.com"}, {amount: 100, customer_email: "test@test.com"}] 
		out: [{amount: 4, ... }, {amount: 52, ... }, {amount: 99, ... }, {amount: 100, ...}] 
	e:
		in: []
		out: []
		
		
Notation attempts:
all in out: x in orders
	all x in out: x.amount <= 100
	all x in out: x.amount <= 100 -> x in orders
```

```
Problem 3: group orders by customer,  sum amounts

type Customer = { first_name: Text, last_name: Text, email: Text }
type Order = { amount: Int, customer_email: Text }
type OrderGroup = { customer_email: Text, orders: Order[], total_amount: Int }

group_by_customer(orders: Order[]) -> OrderGroup[]
	note "group orders by customer, sum amounts per customer"
	# For every distinct customer_email in the input, there is exactly one group with that email
	# No two groups share a customer_email
	# Every order in a group has that group's customer_email
	# The orders across all groups, taken together, are the same elements as the input orders
	# Each group's total_amount is the sum of the amounts of its orders
	# Order of groups, and of orders within a group, is unspecified
	
	e:
		in: [{amount: 4, customer_email: "test@test.com"}, {amount: 42143, customer_email: "test@test.com"}, {amount: 52, customer_email: "jess@test.com"}, {amount: 99, customer_email: "jess@test.com"}, {amount: 100, customer_email: "joe@abe.com"}]
		out: [{
				customer_email: "test@test.com",
				orders: [
					{amount: 4, customer_email: "test@test.com"},
					{amount: 42143, customer_email: "test@test.com"}
				],
				total_amount: 42147
			}, 
			{
				customer_email: "jess@test.com"
				orders: [
					{amount: 52, customer_email: "jess@test.com"},
					{amount: 99, customer_email: "jess@test.com"}
				],
				total_amount: 151
			},
			{
				customer_email: "joe@abe.com"
				orders: [
					{amount: 100, customer_email: "joe@abe.com"},
				],
				total_amount: 100
			}]

reached for: unique modifiers or easier flattening comparison, almost wanted to use indexes e.g for x, i in grouped_orders


notation attempts
# For every order in the input, the order group's email must match order email
	all g in out: all(g.orders).customer_email == g.customer_email
# For every order in the input, it exists in the output
all g.orders o in out: o in orders
# For every order group, the total_amount is the sum of all its orders
all g in out: g.total_amount == sum(all(g.orders).amount)
# Every order appears exactly once in all(g.orders)
unique all(all(g).orders)
# the input and the flattened grouped orders have the same elements
# (NEED TO FIGURE OUT SYNTAX)
# No two groups share an email
all g in out: unique(g.customer_email)
```

```
Problem 4 (Top N) Highest 3 scores, ties broken by name (A-Z ascending)

type Score = {score: Float, username: Text}
# (I reached for [0:3] because we know the exact amount of outputs we want)
# one oddity of this is the baked in 0 index offsets (almost want to write as 0:2 but im really saying this output can return 0:3 elements exactly)
top_three_scores(scores: Score[]) -> Score[0:3]
	# Only return up to three scores
	# Every score in the output is >= every score not in the output
	# Ties should be ascending A-Z by username
	# Order matters; highest scores earliest in list
	# Every output element is an input element
	e:
		in: [
			{score: 4.0, username: "Aaly"},
			{score: 4.3, username: "Ben"},
			{score: 4.3, username: "Abe"},
			{score: 4.5, username: "Charlie"}
		]
		out: [
			{score: 4.5, username: "Charlie"}, # Charlie > both because score >
			{score: 4.3, username: "Abe"}, # abe wins > ben with score ==
			{score: 4.3, username: "Ben"}
		]
	e:
		in: [
			{score: 4.0, username: "Aaly"},
		]
		out: [
			{score: 4.0, username: "Aaly"}
		]
	e:
		in: [
			{score: 4.0, username: "Aaly"},
			{score: 4.3, username: "Ben"},
		]
		out: [
			{score: 4.3, username: "Ben"},
			{score: 4.0, username: "Aaly"}
		]
	e:
		in: []
		out: []
```


```
Problem 5a: earliest per key, keep first occurrence of order per email

type Order = { amount: Int, customer_email: Text, ordered_at: Time }

get_first_customer_orders(orders: Order[]) -> Order[]
	# Every customer_email in the input orders appears exactly once in the output orders
	# For every output order, no input order with the same email has an earlier ordered_at
	# Every order returned must exist in orders
	# Order of return list does not matter
	e:
		in: [
			{amount: 2, customer_email: "j@test.com", ordered_at: "2026-10-19T14:19:01Z"}
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
			{amount: 2, customer_email: "j@test.com", ordered_at: "2026-10-19T14:19:01Z"}
			{amount: 4, customer_email: "j@test.com", ordered_at: "2026-09-19T14:19:01Z"},
			{amount: 42, customer_email: "arj@abctest.com", ordered_at: "2026-09-19T14:19:01Z"},
			{amount: 22, customer_email: "arj@abctest.com", ordered_at: "2025-09-19T14:19:01Z"}
		]
		out: [
			{amount: 4, customer_email: "j@test.com", ordered_at: "2026-09-19T14:19:01Z"},
			{amount: 22, customer_email: "arj@abctest.com", ordered_at: "2025-09-19T14:19:01Z"}
		]
		
		
reached for: new Time primitive, needed fully written ISO timestamp which is annoying; needs easier way to write in examples.
```

```
Problem 5b: Dedupe & Idempotence; keep the first occurrence by list position per email

type Order = { amount: Int, customer_email: Text }
get_first_occurrence_per_email(orders: Order[]) -> Order[]
	# Every customer email in the input orders appears exactly once in the output orders
	# For every output order, no earlier input order in the list has the same customer email
	# Every order returned must exist in orders
	# The output is in the original input order
	# Running the original function on its own output results in the same output
	e:
		in: [
			{amount: 2, customer_email: "j@test.com"}
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

reached for: needing to specify order of output
```

```
Problem 6: Flatten nested Orders with line items → flat rows carrying the order id
type OrderItemRow = { id: UUID, amount: Int, item_id: UUID }
type Order = { id: UUID, customer_email: Text, items: OrderItemRow[] }
type OrderLineItem = { line_id: UUID, order_id: UUID, customer_email: Text, amount: Int, item_id: UUID }

flatten_order(o: Order) -> OrderLineItem[]
	# For every output row, there is exactly one input item with the same `id`, and the row’s `amount` and `item_id` are that item’s
	# For every input item, there is exactly one output row with its id
	# Every output row carries the input order's id as order_id and its customer_email as customer_email
	# Output rows are in the same order as the input items
	
	e:
		in:  { 
			id: "o1",
			customer_email: "j@test.com", 
			items: [
		        { id: "li1", amount: 2, item_id: "it9" },
		        { id: "li2", amount: 1, item_id: "it4" }
			]
		}

		out: [ 
			{ 
				order_id: "o1",
				customer_email: "j@test.com",
				line_id: "li1",
				amount: 2,
				item_id: "it9"
			},
			{ order_id: "o1", customer_email: "j@test.com", line_id: "li2", amount: 1, item_id: "it4" }
		]
	e:
		in:  { 
			id: "o1",
			customer_email: "j@test.com", 
			items: []
		}
		out: []
	
		       
reaching for: type definitions shared via import & export in the graph somehow with easy viewing / type inference both for model and human
the one-to-one correspondence between two lists (sentences 1 and 2 as a pair)
```

```
Problem 7 Orders enriched with user name by `user_id`; some orders have no user  Inner vs left join, optionals, whether the missing case is a sum type

type User = { id: UUID, user_name: Text }
type Order = { 
	id: UUID,
	customer_email: Text,
	amount: Int,
	user_id: Option<UUID>
}
type EnrichedOrder = { ...Order, user_name: Option<Text> }

enrich_orders(orders: Order[], users: User[]) -> EnrichedOrder[]
	# For every input order, there is exactly one output order with the same id, customer_email, amount, and user_id
	# For every output order, there is exactly one input order with the same id, customer_email, amount, and user_id
	# For every output order, user_name is the name of the input user if the order's user_id matches, otherwise user_name is None.
	# Output rows are in the same order as input items
	
	e:
	note "simple test case with happy path"
		in: ([
			{ 
				id: "o1",
				customer_email: "j@test.com",
				amount: 2,
				user_id: None
			},
			{ 
				id: "o2",
				customer_email: "j@test.com",
				amount: 1,
				user_id: "u1a"
			},
		], 
		[
			{
				id: "u1a",
				user_name: "jess_test"
			}
		])
		out: [
			{ 
				id: "o1",
				customer_email: "j@test.com",
				amount: 2,
				user_id: None,
				user_name: None
			},
			{ 
				id: "o2",
				customer_email: "j@test.com",
				amount: 1,
				user_id: "u1a"
				user_name: "jess_test"
			},
		]
	e:
	note "no users provided or present in orders"
		in: ([
			{ 
				id: "o1",
				customer_email: "j@test.com",
				amount: 2,
				user_id: None
			},
			{ 
				id: "o2",
				customer_email: "j@test.com",
				amount: 2,
				user_id: None
			},
		],
		[])
		out: [
			{ 
				id: "o1",
				customer_email: "j@test.com",
				amount: 2,
				user_id: None,
				user_name: None
			},
			{ 
				id: "o2",
				customer_email: "j@test.com",
				amount: 2,
				user_id: None,
				user_name: None
			},
		]
	e:
	note "users provided do not match orders"
		in: ([
			{ 
				id: "o1",
				customer_email: "a@test.com",
				amount: 2,
				user_id: None
			},
			{ 
				id: "o2",
				customer_email: "a@test.com",
				amount: 2,
				user_id: "u2b"
			},
		],
		[{
			id: "u1a",
			user_name: "jess_test"
		}])
		out: [
			{ 
				id: "o1",
				customer_email: "a@test.com",
				amount: 2,
				user_id: None,
				user_name: None
			},
			{ 
				id: "o2",
				customer_email: "a@test.com",
				amount: 2,
				user_id: "u2b",
				user_name: None
			},
		]
reaching for: users typically have data they are trying to maniuplate in a raw format of one form or another, seeding example data automatically into the human spec env would be nice as a tooling set to aid in the 'annoyance' of spec definition -> similarly for type inference likely
- again reaching for order mattering or not
```


```
Problem 8 - Validate with accumulation
Signup form: email shape, age 13–120, password length ≥ 12; return the clean record or _all_ errors

type Payload = { email: Text, age: Int, password: Text }

type ValidationError = { code: Text, display: Text }

email_error: ValidationError = {
	code: "e-email-shape",
	display: "Please provide a valid email"
}

age_error: ValidationError = {
	code: "e-age-invalid",
	display: "You are not old enough or did not enter a valid birthday."
}

password_error: ValidationError = {
	code: "e-pass-length",
	display: "Your password must be at least 12 characters" 
}


validate_signup(p: Payload) -> Result<Payload, ValidationError[]>
	# The output is Ok(p) with p equal to the input if and only if no check fails; otherwise it is Err with the failing errors
	# email_error is in the output if and only if the email is misformed
	# age_error is in the Err list if and only if the age is outside 13–120
	# password_error is in the Err list if and only if the password has fewer than 12 characters
	# Each error appears at most once
	e:
		in: { email: "test@test.com", age: 41, password: "asb32fslasdlkal343"}
		out: Ok({email: "test@test.com", age: 41, password: "asb32fslasdlkal343"})
	e:
		in: { email: "test", age: 4, password: "pass123" }
		out: Err([email_error, age_error, password_error])
	e:
		in: { email: "test", age: 42, password: "asb32fslasdlkal343" }
		out: Err([email_error])
	e:
		in: { email: "test@test.com", age: 10, password: "asb32fslasdlkal343" }
		out: Err([age_error])
	e:
		in: { email: "test@test.com", age: 45, password: "pass123" }
		out: Err([password_error])
	e:
		in: { email: "test", age: 45, password: "pass123" }
		out: Err([email_error, password_error])
	e:
	note "age lower boundary, 12 fails" in: { email: "test@test.com", age: 12, password: "asb32fslasdlkal343" } out: Err([age_error]) 
	e: 
	note "age lower boundary, 13 passes"
		in: { email: "test@test.com", age: 13, password: "asb32fslasdlkal343" } 
		out: Ok({ email: "test@test.com", age: 13, password: "asb32fslasdlkal343" })
	e:
	note "age upper boundary, 120 passes"
		in: { email: "test@test.com", age: 120, password: "asb32fslasdlkal343" } 
		out: Ok({ email: "test@test.com", age: 120, password: "asb32fslasdlkal343" })
	e:
	note "age upper boundary, 121 fails"
		in: { email: "test@test.com", age: 121, password: "asb32fslasdlkal343" } 
		out: Err([age_error])
	e:
	note "password boundary, 11 characters fails"
		in: { email: "test@test.com", age: 41, password: "abcdefghijk" }
		out: Err([password_error])
	e: note "password boundary, 12 characters passes"
		in: { email: "test@test.com", age: 41, password: "abcdefghijkl" }
		out: Ok({ email: "test@test.com", age: 41, password: "abcdefghijkl" })

Reaching for:
- SensitiveText (some sort of text type to let model know it is sensitive) - possibly due with Trust definitions
- Internationalization built in for text / errors
- Named predicates: "the email is misformed" cannot be stated in the property fragment. The obligation needs a way to declare a boolean function (well_formed_email, valid_age, long_enough) that properties reference. Same mechanism the shape library will need.
- Named example values: the same payload literal is repeated nine times across the examples; the second copy-paste pass introduced two wrong examples this afternoon.
```

## Outstanding questions
- How do we handle async (multi-threading)? Ideally this would be completely abstract from the user but I think it may also matter in some cases. Needs more thought.


# 

## Decision: aggregates and built-ins (2026-10-20)

Quantifiers produce truth; aggregates produce values. `all` cannot accumulate across
elements, so a sum is never expressible by nesting quantifiers. Every aggregate needs a name.

A name resolves to one of two places:
- Built-in: evaluated by the checker. The property is checkable without trusting any performance.
- Graph reference: another definition (`sum(xs: Int[]) -> Int` as an obligation with its own
  properties and performance). Checking the property runs that node's performance, so this
  property's evidence depends on that node's evidence.

Rule: the checker owns a small fixed set so properties stay checkable on their own:
sum, len, min, max, count. Anything else is a graph reference and follows the S12 rule.

The spelling is identical either way: `g.total_amount == sum(g.orders.amount)`.
Only the resolver differs.

Closes Reached-for "broadcast construct all(xs).field": the broadcast rule covers it.