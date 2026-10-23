toolz.dicttoolz.keymap (from toolz 1.2.0)

Signature, in words: `keymap` takes the required arguments `func`, `d`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Apply function to keys of dictionary

>>> bills = {"Alice": [20, 15, 30], "Bob": [10, 35]}
>>> keymap(str.lower, bills)  # doctest: +SKIP
{'alice': [20, 15, 30], 'bob': [10, 35]}

See Also:
    valmap
    itemmap
