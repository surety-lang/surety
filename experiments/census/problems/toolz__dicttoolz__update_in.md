toolz.dicttoolz.update_in (from toolz 1.2.0)

Signature, in words: `update_in` takes the required arguments `d`, `keys`, `func`; the optional arguments `default` (default none), `factory` (default the `dict` built-in).

The docstring:

Update value in a (potentially) nested dictionary

inputs:
d - dictionary on which to operate
keys - list or tuple giving the location of the value to be changed in d
func - function to operate on that value

If keys == [k0,..,kX] and d[k0]..[kX] == v, update_in returns a copy of the
original dictionary with v replaced by func(v), but does not mutate the
original dictionary.

If k0 is not a key in d, update_in creates nested dictionaries to the depth
specified by the keys, with the innermost value set to func(default).

>>> inc = lambda x: x + 1
>>> update_in({'a': 0}, ['a'], inc)
{'a': 1}

>>> transaction = {'name': 'Alice',
...                'purchase': {'items': ['Apple', 'Orange'],
...                             'costs': [0.50, 1.25]},
...                'credit card': '5555-1234-1234-1234'}
>>> update_in(transaction, ['purchase', 'costs'], sum) # doctest: +SKIP
{'credit card': '5555-1234-1234-1234',
 'name': 'Alice',
 'purchase': {'costs': 1.75, 'items': ['Apple', 'Orange']}}

>>> # updating a value when k0 is not in d
>>> update_in({}, [1, 2, 3], str, default="bar")
{1: {2: {3: 'bar'}}}
>>> update_in({1: 'foo'}, [2, 3, 4], inc, 0)
{1: 'foo', 2: {3: {4: 1}}}
