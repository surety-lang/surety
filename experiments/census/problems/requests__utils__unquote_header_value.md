requests.utils.unquote_header_value (from requests 2.34.2)

Signature, in words: `unquote_header_value` takes the required argument `value`; the optional argument `is_filename` (default `False`). It returns a value of type `str`.

The docstring:

Unquotes a header value.  (Reversal of :func:`quote_header_value`).
This does not use the real unquoting but what browsers are actually
using for quoting.

:param value: the header value to unquote.
:rtype: str
