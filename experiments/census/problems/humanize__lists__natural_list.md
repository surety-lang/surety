humanize.lists.natural_list (from humanize 4.16.0)

Signature, in words: `natural_list` takes the required argument `items`. It returns a value of type `str`.

The docstring:

Natural list.

Convert a list of items into a human-readable string with commas and 'and'.

Examples:
    >>> natural_list(["one", "two", "three"])
    'one, two and three'
    >>> natural_list(["one", "two"])
    'one and two'
    >>> natural_list(["one"])
    'one'

Args:
    items (list): An iterable of items.

Returns:
    str: A string with commas and 'and' in the right places.
