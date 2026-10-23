more_itertools.more.spy (from more-itertools 11.2.1)

Signature, in words: `spy` takes the required argument `iterable`; the optional argument `n` (default `1`).

The docstring:

Return a 2-tuple with a list containing the first *n* elements of
*iterable*, and an iterator with the same items as *iterable*.
This allows you to "look ahead" at the items in the iterable without
advancing it.

There is one item in the list by default:

    >>> iterable = 'abcdefg'
    >>> head, iterable = spy(iterable)
    >>> head
    ['a']
    >>> list(iterable)
    ['a', 'b', 'c', 'd', 'e', 'f', 'g']

You may use unpacking to retrieve items instead of lists:

    >>> (head,), iterable = spy('abcdefg')
    >>> head
    'a'
    >>> (first, second), iterable = spy('abcdefg', 2)
    >>> first
    'a'
    >>> second
    'b'

The number of items requested can be larger than the number of items in
the iterable:

    >>> iterable = [1, 2, 3, 4, 5]
    >>> head, iterable = spy(iterable, 10)
    >>> head
    [1, 2, 3, 4, 5]
    >>> list(iterable)
    [1, 2, 3, 4, 5]
