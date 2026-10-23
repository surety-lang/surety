toolz.itertoolz.pluck (from toolz 1.2.0)

Signature, in words: `pluck` takes the required arguments `ind`, `seqs`; the optional argument `default` (default `'__no__default__'`).

The docstring:

plucks an element or several elements from each item in a sequence.

``pluck`` maps ``itertoolz.get`` over a sequence and returns one or more
elements of each item in the sequence.

This is equivalent to running `map(curried.get(ind), seqs)`

``ind`` can be either a single string/index or a list of strings/indices.
``seqs`` should be sequence containing sequences or dicts.

e.g.

>>> data = [{'id': 1, 'name': 'Cheese'}, {'id': 2, 'name': 'Pies'}]
>>> list(pluck('name', data))
['Cheese', 'Pies']
>>> list(pluck([0, 1], [[1, 2, 3], [4, 5, 7]]))
[(1, 2), (4, 5)]

See Also:
    get
    map
