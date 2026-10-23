more_itertools.more.is_sorted (from more-itertools 11.2.1)

Signature, in words: `is_sorted` takes the required argument `iterable`; the optional arguments `key` (default none), `reverse` (default `False`), `strict` (default `False`).

The docstring:

Returns ``True`` if the items of iterable are in sorted order, and
``False`` otherwise. *key* and *reverse* have the same meaning that they do
in the built-in :func:`sorted` function.

>>> is_sorted(['1', '2', '3', '4', '5'], key=int)
True
>>> is_sorted([5, 4, 3, 1, 2], reverse=True)
False

If *strict*, tests for strict sorting, that is, returns ``False`` if equal
elements are found:

>>> is_sorted([1, 2, 2])
True
>>> is_sorted([1, 2, 2], strict=True)
False

The function returns ``False`` after encountering the first out-of-order
item, which means it may produce results that differ from the built-in
:func:`sorted` function for objects with unusual comparison dynamics
(like ``math.nan``). If there are no out-of-order items, the iterable is
exhausted.
