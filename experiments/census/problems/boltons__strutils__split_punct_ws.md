boltons.strutils.split_punct_ws (from boltons 26.2.0)

Signature, in words: `split_punct_ws` takes the required argument `text`.

The docstring:

While :meth:`str.split` will split on whitespace,
:func:`split_punct_ws` will split on punctuation and
whitespace. This used internally by :func:`slugify`, above.

>>> split_punct_ws('First post! Hi!!!!~1    ')
['First', 'post', 'Hi', '1']
