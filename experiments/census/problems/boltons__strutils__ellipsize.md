boltons.strutils.ellipsize (from boltons 26.2.0)

Signature, in words: `ellipsize` takes the required argument `text`; the optional argument `max_len` (default `160`); the keyword-only argument `ellipsis` (default `'…'`).

The docstring:

Truncate *text* to at most *max_len* characters, cutting at the
last space before the limit and appending *ellipsis*. The returned
string, ellipsis included, is never longer than *max_len*.

Text short enough to fit is returned unchanged:

>>> ellipsize('Hello, World!', 16)
'Hello, World!'

Longer text is cut at a space, never mid-word, and trailing
punctuation at the cut is stripped:

>>> ellipsize('Beautiful is better than ugly. Explicit is better.', 31)
'Beautiful is better than ugly…'

A ``.`` between two digits is a decimal point, not sentence
punctuation, and numbers are kept whole:

>>> ellipsize('rates around 6.5% this week', 20)
'rates around 6.5%…'

A single token longer than *max_len* is hard-cut at the limit:

>>> ellipsize('antidisestablishmentarianism', 10)
'antidises…'

Args:
    text (str): The string to truncate.
    max_len (int): Maximum length of the result, including the
        ellipsis. Must be greater than ``len(ellipsis)``.
    ellipsis (str): The suffix appended to truncated text.
        Defaults to ``'…'`` (U+2026, HORIZONTAL ELLIPSIS).
