toolz.dicttoolz.dissoc (from toolz 1.2.0)

Signature, in words: `dissoc` takes the required argument `d`; any number of positional arguments `keys`; any keyword arguments `kwargs`.

The docstring:

Return a new dict with the given key(s) removed.

New dict has d[key] deleted for each supplied key.
Does not modify the initial dictionary.

>>> dissoc({'x': 1, 'y': 2}, 'y')
{'x': 1}
>>> dissoc({'x': 1, 'y': 2}, 'y', 'x')
{}
>>> dissoc({'x': 1}, 'y') # Ignores missing keys
{'x': 1}
