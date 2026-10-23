boltons.strutils.unit_len (from boltons 26.2.0)

Signature, in words: `unit_len` takes the required argument `sized_iterable`; the optional argument `unit_noun` (default `'item'`).

The docstring:

Returns a plain-English description of an iterable's
:func:`len()`, conditionally pluralized with :func:`cardinalize`,
detailed below.

>>> print(unit_len(range(10), 'number'))
10 numbers
>>> print(unit_len('aeiou', 'vowel'))
5 vowels
>>> print(unit_len([], 'worry'))
No worries
