boltons.strutils.singularize (from boltons 26.2.0)

Signature, in words: `singularize` takes the required argument `word`.

The docstring:

Semi-intelligently converts an English plural *word* to its
singular form, preserving case pattern.

>>> singularize('chances')
'chance'
>>> singularize('Activities')
'Activity'
>>> singularize('Glasses')
'Glass'
>>> singularize('FEET')
'FOOT'
