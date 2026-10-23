boltons.urlutils.parse_url (from boltons 26.2.0)

Signature, in words: `parse_url` takes the required argument `url_text`.

The docstring:

Used to parse the text for a single URL into a dictionary, used
internally by the :class:`URL` type.

Note that "URL" has a very narrow, standards-based
definition. While :func:`parse_url` may raise
:class:`URLParseError` under a very limited number of conditions,
such as non-integer port, a surprising number of strings are
technically valid URLs. For instance, the text ``"url"`` is a
valid URL, because it is a relative path.

In short, do not expect this function to validate form inputs or
other more colloquial usages of URLs.

>>> res = parse_url('http://127.0.0.1:3000/?a=1')
>>> sorted(res.keys())  # res is a basic dictionary
['_netloc_sep', 'authority', 'family', 'fragment', 'host', 'password', 'path', 'port', 'query', 'scheme', 'username']
