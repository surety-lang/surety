boltons.iterutils.redundant (from boltons 26.2.0)

Signature, in words: `redundant` takes the required argument `src`; the optional arguments `key` (default none), `groups` (default `False`).

The docstring:

The complement of :func:`unique()`.

By default returns non-unique/duplicate values as a list of the
*first* redundant value in *src*. Pass ``groups=True`` to get
groups of all values with redundancies, ordered by position of the
first redundant value. This is useful in conjunction with some
normalizing *key* function.

>>> redundant([1, 2, 3, 4])
[]
>>> redundant([1, 2, 3, 2, 3, 3, 4])
[2, 3]
>>> redundant([1, 2, 3, 2, 3, 3, 4], groups=True)
[[2, 2], [3, 3, 3]]

An example using a *key* function to do case-insensitive
redundancy detection.

>>> redundant(['hi', 'Hi', 'HI', 'hello'], key=str.lower)
['Hi']
>>> redundant(['hi', 'Hi', 'HI', 'hello'], groups=True, key=str.lower)
[['hi', 'Hi', 'HI']]

*key* should also be used when the values in *src* are not hashable.

.. note::

   This output of this function is designed for reporting
   duplicates in contexts when a unique input is desired. Due to
   the grouped return type, there is no streaming equivalent of
   this function for the time being.
