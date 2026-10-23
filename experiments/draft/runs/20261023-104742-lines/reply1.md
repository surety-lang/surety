```surety
# If the output is true, every two given sequences have the same items in the same order.
	out -> all a in iterables: all b in iterables: a.items == b.items
# If every two given sequences have the same items in the same order, the output is true.
	not out -> any a in iterables: any b in iterables: a.items != b.items
```