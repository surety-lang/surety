toolz.itertoolz.reduceby (from toolz 1.2.0)

Signature, in words: `reduceby` takes the required arguments `key`, `binop`, `seq`; the optional argument `init` (default `'__no__default__'`).

The docstring:

Perform a simultaneous groupby and reduction

The computation:

>>> result = reduceby(key, binop, seq, init)      # doctest: +SKIP

is equivalent to the following:

>>> def reduction(group):                           # doctest: +SKIP
...     return reduce(binop, group, init)           # doctest: +SKIP

>>> groups = groupby(key, seq)                    # doctest: +SKIP
>>> result = valmap(reduction, groups)              # doctest: +SKIP

But the former does not build the intermediate groups, allowing it to
operate in much less space.  This makes it suitable for larger datasets
that do not fit comfortably in memory

The ``init`` keyword argument is the default initialization of the
reduction.  This can be either a constant value like ``0`` or a callable
like ``lambda : 0`` as might be used in ``defaultdict``.

Simple Examples
---------------

>>> from operator import add, mul
>>> iseven = lambda x: x % 2 == 0

>>> data = [1, 2, 3, 4, 5]

>>> reduceby(iseven, add, data)  # doctest: +SKIP
{False: 9, True: 6}

>>> reduceby(iseven, mul, data)  # doctest: +SKIP
{False: 15, True: 8}

Complex Example
---------------

>>> projects = [{'name': 'build roads', 'state': 'CA', 'cost': 1000000},
...             {'name': 'fight crime', 'state': 'IL', 'cost': 100000},
...             {'name': 'help farmers', 'state': 'IL', 'cost': 2000000},
...             {'name': 'help farmers', 'state': 'CA', 'cost': 200000}]

>>> reduceby('state',                        # doctest: +SKIP
...          lambda acc, x: acc + x['cost'],
...          projects, 0)
{'CA': 1200000, 'IL': 2100000}

Example Using ``init``
----------------------

>>> def set_add(s, i):
...     s.add(i)
...     return s

>>> reduceby(iseven, set_add, [1, 2, 3, 4, 1, 2, 3], set)  # doctest: +SKIP
{True:  set([2, 4]),
 False: set([1, 3])}
