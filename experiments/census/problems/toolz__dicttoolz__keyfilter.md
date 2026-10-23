toolz.dicttoolz.keyfilter (from toolz 1.2.0)

Signature, in words: `keyfilter` takes the required arguments `predicate`, `d`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Filter items in dictionary by key

>>> iseven = lambda x: x % 2 == 0
>>> d = {1: 2, 2: 3, 3: 4, 4: 5}
>>> keyfilter(iseven, d)
{2: 3, 4: 5}

See Also:
    valfilter
    itemfilter
    keymap
