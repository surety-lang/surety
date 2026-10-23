toolz.itertoolz.mapcat (from toolz 1.2.0)

Signature, in words: `mapcat` takes the required arguments `func`, `seqs`.

The docstring:

Apply func to each sequence in seqs, concatenating results.

>>> list(mapcat(lambda s: [c.upper() for c in s],
...             [["a", "b"], ["c", "d", "e"]]))
['A', 'B', 'C', 'D', 'E']
