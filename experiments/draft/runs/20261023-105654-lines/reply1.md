```surety
# Every output tuple has exactly as many values as there are pools.
	all t in out: len(t.values) == len(iterables) * max(repeat, 0)
# Each value in an output tuple is one of the values of the pool at that position.
	reaching for: for every tuple t and every position p in t.values, the value at position p is in the values of the pool at position p, where the pool list is iterables repeated max(repeat, 0) times, i.e. iterables[p mod len(iterables)]; the notation has no way to bind an element's position as an Int, no list repetition, and no %
# The output is empty exactly when some pool is empty.
	len(out) == 0 <-> repeat > 0 and any p in iterables: len(p.values) == 0
# When no pool is empty, the first tuple holds the first value of every pool.
	reaching for: when no pool is empty, out[0].values equals the list of each pool's values[0], taken over iterables repeated max(repeat, 0) times; the notation has no list repetition and no way to build the list of first elements of a list of lists (no map over a slice per element)
# Each tuple after the first is the tuple before it with one position set to that pool's next value and every other position kept.
	reaching for: for every adjacent pair out[i], out[i+1] there is exactly one position p where they may differ, out[i+1].values[p] is the element after out[i].values[p] in the values of the pool at position p (iterables repeated max(repeat, 0) times), and every other position is equal; the notation cannot range over adjacent pairs, index a list at a bound Int position, or say "the next element after this occurrence"
# The position that advances never moves back to an earlier pool from one tuple to the next.
	reaching for: writing a(i) for the position advanced between out[i] and out[i+1], a(i) <= a(i+1) for every i; the notation cannot name the advancing position as an Int or range over consecutive pairs of tuples
# Every value of every pool appears at that pool's position in some output tuple.
	reaching for: for every pool position p (positions of iterables repeated max(repeat, 0) times) and every value v in that pool's values, some tuple t in out has t.values[p] == v; the notation cannot quantify over positions as Ints, index t.values at p, or repeat a list
```