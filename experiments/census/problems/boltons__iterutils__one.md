boltons.iterutils.one (from boltons 26.2.0)

Signature, in words: `one` takes the required argument `src`; the optional arguments `default` (default none), `key` (default none).

The docstring:

Along the same lines as builtins, :func:`all` and :func:`any`, and
similar to :func:`first`, ``one()`` returns the single object in
the given iterable *src* that evaluates to ``True``, as determined
by callable *key*. If unset, *key* defaults to :class:`bool`. If
no such objects are found, *default* is returned. If *default* is
not passed, ``None`` is returned.

If *src* has more than one object that evaluates to ``True``, or
if there is no object that fulfills such condition, return
*default*. It's like an `XOR`_ over an iterable.

>>> one((True, False, False))
True
>>> one((True, False, True))
>>> one((0, 0, 'a'))
'a'
>>> one((0, False, None))
>>> one((True, True), default=False)
False
>>> bool(one(('', 1)))
True
>>> one((10, 20, 30, 42), key=lambda i: i > 40)
42

See `Martín Gaitán's original repo`_ for further use cases.

.. _Martín Gaitán's original repo: https://github.com/mgaitan/one
.. _XOR: https://en.wikipedia.org/wiki/Exclusive_or
