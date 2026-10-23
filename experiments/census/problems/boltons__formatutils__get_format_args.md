boltons.formatutils.get_format_args (from boltons 26.2.0)

Signature, in words: `get_format_args` takes the required argument `fstr`.

The docstring:

Turn a format string into two lists of arguments referenced by the
format string. One is positional arguments, and the other is named
arguments. Each element of the list includes the name and the
nominal type of the field.

# >>> get_format_args("{noun} is {1:d} years old{punct}")
# ([(1, <type 'int'>)], [('noun', <type 'str'>), ('punct', <type 'str'>)])

# XXX: Py3k
>>> get_format_args("{noun} is {1:d} years old{punct}") ==         ([(1, int)], [('noun', str), ('punct', str)])
True
