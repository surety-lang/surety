boltons.strutils.removeprefix (from boltons 26.2.0)

Signature, in words: `removeprefix` takes the required arguments `text`, `prefix`. It returns a value of type `str`.

The docstring:

Remove `prefix` from start of `text` if present.

Backport of `str.removeprefix` for Python versions less than 3.9.

Args:
    text: A string to remove the prefix from.
    prefix: The string to remove from the beginning of `text`.
