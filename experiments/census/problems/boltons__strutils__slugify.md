boltons.strutils.slugify (from boltons 26.2.0)

Signature, in words: `slugify` takes the required argument `text`; the optional arguments `delim` (default `'_'`), `lower` (default `True`), `ascii` (default `False`).

The docstring:

A basic function that turns text full of scary characters
(i.e., punctuation and whitespace), into a relatively safe
lowercased string separated only by the delimiter specified
by *delim*, which defaults to ``_``.

The *ascii* convenience flag will :func:`asciify` the slug if
you require ascii-only slugs.

>>> slugify('First post! Hi!!!!~1    ')
'first_post_hi_1'

>>> slugify("Kurt Gödel's pretty cool.", ascii=True) ==         b'kurt_goedel_s_pretty_cool'
True
