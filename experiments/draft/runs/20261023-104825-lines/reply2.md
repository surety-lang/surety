```surety
# The output is Ok exactly when every item on which the validator raises has some class of its exception among the exceptions.
	reaching for: an iff whose two sides are both quantified, out is Ok exactly when every item with a nonempty raises has some r in its raises with r in exceptions. As one line, one quantifier has to sit mid-line on one side of the `<->`. The corpus form is an abbreviation `all_caught = all i in items where len(i.raises) > 0: any r in i.raises: r in exceptions` followed by `all_caught <-> exactly_one Ok(_) in out`, which is two lines.

# Every element of an Ok output is an item on which the validator returns, with an empty raises.
	all Ok(res) in out: all x in res: x in items and len(x.raises) == 0

# An Ok output is drawn from the items.
	all Ok(res) in out: res drawn_from items

# Every item left out of an Ok output is one on which the validator raises an exception with some class among the exceptions.
	all Ok(res) in out: all i in items where i not in res: any r in i.raises: r in exceptions

# An Ok output keeps the items' order.
	all Ok(res) in out: res same_order_as items

# An Err output carries an input item on which the validator raises an exception with no class among the exceptions.
	all Err(x) in out: x in items and len(x.raises) > 0 and none r in x.raises: r in exceptions

# Every item before the Err item in the input either has an empty raises or has some class of its exception among the exceptions.
	all Err(x) in out: all i in items where i before x in items: len(i.raises) == 0 or any r in i.raises: r in exceptions
```