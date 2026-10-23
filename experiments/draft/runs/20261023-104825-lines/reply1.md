```surety
# The output is Ok exactly when every item on which the validator raises has some class of its exception among the exceptions.
	(exactly_one Ok(_) in out) <-> all i in items where len(i.raises) > 0: any c in i.raises: c in exceptions
# Every element of an Ok output is an item on which the validator returns, with an empty raises.
	all Ok(res) in out: all r in res: len(r.raises) == 0
# An Ok output is drawn from the items.
	all Ok(res) in out: res drawn_from items
# Every item left out of an Ok output is one on which the validator raises an exception with some class among the exceptions.
	all Ok(res) in out: all i leftover in items: any c in i.raises: c in exceptions
# An Ok output keeps the items' order.
	all Ok(res) in out: res same_order_as items
# An Err output carries an input item on which the validator raises an exception with no class among the exceptions.
	all Err(e) in out: e in items and len(e.raises) > 0 and none c in e.raises: c in exceptions
# Every item before the Err item in the input either has an empty raises or has some class of its exception among the exceptions.
	all Err(e) in out: all i in items where i before e in items: len(i.raises) == 0 or any c in i.raises: c in exceptions
```