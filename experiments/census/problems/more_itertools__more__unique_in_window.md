more_itertools.more.unique_in_window (from more-itertools 11.2.1)

Signature, in words: `unique_in_window` takes the required arguments `iterable`, `n`; the optional argument `key` (default none).

The docstring:

Yield the items from *iterable* that haven't been seen recently.
*n* is the size of the sliding window.

    >>> iterable = [0, 1, 0, 2, 3, 0]
    >>> n = 3
    >>> list(unique_in_window(iterable, n))
    [0, 1, 2, 3, 0]

The *key* function, if provided, will be used to determine uniqueness:

    >>> list(unique_in_window('abAcda', 3, key=lambda x: x.lower()))
    ['a', 'b', 'c', 'd', 'a']

Updates a sliding window no larger than n and yields a value
if the item only occurs once in the updated window.

When `n == 1`, *unique_in_window* is memoryless:

    >>> list(unique_in_window('aab', n=1))
    ['a', 'a', 'b']

The items in *iterable* must be hashable.
