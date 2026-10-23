boltons.iterutils.get_path (from boltons 26.2.0)

Signature, in words: `get_path` takes the required arguments `root`, `path`; the optional argument `default` (no default given: a sentinel meaning 'not supplied').

The docstring:

Retrieve a value from a nested object via a tuple representing the
lookup path.

>>> root = {'a': {'b': {'c': [[1], [2], [3]]}}}
>>> get_path(root, ('a', 'b', 'c', 2, 0))
3

The path tuple format is intentionally consistent with that of
:func:`remap`, but a single dotted string can also be passed.

One of get_path's chief aims is improved error messaging. EAFP is
great, but the error messages are not.

For instance, ``root['a']['b']['c'][2][1]`` gives back
``IndexError: list index out of range``

What went out of range where? get_path currently raises
``PathAccessError: could not access 2 from path ('a', 'b', 'c', 2,
1), got error: IndexError('list index out of range',)``, a
subclass of IndexError and KeyError.

You can also pass a default that covers the entire operation,
should the lookup fail at any level.

Args:
   root: The target nesting of dictionaries, lists, or other
      objects supporting ``__getitem__``.
   path (tuple): A sequence of strings and integers to be successively
      looked up within *root*. A dot-separated (``a.b``) string may 
      also be passed.
   default: The value to be returned should any
      ``PathAccessError`` exceptions be raised.
