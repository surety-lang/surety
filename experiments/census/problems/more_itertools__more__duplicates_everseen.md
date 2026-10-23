more_itertools.more.duplicates_everseen (from more-itertools 11.2.1)

Signature, in words: `duplicates_everseen` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Yield duplicate elements after their first appearance.

>>> list(duplicates_everseen('mississippi'))
['s', 'i', 's', 's', 'i', 'p', 'i']
>>> list(duplicates_everseen('AaaBbbCccAaa', str.lower))
['a', 'a', 'b', 'b', 'c', 'c', 'A', 'a', 'a']

This function is analogous to :func:`unique_everseen` and is subject to
the same performance considerations.

If you would like each duplicate to only appear once, much like ``uniq -d``
in the Unix shell or ``Itertools::duplicates`` from the Rust ``itertools``
crate, pass the return value of this function into :func:`unique_everseen`
with the same ``key``.
