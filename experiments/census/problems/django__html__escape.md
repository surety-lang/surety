django.utils.html.escape (from django 6.1.2)

Signature, in words: `escape` takes the required argument `text`.

The docstring:

Return the given text with ampersands, quotes and angle brackets encoded
for use in HTML.

Always escape input, even if it's already escaped and marked as such.
This may result in double-escaping. If this is a concern, use
conditional_escape() instead.
