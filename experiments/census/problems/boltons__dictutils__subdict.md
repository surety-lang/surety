boltons.dictutils.subdict (from boltons 26.2.0)

Signature, in words: `subdict` takes the required argument `d`; the optional arguments `keep` (default none), `drop` (default none).

The docstring:

Compute the "subdictionary" of a dict, *d*.

A subdict is to a dict what a subset is a to set. If *A* is a
subdict of *B*, that means that all keys of *A* are present in
*B*.

Returns a new dict with any keys in *drop* removed, and any keys
in *keep* still present, provided they were in the original
dict. *keep* defaults to all keys, *drop* defaults to empty, so
without one of these arguments, calling this function is
equivalent to calling ``dict()``.

>>> from pprint import pprint as pp
>>> pp(subdict({'a': 1, 'b': 2}))
{'a': 1, 'b': 2}
>>> subdict({'a': 1, 'b': 2, 'c': 3}, drop=['b', 'c'])
{'a': 1}
>>> pp(subdict({'a': 1, 'b': 2, 'c': 3}, keep=['a', 'c']))
{'a': 1, 'c': 3}
