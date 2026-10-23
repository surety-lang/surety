boltons.strutils.find_hashtags (from boltons 26.2.0)

Signature, in words: `find_hashtags` takes the required argument `string`.

The docstring:

Finds and returns all hashtags in a string, with the hashmark
removed. Supports full-width hashmarks for Asian languages and
does not false-positive on URL anchors.

>>> find_hashtags('#atag http://asite/#ananchor')
['atag']

``find_hashtags`` also works with unicode hashtags.
