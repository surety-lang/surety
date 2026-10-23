more_itertools.more.filter_except (from more-itertools 11.2.1)

Signature, in words: `filter_except` takes the required arguments `validator`, `iterable`; any number of positional arguments `exceptions`.

The docstring:

Yield the items from *iterable* for which the *validator* function does
not raise one of the specified *exceptions*.

*validator* is called for each item in *iterable*.
It should be a function that accepts one argument and raises an exception
if that item is not valid.

>>> iterable = ['1', '2', 'three', '4', None]
>>> list(filter_except(int, iterable, ValueError, TypeError))
['1', '2', '4']

If an exception other than one given by *exceptions* is raised by
*validator*, it is raised like normal.
