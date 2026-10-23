more_itertools.more.substrings_indexes (from more-itertools 11.2.1)

Signature, in words: `substrings_indexes` takes the required argument `seq`; the optional argument `reverse` (default `False`).

The docstring:

Yield all substrings and their positions in *seq*

The items yielded will be a tuple of the form ``(substr, i, j)``, where
``substr == seq[i:j]``.

This function only works for iterables that support slicing, such as
``str`` objects.

>>> for item in substrings_indexes('more'):
...    print(item)
('m', 0, 1)
('o', 1, 2)
('r', 2, 3)
('e', 3, 4)
('mo', 0, 2)
('or', 1, 3)
('re', 2, 4)
('mor', 0, 3)
('ore', 1, 4)
('more', 0, 4)

Set *reverse* to ``True`` to yield the same items in the opposite order.
