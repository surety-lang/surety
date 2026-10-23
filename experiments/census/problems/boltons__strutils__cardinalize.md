boltons.strutils.cardinalize (from boltons 26.2.0)

Signature, in words: `cardinalize` takes the required arguments `unit_noun`, `count`.

The docstring:

Conditionally pluralizes a singular word *unit_noun* if
*count* is not one, preserving case when possible.

>>> vowels = 'aeiou'
>>> print(len(vowels), cardinalize('vowel', len(vowels)))
5 vowels
>>> print(3, cardinalize('Wish', 3))
3 Wishes
