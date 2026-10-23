requests.utils.parse_list_header (from requests 2.34.2)

Signature, in words: `parse_list_header` takes the required argument `value`. It returns a value of type `list[str]`.

The docstring:

Parse lists as described by RFC 2068 Section 2.

In particular, parse comma-separated lists where the elements of
the list may include quoted-strings.  A quoted-string could
contain a comma.  A non-quoted string could have quotes in the
middle.  Quotes are removed automatically after parsing.

It basically works like :func:`parse_set_header` just that items
may appear multiple times and case sensitivity is preserved.

The return value is a standard :class:`list`:

>>> parse_list_header('token, "quoted value"')
['token', 'quoted value']

To create a header from the :class:`list` again, use the
:func:`dump_header` function.

:param value: a string with a list header.
:return: :class:`list`
:rtype: list
