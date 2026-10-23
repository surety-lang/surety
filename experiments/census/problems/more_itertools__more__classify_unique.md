more_itertools.more.classify_unique (from more-itertools 11.2.1)

Signature, in words: `classify_unique` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Classify each element in terms of its uniqueness.

For each element in the input iterable, return a 3-tuple consisting of:

1. The element itself
2. ``False`` if the element is equal to the one preceding it in the input,
   ``True`` otherwise (i.e. the equivalent of :func:`unique_justseen`)
3. ``False`` if this element has been seen anywhere in the input before,
   ``True`` otherwise (i.e. the equivalent of :func:`unique_everseen`)

>>> list(classify_unique('otto'))    # doctest: +NORMALIZE_WHITESPACE
[('o', True,  True),
 ('t', True,  True),
 ('t', False, False),
 ('o', True,  False)]

This function is analogous to :func:`unique_everseen` and is subject to
the same performance considerations.
