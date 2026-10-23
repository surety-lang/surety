toolz.dicttoolz.valfilter (from toolz 1.2.0)

Signature, in words: `valfilter` takes the required arguments `predicate`, `d`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Filter items in dictionary by value

>>> iseven = lambda x: x % 2 == 0
>>> d = {1: 2, 2: 3, 3: 4, 4: 5}
>>> valfilter(iseven, d)
{1: 2, 3: 4}

See Also:
    keyfilter
    itemfilter
    valmap
