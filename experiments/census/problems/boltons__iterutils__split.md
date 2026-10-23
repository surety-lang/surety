boltons.iterutils.split (from boltons 26.2.0)

Signature, in words: `split` takes the required argument `src`; the optional arguments `sep` (default none), `maxsplit` (default none).

The docstring:

Splits an iterable based on a separator. Like :meth:`str.split`,
but for all iterables. Returns a list of lists.

>>> split(['hi', 'hello', None, None, 'sup', None, 'soap', None])
[['hi', 'hello'], ['sup'], ['soap']]

See :func:`split_iter` docs for more info.
