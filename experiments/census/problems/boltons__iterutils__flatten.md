boltons.iterutils.flatten (from boltons 26.2.0)

Signature, in words: `flatten` takes the required argument `iterable`.

The docstring:

``flatten()`` returns a collapsed list of all the elements from
*iterable* while collapsing any nested iterables.

>>> nested = [[1, 2], [[3], [4, 5]]]
>>> flatten(nested)
[1, 2, 3, 4, 5]
