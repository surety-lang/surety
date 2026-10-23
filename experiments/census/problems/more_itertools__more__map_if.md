more_itertools.more.map_if (from more-itertools 11.2.1)

Signature, in words: `map_if` takes the required arguments `iterable`, `pred`, `func`; the optional argument `func_else` (default none).

The docstring:

Evaluate each item from *iterable* using *pred*. If the result is
equivalent to ``True``, transform the item with *func* and yield it.
Otherwise, transform the item with *func_else* and yield it.

*pred*, *func*, and *func_else* should each be functions that accept
one argument. By default, *func_else* is the identity function.

>>> from math import sqrt
>>> iterable = list(range(-5, 5))
>>> iterable
[-5, -4, -3, -2, -1, 0, 1, 2, 3, 4]
>>> list(map_if(iterable, lambda x: x > 3, lambda x: 'toobig'))
[-5, -4, -3, -2, -1, 0, 1, 2, 3, 'toobig']
>>> list(map_if(iterable, lambda x: x >= 0,
... lambda x: f'{sqrt(x):.2f}', lambda x: None))
[None, None, None, None, None, '0.00', '1.00', '1.41', '1.73', '2.00']
