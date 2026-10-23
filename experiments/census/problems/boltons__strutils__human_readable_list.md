boltons.strutils.human_readable_list (from boltons 26.2.0)

Signature, in words: `human_readable_list` takes the required argument `items`; the optional arguments `delimiter` (default `','`), `conjunction` (default `'and'`); the keyword-only argument `oxford` (default `True`). It returns a value of type `str`.

The docstring:

Given a list of strings, return a human readable string with
appropriate delimiters and the conjunction word.

Args:
    items: The list of strings to join.
    delimiter (optional): The delimiter to use between items.
    conjunction (optional): The word to use before the last item.
    oxford (optional): Whether to use the Oxford comma/delimiter before
        the conjunction in lists of 3+ items.

Returns:
    str: The human readable string.
