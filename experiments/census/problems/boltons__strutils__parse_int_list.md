boltons.strutils.parse_int_list (from boltons 26.2.0)

Signature, in words: `parse_int_list` takes the required argument `range_string`; the optional arguments `delim` (default `','`), `range_delim` (default `'-'`).

The docstring:

Returns a sorted list of positive integers based on
*range_string*. Reverse of :func:`format_int_list`.

Args:
    range_string (str): String of comma separated positive
        integers or ranges (e.g. '1,2,4-6,8'). Typical of a custom
        page range string used in printer dialogs.
    delim (char): Defaults to ','. Separates integers and
        contiguous ranges of integers.
    range_delim (char): Defaults to '-'. Indicates a contiguous
        range of integers.

>>> parse_int_list('1,3,5-8,10-11,15')
[1, 3, 5, 6, 7, 8, 10, 11, 15]
