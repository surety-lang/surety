boltons.strutils.indent (from boltons 26.2.0)

Signature, in words: `indent` takes the required arguments `text`, `margin`; the optional arguments `newline` (default `'\n'`), `key` (default the `bool` built-in).

The docstring:

The missing counterpart to the built-in :func:`textwrap.dedent`.

Args:
    text (str): The text to indent.
    margin (str): The string to prepend to each line.
    newline (str): The newline used to rejoin the lines (default: ``\n``)
    key (callable): Called on each line to determine whether to
      indent it. Default: :class:`bool`, to ensure that empty lines do
      not get whitespace added.
