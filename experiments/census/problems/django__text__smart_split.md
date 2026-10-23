django.utils.text.smart_split (from django 6.1.2)

Signature, in words: `smart_split` takes the required argument `text`.

The docstring:

Generator that splits a string by spaces, leaving quoted phrases together.
Supports both single and double quotes, and supports escaping quotes with
backslashes. In the output, strings will keep their initial and trailing
quote marks and escaped quotes will remain escaped (the results can then
be further processed with unescape_string_literal()).

>>> list(smart_split(r'This is "a person\'s" test.'))
['This', 'is', '"a person\\\'s"', 'test.']
>>> list(smart_split(r"Another 'person\'s' test."))
['Another', "'person\\'s'", 'test.']
>>> list(smart_split(r'A "\"funky\" style" test.'))
['A', '"\\"funky\\" style"', 'test.']
