toolz.recipes.countby (from toolz 1.2.0)

Signature, in words: `countby` takes the required arguments `key`, `seq`.

The docstring:

Count elements of a collection by a key function

>>> countby(len, ['cat', 'mouse', 'dog'])
{3: 2, 5: 1}

>>> def iseven(x): return x % 2 == 0
>>> countby(iseven, [1, 2, 3])  # doctest:+SKIP
{True: 1, False: 2}

See Also:
    groupby
