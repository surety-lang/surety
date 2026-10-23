boltons.urlutils.unquote (from boltons 26.2.0)

Signature, in words: `unquote` takes the required argument `string`; the optional arguments `encoding` (default `'utf-8'`), `errors` (default `'replace'`).

The docstring:

Percent-decode a string, by replacing %xx escapes with their
single-character equivalent. The optional *encoding* and *errors*
parameters specify how to decode percent-encoded sequences into
Unicode characters, as accepted by the :meth:`bytes.decode()` method.  By
default, percent-encoded sequences are decoded with UTF-8, and
invalid sequences are replaced by a placeholder character.

>>> unquote(u'abc%20def')
u'abc def'
