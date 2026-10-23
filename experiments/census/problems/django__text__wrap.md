django.utils.text.wrap (from django 6.1.2)

Signature, in words: `wrap` takes the required arguments `text`, `width`.

The docstring:

A word-wrap function that preserves existing line breaks. Expects that
existing line breaks are posix newlines.

Preserve all white space except added line breaks consume the space on
which they break the line.

Don't wrap long words, thus the output text may have lines longer than
``width``.
