boltons.strutils.ordinalize (from boltons 26.2.0)

Signature, in words: `ordinalize` takes the required argument `number`; the optional argument `ext_only` (default `False`).

The docstring:

Turns *number* into its cardinal form, i.e., 1st, 2nd,
3rd, 4th, etc. If the last character isn't a digit, it returns the
string value unchanged.

Args:
    number (int or str): Number to be cardinalized.
    ext_only (bool): Whether to return only the suffix. Default ``False``.

>>> print(ordinalize(1))
1st
>>> print(ordinalize(3694839230))
3694839230th
>>> print(ordinalize('hi'))
hi
>>> print(ordinalize(1515))
1515th
