boltons.iterutils.untyped_sorted (from boltons 26.2.0)

Signature, in words: `untyped_sorted` takes the required argument `iterable`; the optional arguments `key` (default none), `reverse` (default `False`).

The docstring:

A version of :func:`sorted` which will happily sort an iterable of
heterogeneous types and return a new list, similar to legacy Python's
behavior.

>>> untyped_sorted(['abc', 2.0, 1, 2, 'def'])
[1, 2.0, 2, 'abc', 'def']

Note how mutually orderable types are sorted as expected, as in
the case of the integers and floats above.

.. note::

   Results may vary across Python versions and builds, but the
   function will produce a sorted list, except in the case of
   explicitly unorderable objects.
