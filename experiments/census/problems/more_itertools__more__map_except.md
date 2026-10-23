more_itertools.more.map_except (from more-itertools 11.2.1)

Signature, in words: `map_except` takes the required arguments `function`, `iterable`; any number of positional arguments `exceptions`.

The docstring:

Transform each item from *iterable* with *function* and yield the
result, unless *function* raises one of the specified *exceptions*.

*function* is called to transform each item in *iterable*.
It should accept one argument.

>>> iterable = ['1', '2', 'three', '4', None]
>>> list(map_except(int, iterable, ValueError, TypeError))
[1, 2, 4]

If an exception other than one given by *exceptions* is raised by
*function*, it is raised like normal.
