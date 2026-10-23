toolz.itertoolz.concat (from toolz 1.2.0)

Signature, in words: `concat` takes the required argument `seqs`.

The docstring:

Concatenate zero or more iterables, any of which may be infinite.

An infinite sequence will prevent the rest of the arguments from
being included.

We use chain.from_iterable rather than ``chain(*seqs)`` so that seqs
can be a generator.

>>> list(concat([[], [1], [2, 3]]))
[1, 2, 3]

See also:
    itertools.chain.from_iterable  equivalent
