more_itertools.more.zip_broadcast (from more-itertools 11.2.1)

Signature, in words: `zip_broadcast` takes any number of positional arguments `objects`; the keyword-only arguments `scalar_types` (default `(<class 'str'>, <class 'bytes'>)`), `strict` (default `False`).

The docstring:

A version of :func:`zip` that "broadcasts" any scalar
(i.e., non-iterable) items into output tuples.

>>> iterable_1 = [1, 2, 3]
>>> iterable_2 = ['a', 'b', 'c']
>>> scalar = '_'
>>> list(zip_broadcast(iterable_1, iterable_2, scalar))
[(1, 'a', '_'), (2, 'b', '_'), (3, 'c', '_')]

The *scalar_types* keyword argument determines what types are considered
scalar. It is set to ``(str, bytes)`` by default. Set it to ``None`` to
treat strings and byte strings as iterable:

>>> list(zip_broadcast('abc', 0, 'xyz', scalar_types=None))
[('a', 0, 'x'), ('b', 0, 'y'), ('c', 0, 'z')]

If the *strict* keyword argument is ``True``, then
``ValueError`` will be raised if any of the iterables have
different lengths.
