requests.utils.parse_header_links (from requests 2.34.2)

Signature, in words: `parse_header_links` takes the required argument `value`. It returns a value of type `list[dict[str, str]]`.

The docstring:

Return a list of parsed link headers proxies.

i.e. Link: <http:/.../front.jpeg>; rel=front; type="image/jpeg",<http://.../back.jpeg>; rel=back;type="image/jpeg"

:rtype: list
