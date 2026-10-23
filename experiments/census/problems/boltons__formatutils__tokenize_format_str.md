boltons.formatutils.tokenize_format_str (from boltons 26.2.0)

Signature, in words: `tokenize_format_str` takes the required argument `fstr`; the optional argument `resolve_pos` (default `True`).

The docstring:

Takes a format string, turns it into a list of alternating string
literals and :class:`BaseFormatField` tokens. By default, also
infers anonymous positional references into explicit, numbered
positional references. To disable this behavior set *resolve_pos*
to ``False``.
