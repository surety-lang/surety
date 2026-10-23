boltons.strutils.iter_splitlines (from boltons 26.2.0)

Signature, in words: `iter_splitlines` takes the required argument `text`.

The docstring:

Like :meth:`str.splitlines`, but returns an iterator of lines
instead of a list. Also similar to :meth:`file.next`, as that also
lazily reads and yields lines from a file.

This function works with a variety of line endings, but as always,
be careful when mixing line endings within a file.

>>> list(iter_splitlines('\nhi\nbye\n'))
['', 'hi', 'bye', '']
>>> list(iter_splitlines('\r\nhi\rbye\r\n'))
['', 'hi', 'bye', '']
>>> list(iter_splitlines(''))
[]
