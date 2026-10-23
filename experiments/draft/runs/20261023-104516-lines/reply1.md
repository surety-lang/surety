```surety
# Every output index is the start of some window on which the predicate returned true.
	all i in out: any w in windows where w.matched: w.start == i
# Every window on which the predicate returned true has its start in the output.
	all w in windows where w.matched: w.start in out
# No index appears twice in the output.
	unique out
# The output is sorted from the largest index to the smallest.
	all a in out: all b in out where a before b in out: a >= b
```