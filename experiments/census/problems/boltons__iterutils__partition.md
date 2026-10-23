boltons.iterutils.partition (from boltons 26.2.0)

Signature, in words: `partition` takes the required argument `src`; any number of positional arguments `keys`; the optional argument `key` (default the `bool` built-in).

The docstring:

No relation to :meth:`str.partition`, ``partition`` is like
:func:`bucketize`, but for added convenience returns a collection for
each predicate passed.

``partition`` now accepts multiple *key* functions and will return
``N + 1`` lists for ``N`` predicates. Each value from *src* is placed
into the first list whose predicate evaluates to ``True`` with values
that match none of the predicates placed in the last list.

>>> nonempty, empty = partition(['', '', 'hi', '', 'bye'])
>>> nonempty
['hi', 'bye']

*key* defaults to :class:`bool`, but can be carefully overridden to
use either a function that returns either ``True`` or ``False`` or
a string name of the attribute on which to partition objects.

>>> import string
>>> is_digit = lambda x: x in string.digits
>>> decimal_digits, hexletters = partition(string.hexdigits, is_digit)
>>> ''.join(decimal_digits), ''.join(hexletters)
('0123456789', 'abcdefABCDEF')

Multiple predicates may be supplied to divide into more buckets:

>>> positive, negative, zero = partition(range(-1, 2),
...                                     lambda i: i > 0,
...                                     lambda i: i < 0)
>>> positive, negative, zero
([1], [-1], [0])
