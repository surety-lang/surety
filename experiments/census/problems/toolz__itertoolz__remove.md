toolz.itertoolz.remove (from toolz 1.2.0)

Signature, in words: `remove` takes the required arguments `predicate`, `seq`.

The docstring:

Return those items of sequence for which predicate(item) is False

>>> def iseven(x):
...     return x % 2 == 0
>>> list(remove(iseven, [1, 2, 3, 4]))
[1, 3]
