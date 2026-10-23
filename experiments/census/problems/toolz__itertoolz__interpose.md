toolz.itertoolz.interpose (from toolz 1.2.0)

Signature, in words: `interpose` takes the required arguments `el`, `seq`.

The docstring:

Introduce element between each pair of elements in seq

>>> list(interpose("a", [1, 2, 3]))
[1, 'a', 2, 'a', 3]
>>> list(interpose("a", []))
[]
