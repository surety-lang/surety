boltons.iterutils.bucketize (from boltons 26.2.0)

Signature, in words: `bucketize` takes the required argument `src`; the optional arguments `key` (default the `bool` built-in), `value_transform` (default none), `key_filter` (default none).

The docstring:

Group values in the *src* iterable by the value returned by *key*.

>>> bucketize(range(5))
{False: [0], True: [1, 2, 3, 4]}
>>> is_odd = lambda x: x % 2 == 1
>>> bucketize(range(5), is_odd)
{False: [0, 2, 4], True: [1, 3]}

*key* is :class:`bool` by default, but can either be a callable or a string or a list
if it is a string, it is the name of the attribute on which to bucketize objects.

>>> bucketize([1+1j, 2+2j, 1, 2], key='real')
{1.0: [(1+1j), 1], 2.0: [(2+2j), 2]}

if *key* is a list, it contains the buckets where to put each object

>>> bucketize([1,2,365,4,98],key=[0,1,2,0,2])
{0: [1, 4], 1: [2], 2: [365, 98]}


Value lists are not deduplicated:

>>> bucketize([None, None, None, 'hello'])
{False: [None, None, None], True: ['hello']}

Bucketize into more than 3 groups

>>> bucketize(range(10), lambda x: x % 3)
{0: [0, 3, 6, 9], 1: [1, 4, 7], 2: [2, 5, 8]}

``bucketize`` has a couple of advanced options useful in certain
cases.  *value_transform* can be used to modify values as they are
added to buckets, and *key_filter* will allow excluding certain
buckets from being collected.

>>> bucketize(range(5), value_transform=lambda x: x*x)
{False: [0], True: [1, 4, 9, 16]}

>>> bucketize(range(10), key=lambda x: x % 3, key_filter=lambda k: k % 3 != 1)
{0: [0, 3, 6, 9], 2: [2, 5, 8]}

Note in some of these examples there were at most two keys, ``True`` and
``False``, and each key present has a list with at least one
item. See :func:`partition` for a version specialized for binary
use cases.
