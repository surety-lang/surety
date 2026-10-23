toolz.itertoolz.join (from toolz 1.2.0)

Signature, in words: `join` takes the required arguments `leftkey`, `leftseq`, `rightkey`, `rightseq`; the optional arguments `left_default` (default `'__no__default__'`), `right_default` (default `'__no__default__'`).

The docstring:

Join two sequences on common attributes

This is a semi-streaming operation.  The LEFT sequence is fully evaluated
and placed into memory.  The RIGHT sequence is evaluated lazily and so can
be arbitrarily large.

(Note: If right_default is defined, then unique keys of rightseq
    will also be stored in memory.)

>>> friends = [('Alice', 'Edith'),
...            ('Alice', 'Zhao'),
...            ('Edith', 'Alice'),
...            ('Zhao', 'Alice'),
...            ('Zhao', 'Edith')]

>>> cities = [('Alice', 'NYC'),
...           ('Alice', 'Chicago'),
...           ('Dan', 'Sydney'),
...           ('Edith', 'Paris'),
...           ('Edith', 'Berlin'),
...           ('Zhao', 'Shanghai')]

>>> # Vacation opportunities
>>> # In what cities do people have friends?
>>> result = join(second, friends,
...               first, cities)
>>> for ((a, b), (c, d)) in sorted(unique(result)):
...     print((a, d))
('Alice', 'Berlin')
('Alice', 'Paris')
('Alice', 'Shanghai')
('Edith', 'Chicago')
('Edith', 'NYC')
('Zhao', 'Chicago')
('Zhao', 'NYC')
('Zhao', 'Berlin')
('Zhao', 'Paris')

Specify outer joins with keyword arguments ``left_default`` and/or
``right_default``.  Here is a full outer join in which unmatched elements
are paired with None.

>>> identity = lambda x: x
>>> list(join(identity, [1, 2, 3],
...           identity, [2, 3, 4],
...           left_default=None, right_default=None))
[(2, 2), (3, 3), (None, 4), (1, None)]

Usually the key arguments are callables to be applied to the sequences.  If
the keys are not obviously callable then it is assumed that indexing was
intended, e.g. the following is a legal change.
The join is implemented as a hash join and the keys of leftseq must be
hashable. Additionally, if right_default is defined, then keys of rightseq
must also be hashable.

>>> # result = join(second, friends, first, cities)
>>> result = join(1, friends, 0, cities)  # doctest: +SKIP
