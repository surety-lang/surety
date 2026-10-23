django.utils.http.split_directive_names (from django 6.1.2)

Signature, in words: `split_directive_names` takes the required argument `value`.

The docstring:

Yield the lowercased directive names from an HTTP header value.

Any qualified value is discarded, so that qualified forms permitted by
RFC 9111 (e.g. `private="Set-Cookie"`) reduce to their directive name
("private").

Use to check for the presence of a directive when its value is not needed;
use `split_header_value()` when the value matters (e.g. `max-age`).
