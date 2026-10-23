boltons.iterutils.first (from boltons 26.2.0)

Signature, in words: `first` takes the required argument `iterable`; the optional arguments `default` (default none), `key` (default none).

The docstring:

Return first element of *iterable* that evaluates to ``True``, else
return ``None`` or optional *default*. Similar to :func:`one`.

>>> first([0, False, None, [], (), 42])
42
>>> first([0, False, None, [], ()]) is None
True
>>> first([0, False, None, [], ()], default='ohai')
'ohai'
>>> import re
>>> m = first(re.match(regex, 'abc') for regex in ['b.*', 'a(.*)'])
>>> m.group(1)
'bc'

The optional *key* argument specifies a one-argument predicate function
like that used for *filter()*.  The *key* argument, if supplied, should be
in keyword form. For example, finding the first even number in an iterable:

>>> first([1, 1, 3, 4, 5], key=lambda x: x % 2 == 0)
4

Contributed by Hynek Schlawack, author of `the original standalone module`_.

.. _the original standalone module: https://github.com/hynek/first
