toolz.dicttoolz.merge (from toolz 1.2.0)

Signature, in words: `merge` takes any number of positional arguments `dicts`; any keyword arguments `kwargs`.

The docstring:

Merge a collection of dictionaries

>>> merge({1: 'one'}, {2: 'two'})
{1: 'one', 2: 'two'}

Later dictionaries have precedence

>>> merge({1: 2, 3: 4}, {3: 3, 4: 4})
{1: 2, 3: 3, 4: 4}

See Also:
    merge_with
