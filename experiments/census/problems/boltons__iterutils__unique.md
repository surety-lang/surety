boltons.iterutils.unique (from boltons 26.2.0)

Signature, in words: `unique` takes the required argument `src`; the optional argument `key` (default none).

The docstring:

``unique()`` returns a list of unique values, as determined by
*key*, in the order they first appeared in the input iterable,
*src*.

>>> ones_n_zeros = '11010110001010010101010'
>>> ''.join(unique(ones_n_zeros))
'10'

See :func:`unique_iter` docs for more details.
