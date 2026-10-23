toolz.itertoolz.concatv (from toolz 1.2.0)

Signature, in words: `concatv` takes any number of positional arguments `seqs`.

The docstring:

Variadic version of concat

>>> list(concatv([], ["a"], ["b", "c"]))
['a', 'b', 'c']

See also:
    itertools.chain
