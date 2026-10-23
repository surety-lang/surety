more_itertools.more.constrained_batches (from more-itertools 11.2.1)

Signature, in words: `constrained_batches` takes the required arguments `iterable`, `max_size`; the optional arguments `max_count` (default none), `get_len` (default the `len` function), `strict` (default `True`).

The docstring:

Yield batches of items from *iterable* with a combined size limited by
*max_size*.

>>> iterable = [b'12345', b'123', b'12345678', b'1', b'1', b'12', b'1']
>>> list(constrained_batches(iterable, 10))
[(b'12345', b'123'), (b'12345678', b'1', b'1'), (b'12', b'1')]

If a *max_count* is supplied, the number of items per batch is also
limited. It must be greater than zero:

>>> iterable = [b'12345', b'123', b'12345678', b'1', b'1', b'12', b'1']
>>> list(constrained_batches(iterable, 10, max_count = 2))
[(b'12345', b'123'), (b'12345678', b'1'), (b'1', b'12'), (b'1',)]

If a *get_len* function is supplied, use that instead of :func:`len` to
determine item size.

If *strict* is ``True``, raise ``ValueError`` if any single item is bigger
than *max_size*. Otherwise, allow single items to exceed *max_size*.
