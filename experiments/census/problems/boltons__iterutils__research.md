boltons.iterutils.research (from boltons 26.2.0)

Signature, in words: `research` takes the required argument `root`; the optional arguments `query` (default the `<lambda>` function), `reraise` (default `False`), `enter` (default the `default_enter` function).

The docstring:

The :func:`research` function uses :func:`remap` to recurse over
any data nested in *root*, and find values which match a given
criterion, specified by the *query* callable.

Results are returned as a list of ``(path, value)`` pairs. The
paths are tuples in the same format accepted by
:func:`get_path`. This can be useful for comparing values nested
in two or more different structures.

Here's a simple example that finds all integers:

>>> root = {'a': {'b': 1, 'c': (2, 'd', 3)}, 'e': None}
>>> res = research(root, query=lambda p, k, v: isinstance(v, int))
>>> print(sorted(res))
[(('a', 'b'), 1), (('a', 'c', 0), 2), (('a', 'c', 2), 3)]

Note how *query* follows the same, familiar ``path, key, value``
signature as the ``visit`` and ``enter`` functions on
:func:`remap`, and returns a :class:`bool`.

Args:
   root: The target object to search. Supports the same types of
      objects as :func:`remap`, including :class:`list`,
      :class:`tuple`, :class:`dict`, and :class:`set`.
   query (callable): The function called on every object to
      determine whether to include it in the search results. The
      callable must accept three arguments, *path*, *key*, and
      *value*, commonly abbreviated *p*, *k*, and *v*, same as
      *enter* and *visit* from :func:`remap`.
   reraise (bool): Whether to reraise exceptions raised by *query*
      or to simply drop the result that caused the error.


With :func:`research` it's easy to inspect the details of a data
structure, like finding values that are at a certain depth (using
``len(p)``) and much more. If more advanced functionality is
needed, check out the code and make your own :func:`remap`
wrapper, and consider `submitting a patch`_!

.. _submitting a patch: https://github.com/mahmoud/boltons/pulls
