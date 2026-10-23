boltons.urlutils.find_all_links (from boltons 26.2.0)

Signature, in words: `find_all_links` takes the required argument `text`; the optional arguments `with_text` (default `False`), `default_scheme` (default `'https'`), `schemes` (default `()`).

The docstring:

This function uses heuristics to searches plain text for strings
that look like URLs, returning a :class:`list` of :class:`URL`
objects. It supports limiting the accepted schemes, and returning
interleaved text as well.

>>> find_all_links('Visit https://boltons.rtfd.org!')
[URL(u'https://boltons.rtfd.org')]
>>> find_all_links('Visit https://boltons.rtfd.org!', with_text=True)
[u'Visit ', URL(u'https://boltons.rtfd.org'), u'!']

Args:
   text (str): The text to search.

   with_text (bool): Whether or not to interleave plaintext blocks
      with the returned URL objects. Having all tokens can be
      useful for transforming the text, e.g., replacing links with
      HTML equivalents. Defaults to ``False``.

   default_scheme (str): Many URLs are written without the scheme
      component. This function can match a reasonable subset of
      those, provided *default_scheme* is set to a string. Set to
      ``False`` to disable matching scheme-less URLs. Defaults to
      ``'https'``.

   schemes (list): A list of strings that a URL's scheme must
      match in order to be included in the results. Defaults to
      empty, which matches all schemes.

.. note:: Currently this function does not support finding IPv6
  addresses or URLs with netloc-less schemes, like mailto.
