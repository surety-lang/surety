boltons.iterutils.soft_sorted (from boltons 26.2.0)

Signature, in words: `soft_sorted` takes the required argument `iterable`; the optional arguments `first` (default none), `last` (default none), `key` (default none), `reverse` (default `False`).

The docstring:

For when you care about the order of some elements, but not about
others.

Use this to float to the top and/or sink to the bottom a specific
ordering, while sorting the rest of the elements according to
normal :func:`sorted` rules.

>>> soft_sorted(['two', 'b', 'one', 'a'], first=['one', 'two'])
['one', 'two', 'a', 'b']
>>> soft_sorted(range(7), first=[6, 15], last=[2, 4], reverse=True)
[6, 5, 3, 1, 0, 2, 4]
>>> import string
>>> ''.join(soft_sorted(string.hexdigits, first='za1', last='b', key=str.lower))
'aA1023456789cCdDeEfFbB'

Args:
   iterable (list): A list or other iterable to sort.
   first (list): A sequence to enforce for elements which should
      appear at the beginning of the returned list.
   last (list): A sequence to enforce for elements which should
      appear at the end of the returned list.
   key (callable): Callable used to generate a comparable key for
      each item to be sorted, same as the key in
      :func:`sorted`. Note that entries in *first* and *last*
      should be the keys for the items. Defaults to
      passthrough/the identity function.
   reverse (bool): Whether or not elements not explicitly ordered
      by *first* and *last* should be in reverse order or not.

Returns a new list in sorted order.
