requests.utils.parse_dict_header (from requests 2.34.2)

Signature, in words: `parse_dict_header` takes the required argument `value`. It returns a value of type `dict[str, str | None]`.

The docstring:

Parse lists of key, value pairs as described by RFC 2068 Section 2 and
convert them into a python dict:

>>> d = parse_dict_header('foo="is a fish", bar="as well"')
>>> type(d) is dict
True
>>> sorted(d.items())
[('bar', 'as well'), ('foo', 'is a fish')]

If there is no value for a key it will be `None`:

>>> parse_dict_header('key_without_value')
{'key_without_value': None}

To create a header from the :class:`dict` again, use the
:func:`dump_header` function.

:param value: a string with a dict header.
:return: :class:`dict`
:rtype: dict
