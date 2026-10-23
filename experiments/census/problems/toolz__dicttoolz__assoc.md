toolz.dicttoolz.assoc (from toolz 1.2.0)

Signature, in words: `assoc` takes the required arguments `d`, `key`, `value`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Return a new dict with new key value pair

New dict has d[key] set to value. Does not modify the initial dictionary.

>>> assoc({'x': 1}, 'x', 2)
{'x': 2}
>>> assoc({'x': 1}, 'y', 3)   # doctest: +SKIP
{'x': 1, 'y': 3}
