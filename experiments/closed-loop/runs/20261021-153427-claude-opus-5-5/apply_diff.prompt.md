Obligation `apply_diff.sur`:

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

Referenced obligation `p8.sur` (for its types; its performance is supplied separately):

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

Your module must define: `apply_diff`.