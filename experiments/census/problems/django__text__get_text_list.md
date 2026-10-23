django.utils.text.get_text_list (from django 6.1.2)

Signature, in words: `get_text_list` takes the required argument `list_`; the optional argument `last_word` (no default given: a sentinel meaning 'not supplied').

The docstring:

>>> get_text_list(['a', 'b', 'c', 'd'])
'a, b, c or d'
>>> get_text_list(['a', 'b', 'c'], 'and')
'a, b and c'
>>> get_text_list(['a', 'b'], 'and')
'a and b'
>>> get_text_list(['a'])
'a'
>>> get_text_list([])
''
