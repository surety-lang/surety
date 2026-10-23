django.utils.http.split_header_value (from django 6.1.2)

Signature, in words: `split_header_value` takes the required argument `value`; the optional argument `sep` (default `','`).

The docstring:

Yield stripped parts of an HTTP header value split by sep.

Use only with headers whose values are token lists (e.g. Vary,
Cache-Control). Do not use with headers that allow quoted strings in values
(e.g. Set-Cookie), as commas inside values will be used as separators.
