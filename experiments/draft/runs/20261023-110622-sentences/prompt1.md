The problem, in prose:

more_itertools.recipes.unique_everseen (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `unique_everseen` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Yield unique elements, preserving order. Remember all elements ever seen.

    >>> list(unique_everseen('AAAABBBCCDAABBB'))
    ['A', 'B', 'C', 'D']
    >>> list(unique_everseen('ABBCcAD', str.casefold))
    ['A', 'B', 'C', 'D']

Raises ``TypeError`` for unhashable items.

Some unhashable objects can be converted to hashable objects
using the *key* parameter:

* For ``list`` objects, try ``key=tuple``.
* For ``set`` objects, try ``key=frozenset``.
* For ``dict`` objects, try ``key=lambda x: frozenset(x.items())``
  or in Python 3.15 and later, set ``key=frozendict``.

Alternatively, consider the ``unique()`` itertool recipe.  It sorts
the data and then uses equality to eliminate duplicates.  Hashability
is not required.

The spec's summary of the notation (`docs/Surety Spec v2.md`):

**Generated inputs (2026-10-21).** The domain carries every integer literal in the obligation and its two neighbours, so a threshold is crossed from both sides whether or not an example crosses it. An example seeds the domain, never the run: `check` and `mutants` run on an obligation with no examples, and a claim no example reached is evidenced by the generated tier rather than skipped.

**The property fragment as of 2026-10-21.** Consolidated from eleven obligations and fourteen claim shapes; the full rule list is in the catch-up note and the decision notes, and this is the shape of it. A quantifier head has five slots, each at most once: `all|any|none|exactly_one <binder> [leftover] in <list> [where <cond>]: <body> [else <body>]`; a body runs to the end of the line or is the indented block under it, so a quantifier is always the last operand on a line; binders never leave their quantifier; a pattern in the binder slot (`Ok(p)`, `Err(errs)`, `_`) selects a side of a sum type. Binders range over *occurrences*: `==`, `in`, and the list relations compare values, `before` compares positions, and `leftover` assigns each output element the earliest matching occurrence, so a dedupe obligation is true on duplicates, and `out drawn_from orders` together with `all o leftover in orders: <o was excluded>` pins multiplicity exactly, with no count-where form needed. One list relation underneath the family: `drawn_from` (sub-multiset); `same_order_as` is it plus order (the subsequence relation), `same_elements_as` is it both ways, `unordered` is the explicit no-claim, `sorted_by` takes a lexicographic key, `unique` is whole-list. Connectives `and or not -> <->` with conventional precedence; built-ins `sum len min max count flatten`, with `len` and `in` applying to `Text` as well as to lists; broadcast (`out.id` is a list of ids); a named predicate with a body is an abbreviation inlined before hashing, and one with no body (`well_formed_email`) is a graph reference, so a `Text`'s shape is a sibling obligation's claim and never the fragment's. A precondition that fits on one list or field is a refinement in the brackets (`Item[unique sku]`, `Score[0..3]`), checked at the boundary; one that relates two things is an error case in a `Result`. The frame rule: when a property pairs an output record with an input record, same-named fields are equal unless stated. `accept <boolean over inputs>` marks a deliberate non-decision, in the hash, reported ACCEPTED, a finding if it never fires. Prose above a property is optional and rendered; `#` is a sentence a human chose to pin, `//` is a comment. One sentence per property (2026-10-22): consecutive `#` lines above a property or an accept line are that one sentence, joined by spaces, so a sentence may be wrapped without touching the hash; any further prose is a `//` comment. No line limit in the grammar; the renderer wraps at a column, and everything that writes a file goes through it. A property may call its own obligation (2026-10-22): in a check the call runs the performance under test, so the clause relates two of its outputs, the metamorphic form (the total of the items is the total of one item carrying their sum); under the underdetermination check the call means the mutant, so the clause never catches a mutant on its own and always needs companions that pin values, and the report says so.

The decisions the notation rests on:

`Decision - accept marks a deliberate non-decision.md`:

# Decision: `accept` marks a deliberate non-decision (2026-10-21)

Slice 2's search reports an input as underdetermined when two outputs that differ pass every clause. Some of those are holes; some are choices the author made on purpose. P5a keeps the earliest order per email, and when two orders share an email and a timestamp, either is the intent. Under `--strict` that intent failed the check, because the obligation had no way to say "I know, and I mean it."

## Rule

`accept <boolean over the inputs>` is a line in the property slot. It names the inputs on which the obligation deliberately admits more than one output. For P5a:

```
accept any a in orders: any b in orders where b != a and b.customer_email == a.customer_email: b.ordered_at == a.ordered_at
```

- An underdetermined input that satisfies an `accept` is reported as ACCEPTED, counted, and does not fail `--strict`. One that satisfies no `accept` is UNDERDETERMINED, as before.
- `accept` is part of the obligation and of its hash. It is a claim ("on these inputs, either answer is right"), not a note, so it cannot be a free-text line, and a reviewer sees it where the properties are.
- An `accept` that no example or generated input satisfies is itself a finding (a dead accept), so the line cannot be used to silence the check with a predicate that never fires.
- `accept true` is refused. The predicate has to mention an input.

## Why not a note

A note is pinned to the obligation and left out of the hash; it is for readers. An `accept` changes what the checker reports and what `--strict` means, so it has to be in the hash and in the properties, or anyone could silence a finding with a comment.

## Why not a keyword per kind of non-decision

`unordered` is the same move for one specific non-decision, order, and it earned a word because every list obligation has to say something about order. `accept` is the general form for the rest; if one kind recurs across the corpus the way order did, it can get its own word then.

## What it gives the graph

When the store exists, the accepted list per obligation is the set of `accept` lines, hashed with it. There is no separate list to maintain.

`Decision - an obligation is accepted only after check and mutants have run on it.md`:

# Decision: an obligation is accepted only after `check` and `mutants` have run on it (2026-10-21)

On the detector's first run over the corpus it found that P5b's first property was written in the out-to-in direction while the sentence above it said in-to-out, which admitted the always-empty performance. The author knew the nothing-lost rule from day one and wrote the sentence correctly. A reviewer read the file the same day. Neither saw it. The prose-beside-notation convention was in place and did not help.

## Rule

An obligation is accepted into the graph only after:

1. `check` has run its generated tier over it against at least one passing performance, and
2. `mutants` has run over it, and
3. the author has answered every finding: with a fix to the obligation, a refinement on a type, or an explicit accepted non-decision (the way `unordered` says "no claim here" for order).

Reading is not a check. Running is. A `#` sentence above a property is for the reader; it is not evidence that the property says what the sentence says, and the lint that asks for a sentence on nested quantifiers stays a lint.

## What this changes

- The risk formerly named "humans won't write specs" is renamed **"specs are unsafe without the check."** It is a workflow risk, not a skills risk, and it is cheaper: the workflow is three commands. The number for the risk table: 1 of 11 obligations written by the language's author carried a silent hole on first run.
- Spec v2's data flow already puts the mutant library at the first step of a change; this makes it a gate on acceptance rather than a step in the flow.
- Accepted non-decisions need a place to live. Until the graph store exists (slice 3), the accepted list is a line in the obligation's `note` and a row in the findings report; P5a's tie on `ordered_at` is the first entry.

`Decision - binders range over occurrences; equality is by value, before is by position.md`:

# Decision: binders range over occurrences; equality is by value, `before` is by position (2026-10-21)

Came out of slice 2's first run. P5b (keep the first occurrence per email) is a dedupe, and its central input is two identical orders. On `orders = [X, X]` the property

```
all o in out:
  all l leftover in orders where l.customer_email == o.customer_email:
    o before l in orders
```

was false under the slice 1 semantics: `leftover` subtracted by value and `before` found positions by first occurrence, so `o` and `l` both resolved to position 0 and "the leftover one is later" failed. The intent was satisfiable; the notation could not say "the second copy." A dedupe obligation that is false on duplicates is not acceptable, so this is a language decision, not a residue.

## Rule

- A binder introduced by `in L` or `leftover in L` ranges over **occurrences** of `L`: a position and the value at it.
- `==`, `in`, `not in`, and every list relation (`drawn_from`, `same_order_as`, `same_elements_as`, `unique`) compare **values**. Nothing about positions leaks into them.
- `leftover in L` removes, for each output element, the **earliest occurrence** of an equal value in `L` not yet removed. What remains are occurrences.
- `a before b in L` compares **positions**. An operand that is an occurrence of `L` has its position. An operand that is not (an element of `out`, say) stands for the occurrence of its value that `leftover` assigned to it, i.e. the earliest occurrence not left over; if no `leftover` is in scope, its first occurrence.

Under this, `orders = [X, X]`, `out = [X]`: `out`'s `X` is position 0, the leftover is position 1, `0 before 1` holds, and the obligation is true on its own core input. The observable consequence in the P5b case is the same as the smaller reading the build proposed (`a before b` with `a == b` is true when the value occurs twice), but stated as a rule about what a binder is rather than a special case of one operator.

## What it does not change

Every property in the corpus except P5b's `before` line evaluates identically, because none of them compares positions. The hash is unchanged (occurrence vs value is evaluation, not syntax). `unique` still means "no two occurrences share the key," which it always did.

## Why not the smaller reading

"`a before b` is true for equal values when the list has duplicates" is true as a consequence but wrong as a definition: it would also hold when `out` kept the *second* copy, and it says nothing about three copies. The occurrence rule gives the same answer on P5b and a correct one everywhere else, and it is one sentence about binders rather than one exception about `before`.

`Decision - preconditions are refinement types; brackets hold refinements, angle brackets hold types.md`:

# Decision: preconditions are refinement types (2026-10-20)

Came out of problem 14. `snapshot_diff(old, new)` only makes sense if each input has unique skus. That is a claim about the inputs, and the fragment had no place for one.

## Rule

A precondition that is structural goes in the input's type, as a refinement, and the boundary validator enforces it. There is no `requires` slot.

```
snapshot_diff(old: Item[unique sku], new: Item[unique sku]) -> Diff
```

If a precondition cannot be stated as a type on one input (it relates two inputs, e.g. `page * size <= len(items)`), that is the sign the function should return a `Result` and make it an error case. Problem 13 is the first test of this.

This is the same move as `Score[0..3]` on an output: a value-level fact about a list, carried by the type, checked at the boundary. It says nothing a property would otherwise need to say; `len(out) == min(3, len(scores))` still has to be stated. A refinement is a bound, not a count.

## Brackets

- Angle brackets hold **types**: `Option<T>`, `Result<T, E>`. These are constructors with type parameters.
- Square brackets hold **list refinements**: `Item[]`, `Item[0..3]`, `Item[unique sku]`, and they compose: `Item[0..3, unique sku]`. These are the same list type with a constraint on it, not a new type.

Writing `List<Item>` next to `Item[]`, or `Unique<Item>[]` next to `Item[unique sku]`, would be two spellings of one thing; same objection as `=` vs `==`. `[]` is the list constructor everywhere.

`Unique<Item::sku>[]` was the first attempt and is wrong for the same reason `all g in out: unique g.customer_email` was wrong: it puts uniqueness on the element. Uniqueness is a property of the list, so the refinement wraps the list.

`unique` inside a type is the same word as `unique` in a property, in a type position. `[0..3]` is an inclusive range, not slice notation.

## Reached for

- `Set<T>`: not needed. A `Set` says unordered, and inputs are lists whose order may matter elsewhere (5a, 5b). `Item[unique sku]` is a list with a key constraint, which is what every case so far has meant.

## Addendum (2026-10-21): a precondition that relates two things is an error case

Slice 2 found the case the rule predicted. `apply_diff` is contradictory when one sku is in both `diff.removed` and `diff.changed`: the obligation requires it absent from the result and present in it. No refinement on one list can exclude that input; it relates two fields of one input.

The rule stands as written, with its scope stated more carefully: a precondition that can be expressed as a refinement on **one** list or field goes in the type; a precondition that relates **two** things (two inputs, or two fields of one input) is not a type and becomes an error case in a `Result`. `apply_diff` gained `conflicting_sku_err`, a `disjoint` abbreviation (`none s in diff.removed.sku: s in diff.changed.sku`), and a fourth conjunct in `checks`. Overlaps with `added` are already caught by `added_ok` (added requires the sku absent from `old`; removed and changed require it present), so only the removed/changed pair needed a new error.

Problem 13 (pagination, `page * size <= len(items)`) is the same shape and gets the same answer.

`Decision - prose sentences may lean on the reader; notation may not.md`:

**Date:** 2026-10-19
**Surfaced by:** Problem 7, sentence 2

## The case

Two phrasings of the same property:

- Explicit: *user_name is the name of the input user whose id equals the order's user_id, if such a user exists; otherwise None.*
- Natural: *user_name is the name of the input user if the order's user_id matches, otherwise user_name is None.*

The explicit form is complete but reads stiffly ("whose" was the sticking point). The natural form reads well and leaves "matches *what*" implicit: a human fills in "an input user's id" without noticing.

## The decision

Prose sentences in an obligation are written the natural way. They are for the human: the author checking their own thinking, and a reviewer reading the obligation later. Ambiguity that any reader resolves the same way is acceptable in prose.

The compressed notation is held to the stricter standard. Every comparison names both sides; every quantifier names its range. Nothing is left for a reader to fill in, because the reader is a checker and a model, and neither one gets to guess.

## Consequence

This is a division of labor between the two layers of the spec, not a defect in either. The prose carries intent at reading speed; the notation carries the exact claim. When the consolidation pass compresses the sentence corpus into notation, the implicit phrases in the prose ("matches," "corresponding," "the first") are exactly the places where the notation has to add an operand. Expect each one to show up as a small expansion.

Follow-on: the prose and the notation for the same property should sit next to each other in the obligation, so the expansion is visible and the prose doesn't drift into saying something the notation doesn't check.

The corpus, 11 obligations written by hand:

`apply_diff.sur`:

```
type Item   = { sku: Id, qty: Int }
type Change = { sku: Id, old_qty: Int, new_qty: Int }
type Diff   = { added: Item[unique sku], removed: Item[unique sku], changed: Change[unique sku] }

type DiffError = ValidationError

missing_removed_sku_err: DiffError = {
	code: "e1-missing-removed-sku",
	display: "Item in diff.removed not present in old"
}

missing_changed_sku_err: DiffError = {
	code: "e1-missing-changed-sku",
	display: "Item in diff.changed not present in old"
}

added_already_present_sku_err: DiffError = {
	code: "e1-added-present-sku",
	display: "Item in diff.added already present in old"
}

conflicting_sku_err: DiffError = {
	code: "e1-conflicting-sku",
	display: "A sku is in both diff.removed and diff.changed"
}

valid_errors = [
	missing_removed_sku_err,
	missing_changed_sku_err,
	added_already_present_sku_err,
	conflicting_sku_err
]

apply_diff(old: Item[unique sku], diff: Diff) -> Result<Item[unique sku], DiffError[]>
	removed_ok = all s in diff.removed.sku: s in old.sku
	changed_ok = all s in diff.changed.sku: s in old.sku
	added_ok   = all s in diff.added.sku:   s not in old.sku
	// a sku in both removed and changed asks for it to be absent and present; added overlaps are already caught by added_ok
	disjoint   = none s in diff.removed.sku: s in diff.changed.sku
	checks     = removed_ok and changed_ok and added_ok and disjoint
	
	checks <-> exactly_one Ok(_) in out
	
	all Err(errs) in out: 
		missing_removed_sku_err in errs <-> not removed_ok
	all Err(errs) in out:
		missing_changed_sku_err in errs <-> not changed_ok
	all Err(errs) in out:
		added_already_present_sku_err in errs <-> not added_ok
	all Err(errs) in out:
		conflicting_sku_err in errs <-> not disjoint
	
	all Err(errs) in out:
		errs drawn_from valid_errors
	all Err(errs) in out:
		errs unordered
		
	# Every added item is in the output as-is.
	all Ok(res) in out:
		all i in diff.added:
			i in res
	
	# No removed sku is in the output.
	all Ok(res) in out:
		all sk in diff.removed.sku:
			sk not in res.sku
		
	# Every changed sku is in the output with its qty equal to `new_qty`.
	all Ok(res) in out:
		all c in diff.changed:
			exactly_one i in res where i.sku == c.sku:
				i.qty == c.new_qty
	
	# Every old item whose sku is neither removed nor changed is in the output unchanged.
	all Ok(res) in out:
		all i_o in old where (
			i_o.sku not in diff.removed.sku and i_o.sku not in diff.changed.sku
		): 
			 i_o in res
	  
	  
	# Every output sku is either an old sku or an added sku; nothing else appears.
	all Ok(res) in out:
		all sk in res.sku:
			sk in old.sku or sk in diff.added.sku 
	
	# Output skus are unique; order unspecified.
	all Ok(res) in out: unique res.sku
	all Ok(res) in out: res unordered

	e:
	  note "one of each kind; A untouched"
	  in: ([
	        { sku: "A", qty: 5 },
	        { sku: "B", qty: 2 },
	        { sku: "C", qty: 1 }
	      ],
	      {
	        added:   [ { sku: "D", qty: 7 } ],
	        removed: [ { sku: "C", qty: 1 } ],
	        changed: [ { sku: "B", old_qty: 2, new_qty: 3 } ]
	      })
	  out: Ok([
	        { sku: "A", qty: 5 },
	        { sku: "B", qty: 3 },
	        { sku: "D", qty: 7 }
	      ])

	e:
	  note "empty diff is the identity"
	  in: ([ { sku: "A", qty: 5 } ], { added: [], removed: [], changed: [] })
	  out: Ok([ { sku: "A", qty: 5 } ])
	
	e:
	  in: ([], { added: [], removed: [], changed: [] })
	  out: Ok([])
	
	e:
	  note "removed sku not in old"
	  in: ([ { sku: "A", qty: 5 } ], { added: [], removed: [ { sku: "Z", qty: 9 } ], changed: [] })
	  out: Err([missing_removed_sku_err])
	
	e:
	  note "changed sku not in old"
	  in: ([ { sku: "A", qty: 5 } ], { added: [], removed: [], changed: [ { sku: "B", old_qty: 1, new_qty: 2 } ] })
	  out: Err([missing_changed_sku_err])
	
	e:
	  note "added sku already in old"
	  in: ([ { sku: "A", qty: 5 } ], { added: [ { sku: "A", qty: 6 } ], removed: [], changed: [] })
	  out: Err([added_already_present_sku_err])
	
	e:
	  note "a sku in both removed and changed"
	  in: ([ { sku: "A", qty: 5 } ],
	      { added: [], removed: [ { sku: "A", qty: 5 } ], changed: [ { sku: "A", old_qty: 5, new_qty: 6 } ] })
	  out: Err([conflicting_sku_err])

	e:
	  note "two disagreements, both reported"
	  in: ([ { sku: "A", qty: 5 } ],
		  { added: [ { sku: "A", qty: 6 } ], removed: [ { sku: "Z", qty: 9 } ], changed: [] })
	  out: Err([missing_removed_sku_err, added_already_present_sku_err])
```

`p1.sur`:

```
type Customer = {first_name: Text, last_name: Text, email: Text}

combine(c: Customer) -> {name: Text, email: Text}
	note "combine a customers' `first_name` & `last_name` into a single `name`"
	// out.email == c.email is not required due to frame preservation
	c.first_name in out.name
	c.last_name in out.name
	e: 
		in: { first_name: "Joe", last_name: "Shmoe", email: "test@test.com" }
		out: { name: "Joe Shmoe", ... }
	e:
		in: { first_name: "", last_name: "Show", email: "test@test.com"  }
		out: { name: "Show", ... }
```

`p14.sur`:

```
snapshot_diff(old: Item[unique sku], new: Item[unique sku]) -> Diff
	note "return the added, removed, and changed items in the inventory diff"
	
	# All items in out.added are present only in new and not in old
	all a in out.added: a in new and a.sku not in old.sku
	# Every item in new whose sku is not in old is in added
	all n in new where n.sku not in old.sku: n in out.added
	
	# all items in out.removed are present only in old and not in new
	all r in out.removed: r in old and r.sku not in new.sku
	# Every item in old whose sku is not in new is in removed
	all o in old where o.sku not in new.sku: o in out.removed
	
	# all items in changed are present in both
	all c in out.changed: c.sku in old.sku and c.sku in new.sku
	# Every sku in both, with a different qty, is in changed
	all n in new:
		all o in old where o.sku == n.sku and o.qty != n.qty:
			n.sku in out.changed.sku
	
	# All items in changed carry `old_qty` and `new_qty`, `qty` is removed
	# Which are set respective to new.qty or old.qty
	all c_out in out.changed:
		exactly_one old_entry in old where c_out.sku == old_entry.sku:
			c_out.old_qty == old_entry.qty
		// note wanted to just nest the sub selects e.g
		// exactly_one new...
	
	all c_out in out.changed:
		exactly_one new_entry in new where c_out.sku == new_entry.sku:
			c_out.new_qty == new_entry.qty
	
	# If there is no update, it is unchanged
	all c in out.changed: c.old_qty != c.new_qty
	
	unique out.added.sku
	unique out.removed.sku
	unique out.changed.sku
	
	out.added unordered
	out.removed unordered
	out.changed unordered
	
	exactly_one Ok(items) in apply_diff(old, out): items same_elements_as new

	
	
	
	e:
		in: ([
				{ sku: "A", qty: 5 },
				{ sku: "B", qty: 2 },
				{ sku: "C", qty: 1 }
			],
			[
				{ sku: "A", qty: 5 },
				{ sku: "B", qty: 3 },
				{ sku: "D", qty: 7 }
			])
		out: {
				added:   [ { sku: "D", qty: 7 } ],
				removed: [ { sku: "C", qty: 1 } ],
				changed: [ { sku: "B", old_qty: 2, new_qty: 3 } ]
			}
		
		
	e:
		in: ([
				{ sku: "A", qty: 5 },
				{ sku: "B", qty: 2 },
				{ sku: "C", qty: 1 }
			],
			[
				{ sku: "A", qty: 5 },
				{ sku: "C", qty: 1 },
				{ sku: "B", qty: 3 },
				{ sku: "D", qty: 7 }
			])
		out: {
				added:   [ { sku: "D", qty: 7 } ],
				removed: [],
				changed: [ { sku: "B", old_qty: 2, new_qty: 3 } ]
			}
	e:
		in: ([], [])
		out: {
			added: [],
			removed: [],
			changed: []
		}
```

`p2.sur`:

```
type Order = {amount: Int, customer_email: Text}
get_small_orders(orders: Order[]) -> Order[]
	note "keep orders at amount 100 or lower"
	all o in out: o in orders
	all o in out: o.amount <= 100
	all o leftover in orders: o.amount > 100
	out same_order_as orders
	
	e:
		in: [{amount: 4, customer_email: "test@test.com"}, {amount: 42143, customer_email: "test@test.com"}, {amount: 52, customer_email: "test@test.com"}, {amount: 99, customer_email: "test@test.com"}, {amount: 100, customer_email: "test@test.com"}] 
		out: [{amount: 4, ... }, {amount: 52, ... }, {amount: 99, ... }, {amount: 100, ...}] 
	e:
		in: []
		out: []
```

`p3.sur`:

```
type Order = { amount: Int, customer_email: Text }
type OrderGroup = { customer_email: Text, orders: Order[], total_amount: Int }

group_by_customer(orders: Order[]) -> OrderGroup[]
	note "group orders by customer, sum amounts per customer"
	# For every distinct customer_email in the input, there is exactly one group with that email in the output
	all o in orders:
		exactly_one g in out:
			g.customer_email == o.customer_email
	
	# No two groups share a customer_email
	unique out.customer_email
	
	# Every order in a group has that group's customer_email
	all g in out:
		all o in g.orders:
			o.customer_email == g.customer_email
	
	# The orders across all groups, taken together, are the same elements as the input orders
	// list of lists via broadcast
	// [[order1, order2, order3], [order4], [order5]]
	// so flatten() is called
	flatten(out.orders) same_elements_as orders
	
	# Each group's total_amount is the sum of the amounts of its orders
	// compare total via sum + broadcast (gets list of all amounts)
	all g in out: g.total_amount == sum(g.orders.amount)
	
	# Every group’s email is some input order’s email
	all g in out: g.customer_email in orders.customer_email
	
	# Order of groups, and of orders within a group, is unspecified
	out unordered
	all g in out: g.orders unordered
	
	e:
		in: [
			{amount: 4, customer_email: "test@test.com"},
			{amount: 42143, customer_email: "test@test.com"},
			{amount: 52, customer_email: "jess@test.com"},
			{amount: 99, customer_email: "jess@test.com"},
			{amount: 100, customer_email: "joe@abe.com"}
		]
		out: [{
				customer_email: "test@test.com",
				orders: [
					{amount: 4, customer_email: "test@test.com"},
					{amount: 42143, customer_email: "test@test.com"}
				],
				total_amount: 42147
			}, 
			{
				customer_email: "jess@test.com",
				orders: [
					{amount: 52, customer_email: "jess@test.com"},
					{amount: 99, customer_email: "jess@test.com"}
				],
				total_amount: 151
			},
			{
				customer_email: "joe@abe.com",
				orders: [
					{amount: 100, customer_email: "joe@abe.com"},
				],
				total_amount: 100
			}]

	e:
		in: []
		out: []
```

`p4.sur`:

```
type Score = {score: Float, username: Text}

top_three_scores(scores: Score[]) -> Score[0..3]
	len(out) == min(3, len(scores))
	all s in out:
		all l leftover in scores:
			s.score > l.score or (
				s.score == l.score and s.username <= l.username
			)
	# Ties should be ascending A-Z by username
	out sorted_by (score desc, username asc)
	# Every output element is an input element
	out drawn_from scores
	e:
		in: [
			{score: 4.0, username: "Aaly"},
			{score: 4.3, username: "Ben"},
			{score: 4.3, username: "Abe"},
			{score: 4.5, username: "Charlie"}
		]
		out: [
			{score: 4.5, username: "Charlie"},
			{score: 4.3, username: "Abe"},
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
	e:
		note "four tied scores with 3 slot output"
		in: [
			{score: 4.3, username: "Dan"},
			{score: 4.3, username: "Ben"},
			{score: 4.3, username: "Cal"},
			{score: 4.3, username: "Abe"}
		]
		out: [
			{score: 4.3, username: "Abe"},
			{score: 4.3, username: "Ben"},
			{score: 4.3, username: "Cal"}
		]
```

`p5a.sur`:

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

`p5b.sur`:

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

`p6.sur`:

```
type OrderItemRow = { id: Id, amount: Int, item_id: Id }
type Order = { id: Id, customer_email: Text, items: OrderItemRow[unique id] }
type OrderLineItem = { line_id: Id, order_id: Id, customer_email: Text, amount: Int, item_id: Id }

flatten_order(order: Order) -> OrderLineItem[]
	# For every output line item, there is exactly one input order.items item with the same id
	all l_i in out:
		exactly_one i_row in order.items:
			i_row.id == l_i.line_id
	// inverse
	all i_row in order.items:
		exactly_one r in out:
			r.line_id == i_row.id
	
	# Every output item carries the input order's id as order_id and its customer_email as customer_email
	out.order_id == order.id
	out.customer_email == order.customer_email
	
	# Output rows are in the same order as the input items
	// use broadcast id comparison
	out.line_id same_order_as order.items.id
	
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
```

`p7.sur`:

```
type User = { id: Id, user_name: Text }
type Order = { 
	id: Id,
	customer_email: Text,
	amount: Int,
	user_id: Option<Id>
}
type EnrichedOrder = { ...Order, user_name: Option<Text> }

enrich_orders(orders: Order[unique id], users: User[unique id]) -> EnrichedOrder[]
	# For every output order, there is exactly one input order with the same id
	all o in out:
		exactly_one i_o in orders:
			o.id == i_o.id
			
	# For every input order, there is exactly one output order with the same id
	all i_o in orders:
		exactly_one o in out:
			i_o.id == o.id
	
	# For every output order, user_name is the name of the input user if the order's user_id matches, otherwise user_name is None.
	all o in out: all u in users where u.id == o.user_id: o.user_name == u.user_name else o.user_name == None
	
	# Output rows are in the same order as input items
	out.id same_order_as orders.id
	
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
				user_id: "u1a",
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
```

`p8.sur`:

```
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

// reach for Set type instead of pure list
valid_errors = [password_error, age_error, email_error]

valid_age(age: Int) = age >= 13 and age <= 120
valid_pass(password: Text) = len(password) >= 12

validate_signup(p: Payload) -> Result<Payload, ValidationError[]>
	# The output is `Ok` if and only if no check fails
	checks = valid_age(p.age) and well_formed_email(p.email) and valid_pass(p.password)
	checks <-> exactly_one Ok(_) in out
	
	# An `Ok` output carries the input unchanged.
	all Ok(p_o) in out: p_o == p
	
	all Err(errs) in out: email_error in errs <-> not well_formed_email(p.email)

	all Err(errs) in out: age_error in errs <-> not valid_age(p.age)

	all Err(errs) in out: password_error in errs <-> not valid_pass(p.password)
	
	all Err(errs) in out: errs drawn_from valid_errors
	
	all Err(errs) in out: errs unordered
	
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
```

Write the first stage for this problem: the types you had to add, the signature, and the sentences.