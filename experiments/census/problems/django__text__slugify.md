django.utils.text.slugify (from django 6.1.2)

Signature, in words: `slugify` takes the required argument `value`; the optional argument `allow_unicode` (default `False`).

The docstring:

Convert to ASCII if 'allow_unicode' is False. Convert spaces or repeated
dashes to single dashes. Remove characters that aren't alphanumerics,
underscores, or hyphens. Convert to lowercase. Also strip leading and
trailing whitespace, dashes, and underscores.
