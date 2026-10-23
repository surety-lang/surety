```surety
# The output has the same elements as the input, each occurring as many times as it does in the input.
	out same_elements_as iterable
# Of two input elements with different values, the one that comes earlier in the input comes later in the output.
	reaching for: for two input occurrences a and b with different values, where a is before b in iterable, the output occurrence that stands for b is before the output occurrence that stands for a. The notation has no way to say which output occurrence stands for which input occurrence. Here `b before a in out` resolves an input element to the first occurrence of its value in out. On iterable [1, 2, 1] that makes the line false on the correct output [1, 2, 1]: the first 1 is before the 2 in the input, and the first 1 in out is before the 2 too. The pairing I wanted is the k-th occurrence of a value in the input with the k-th-from-last occurrence in the output.
# The output's first element is the input's last element.
	len(iterable) > 0 -> out[0 .. 1] == iterable[len(iterable) - 1 .. len(iterable)]
# The output after its first element is the reversal of the input without its last element.
	len(iterable) > 0 -> out[1 .. len(out)] == always_reversible(iterable[0 .. len(iterable) - 1])
```