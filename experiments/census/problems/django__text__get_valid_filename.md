django.utils.text.get_valid_filename (from django 6.1.2)

Signature, in words: `get_valid_filename` takes the required argument `name`.

The docstring:

Return the given string converted to a string that can be used for a clean
filename. Remove leading and trailing spaces; convert other spaces to
underscores; and remove anything that is not an alphanumeric, dash,
underscore, or dot.
>>> get_valid_filename("john's portrait in 2004.jpg")
'johns_portrait_in_2004.jpg'
