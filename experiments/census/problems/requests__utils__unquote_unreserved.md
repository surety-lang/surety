requests.utils.unquote_unreserved (from requests 2.34.2)

Signature, in words: `unquote_unreserved` takes the required argument `uri`. It returns a value of type `str`.

The docstring:

Un-escape any percent-escape sequences in a URI that are unreserved
characters. This leaves all reserved, illegal and non-ASCII bytes encoded.

:rtype: str
