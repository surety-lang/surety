```surety
# The output is Ok exactly when func accepts every item's keys as its keyword arguments (a sibling obligation decides acceptance).
	reaching for: `exactly_one Ok(_) in out` iff `all k in iterable: func_accepts(k.entries.key)`, where `func_accepts` is a bodiless graph reference to the sibling acceptance obligation. Both sides of the iff need a quantifier, but a quantifier must be the last operand on a line. Folding "every item" into a predicate over the whole list would move this sentence's claim into the sibling, and an in-obligation abbreviation would be a second line.
# Every Err output is the type error.
	all Err(e) in out: e == type_error
# An Ok output has as many elements as the iterable has items.
	all Ok(res) in out: len(res) == len(iterable)
# Each element of an Ok output is func's result on the item at the same position in the iterable, so the output keeps the iterable's order.
	reaching for: for every position i, element i of res equals func applied to item i of iterable, where func is a sibling obligation called as a graph reference returning Int. That is a positional zip of res with iterable mapped through func. Binders range over occurrences but cannot name an index to pair the two lists, and the notation has no map of a sibling function over a list to compare against res.
```