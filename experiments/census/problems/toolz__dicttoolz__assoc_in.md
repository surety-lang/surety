toolz.dicttoolz.assoc_in (from toolz 1.2.0)

Signature, in words: `assoc_in` takes the required arguments `d`, `keys`, `value`; the optional argument `factory` (default the `dict` built-in).

The docstring:

Return a new dict with new, potentially nested, key value pair

>>> purchase = {'name': 'Alice',
...             'order': {'items': ['Apple', 'Orange'],
...                       'costs': [0.50, 1.25]},
...             'credit card': '5555-1234-1234-1234'}
>>> assoc_in(purchase, ['order', 'costs'], [0.25, 1.00]) # doctest: +SKIP
{'credit card': '5555-1234-1234-1234',
 'name': 'Alice',
 'order': {'costs': [0.25, 1.00], 'items': ['Apple', 'Orange']}}
