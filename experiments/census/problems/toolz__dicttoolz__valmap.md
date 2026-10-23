toolz.dicttoolz.valmap (from toolz 1.2.0)

Signature, in words: `valmap` takes the required arguments `func`, `d`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Apply function to values of dictionary

>>> bills = {"Alice": [20, 15, 30], "Bob": [10, 35]}
>>> valmap(sum, bills)  # doctest: +SKIP
{'Alice': 65, 'Bob': 45}

See Also:
    keymap
    itemmap
