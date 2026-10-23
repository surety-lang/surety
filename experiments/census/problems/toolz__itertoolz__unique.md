toolz.itertoolz.unique (from toolz 1.2.0)

Signature, in words: `unique` takes the required argument `seq`; the optional argument `key` (default none).

The docstring:

Return only unique elements of a sequence

>>> tuple(unique((1, 2, 3)))
(1, 2, 3)
>>> tuple(unique((1, 2, 1, 3)))
(1, 2, 3)

Uniqueness can be defined by key keyword

>>> tuple(unique(['cat', 'mouse', 'dog', 'hen'], key=len))
('cat', 'mouse')
