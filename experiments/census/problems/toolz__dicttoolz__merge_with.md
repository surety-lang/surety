toolz.dicttoolz.merge_with (from toolz 1.2.0)

Signature, in words: `merge_with` takes the required argument `func`; any number of positional arguments `dicts`; any keyword arguments `kwargs`.

The docstring:

Merge dictionaries and apply function to combined values

A key may occur in more than one dict, and all values mapped from the key
will be passed to the function as a list, such as func([val1, val2, ...]).

>>> merge_with(sum, {1: 1, 2: 2}, {1: 10, 2: 20})
{1: 11, 2: 22}

>>> merge_with(first, {1: 1, 2: 2}, {2: 20, 3: 30})  # doctest: +SKIP
{1: 1, 2: 2, 3: 30}

See Also:
    merge
