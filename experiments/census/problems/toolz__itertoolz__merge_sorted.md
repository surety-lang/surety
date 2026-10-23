toolz.itertoolz.merge_sorted (from toolz 1.2.0)

Signature, in words: `merge_sorted` takes any number of positional arguments `seqs`; any keyword arguments `kwargs`.

The docstring:

Merge and sort a collection of sorted collections

This works lazily and only keeps one value from each iterable in memory.

>>> list(merge_sorted([1, 3, 5], [2, 4, 6]))
[1, 2, 3, 4, 5, 6]

>>> ''.join(merge_sorted('abc', 'abc', 'abc'))
'aaabbbccc'

The "key" function used to sort the input may be passed as a keyword.

>>> list(merge_sorted([2, 3], [1, 3], key=lambda x: x // 3))
[2, 1, 3, 3]
