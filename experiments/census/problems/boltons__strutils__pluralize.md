boltons.strutils.pluralize (from boltons 26.2.0)

Signature, in words: `pluralize` takes the required argument `word`.

The docstring:

Semi-intelligently converts an English *word* from singular form to
plural, preserving case pattern.

>>> pluralize('friend')
'friends'
>>> pluralize('enemy')
'enemies'
>>> pluralize('Sheep')
'Sheep'
