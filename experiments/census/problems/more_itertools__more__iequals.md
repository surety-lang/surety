more_itertools.more.iequals (from more-itertools 11.2.1)

Signature, in words: `iequals` takes any number of positional arguments `iterables`.

The docstring:

Return ``True`` if all given *iterables* are equal to each other,
which means that they contain the same elements in the same order.

The function is useful for comparing iterables of different data types
or iterables that do not support equality checks.

>>> iequals("abc", ['a', 'b', 'c'], ('a', 'b', 'c'), iter("abc"))
True

>>> iequals("abc", "acb")
False

Not to be confused with :func:`all_equal`, which checks whether all
elements of iterable are equal to each other.
