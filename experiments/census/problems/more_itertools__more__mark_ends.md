more_itertools.more.mark_ends (from more-itertools 11.2.1)

Signature, in words: `mark_ends` takes the required argument `iterable`.

The docstring:

Yield 3-tuples of the form ``(is_first, is_last, item)``.

>>> list(mark_ends('ABC'))
[(True, False, 'A'), (False, False, 'B'), (False, True, 'C')]

Use this when looping over an iterable to take special action on its first
and/or last items:

>>> iterable = ['Header', 100, 200, 'Footer']
>>> total = 0
>>> for is_first, is_last, item in mark_ends(iterable):
...     if is_first:
...         continue  # Skip the header
...     if is_last:
...         continue  # Skip the footer
...     total += item
>>> print(total)
300
