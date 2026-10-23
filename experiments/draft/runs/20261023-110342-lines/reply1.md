```surety
# The output is true exactly when every element's key equals every other element's key.
	out <-> all a in iterable: all b in iterable: a.key == b.key
```