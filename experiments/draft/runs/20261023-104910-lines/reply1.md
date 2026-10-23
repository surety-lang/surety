```surety
# If reverse is false and strict is false, the result is true exactly when no item comes before another item with a smaller key.
	(not reverse and not strict) -> out == none a in items: any b in items where a before b in items: b.key < a.key
# If reverse is false and strict is true, the result is true exactly when no item comes before another item with an equal or smaller key.
	(not reverse and strict) -> out == none a in items: any b in items where a before b in items: b.key <= a.key
# If reverse is true and strict is false, the result is true exactly when no item comes before another item with a larger key.
	(reverse and not strict) -> out == none a in items: any b in items where a before b in items: b.key > a.key
# If reverse is true and strict is true, the result is true exactly when no item comes before another item with an equal or larger key.
	(reverse and strict) -> out == none a in items: any b in items where a before b in items: b.key >= a.key
```