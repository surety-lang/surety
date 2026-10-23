boltons.iterutils.same (from boltons 26.2.0)

Signature, in words: `same` takes the required argument `iterable`; the optional argument `ref` (no default given: a sentinel meaning 'not supplied').

The docstring:

``same()`` returns ``True`` when all values in *iterable* are
equal to one another, or optionally a reference value,
*ref*. Similar to :func:`all` and :func:`any` in that it evaluates
an iterable and returns a :class:`bool`. ``same()`` returns
``True`` for empty iterables.

>>> same([])
True
>>> same([1])
True
>>> same(['a', 'a', 'a'])
True
>>> same(range(20))
False
>>> same([[], []])
True
>>> same([[], []], ref='test')
False
