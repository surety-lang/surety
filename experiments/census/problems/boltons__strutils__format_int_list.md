boltons.strutils.format_int_list (from boltons 26.2.0)

Signature, in words: `format_int_list` takes the required argument `int_list`; the optional arguments `delim` (default `','`), `range_delim` (default `'-'`), `delim_space` (default `False`).

The docstring:

Returns a sorted range string from a list of positive integers
(*int_list*). Contiguous ranges of integers are collapsed to min
and max values. Reverse of :func:`parse_int_list`.

Args:
    int_list (list): List of positive integers to be converted
       into a range string (e.g. [1,2,4,5,6,8]).
    delim (char): Defaults to ','. Separates integers and
       contiguous ranges of integers.
    range_delim (char): Defaults to '-'. Indicates a contiguous
       range of integers.
    delim_space (bool): Defaults to ``False``. If ``True``, adds a
       space after all *delim* characters.

>>> format_int_list([1,3,5,6,7,8,10,11,15])
'1,3,5-8,10-11,15'
