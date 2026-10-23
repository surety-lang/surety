toolz.dicttoolz.itemfilter (from toolz 1.2.0)

Signature, in words: `itemfilter` takes the required arguments `predicate`, `d`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Filter items in dictionary by item

>>> def isvalid(item):
...     k, v = item
...     return k % 2 == 0 and v < 4

>>> d = {1: 2, 2: 3, 3: 4, 4: 5}
>>> itemfilter(isvalid, d)
{2: 3}

See Also:
    keyfilter
    valfilter
    itemmap
