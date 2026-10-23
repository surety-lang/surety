```surety
# The output is Ok exactly when the distance is at least 0.
	distance >= 0 <-> exactly_one Ok(_) in out
# Every error output is distance_error.
	all Err(e) in out: e == distance_error
# An Ok output has exactly as many elements as there are input items.
	all Ok(res) in out: len(res) == len(items)
# An Ok output's items are the input items, in the input's order.
	all Ok(res) in out: res.item == items
# In an Ok output, every element whose position is at most distance places from the position of an item that satisfies the predicate is flagged near.
	reaching for: a binder over positions of res, so the line could read: for every position i of res and every position j of items where items at j satisfies and abs(i - j) <= distance, the element of res at i is near. Occurrences carry positions, but only `before` can compare them, and nothing measures how far apart two positions are.
# In an Ok output, every element flagged near is at most distance places from the position of some item that satisfies the predicate.
	reaching for: a binder over positions of res, so the line could read: for every position i of res whose element is near, there is a position j of items where items at j satisfies and abs(i - j) <= distance. As above, the notation can say that one occurrence comes before another, but not how many places apart they are.
```