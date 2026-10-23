```surety
# The head is the iterable's items from position 0 up to but not including position n.
	out.head == iterable[0 .. n]

# The rest is the iterable itself: the same items, in the same order, none left out and none added.
	out.rest == iterable
```