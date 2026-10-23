django.utils.text.unescape_string_literal (from django 6.1.2)

Signature, in words: `unescape_string_literal` takes the required argument `s`.

The docstring:

Convert quoted string literals to unquoted strings with escaped quotes and
backslashes unquoted::

    >>> unescape_string_literal('"abc"')
    'abc'
    >>> unescape_string_literal("'abc'")
    'abc'
    >>> unescape_string_literal('"a \"bc\""')
    'a "bc"'
    >>> unescape_string_literal("'\'ab\' c'")
    "'ab' c"
