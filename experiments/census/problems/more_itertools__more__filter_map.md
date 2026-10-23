more_itertools.more.filter_map (from more-itertools 11.2.1)

Signature, in words: `filter_map` takes the required arguments `func`, `iterable`.

The docstring:

Apply *func* to every element of *iterable*, yielding only those which
are not ``None``.

>>> elems = ['1', 'a', '2', 'b', '3']
>>> list(filter_map(lambda s: int(s) if s.isnumeric() else None, elems))
[1, 2, 3]
