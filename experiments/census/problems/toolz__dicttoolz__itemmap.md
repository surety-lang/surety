toolz.dicttoolz.itemmap (from toolz 1.2.0)

Signature, in words: `itemmap` takes the required arguments `func`, `d`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Apply function to items of dictionary

>>> accountids = {"Alice": 10, "Bob": 20}
>>> itemmap(reversed, accountids)  # doctest: +SKIP
{10: "Alice", 20: "Bob"}

See Also:
    keymap
    valmap
