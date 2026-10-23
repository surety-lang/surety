toolz.itertoolz.get (from toolz 1.2.0)

Signature, in words: `get` takes the required arguments `ind`, `seq`; the optional argument `default` (default `'__no__default__'`).

The docstring:

Get element in a sequence or dict

Provides standard indexing

>>> get(1, 'ABC')       # Same as 'ABC'[1]
'B'

Pass a list to get multiple values

>>> get([1, 2], 'ABC')  # ('ABC'[1], 'ABC'[2])
('B', 'C')

Works on any value that supports indexing/getitem
For example here we see that it works with dictionaries

>>> phonebook = {'Alice':  '555-1234',
...              'Bob':    '555-5678',
...              'Charlie':'555-9999'}
>>> get('Alice', phonebook)
'555-1234'

>>> get(['Alice', 'Bob'], phonebook)
('555-1234', '555-5678')

Provide a default for missing values

>>> get(['Alice', 'Dennis'], phonebook, None)
('555-1234', None)

See Also:
    pluck
