toolz.itertoolz.interleave (from toolz 1.2.0)

Signature, in words: `interleave` takes the required argument `seqs`.

The docstring:

Interleave a sequence of sequences

>>> list(interleave([[1, 2], [3, 4]]))
[1, 3, 2, 4]

>>> ''.join(interleave(('ABC', 'XY')))
'AXBYC'

Both the individual sequences and the sequence of sequences may be infinite

Returns a lazy iterator
