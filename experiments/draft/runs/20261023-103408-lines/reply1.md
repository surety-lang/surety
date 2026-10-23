```surety
# When n is at least 1, the slices joined in order are the elements of seq from position 0 up to but not including the number of slices times n.
	n >= 1 -> flatten(out.slices) == seq[0 .. len(out.slices) * n]
# When n is at least 1, every slice has at least one element and at most n elements.
	n >= 1 -> all s in out.slices: len(s) >= 1 and len(s) <= n
# When n is at least 1, a slice with fewer than n elements is the last slice.
	n >= 1 -> all s in out.slices[0 .. len(out.slices) - 1]: len(s) >= n
# When n is at least 1 and strict is true, every slice has exactly n elements.
	n >= 1 and strict == true -> all s in out.slices: len(s) == n
# When n is at least 1, the length of seq is less than the number of slices plus one, times n, so no full slice is left unyielded.
	n >= 1 -> len(seq) < (len(out.slices) + 1) * n
# When n is at least 1 and there is no error, the length of seq is at most the number of slices times n, so the slices cover the whole of seq.
	n >= 1 and out.error == None -> len(seq) <= len(out.slices) * n
# When n is at least 1, an error is reported exactly when strict is true and the length of seq is not equal to the number of slices times n.
	n >= 1 -> (out.error != None <-> (strict == true and len(seq) != len(out.slices) * n))
# Every reported error is not_divisible_error.
	out.error != None -> out.error == not_divisible_error
```