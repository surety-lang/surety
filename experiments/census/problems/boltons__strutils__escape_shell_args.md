boltons.strutils.escape_shell_args (from boltons 26.2.0)

Signature, in words: `escape_shell_args` takes the required argument `args`; the optional arguments `sep` (default `' '`), `style` (default none).

The docstring:

Returns an escaped version of each string in *args*, according to
*style*.

Args:
    args (list): A list of arguments to escape and join together
    sep (str): The separator used to join the escaped arguments.
    style (str): The style of escaping to use. Can be one of
      ``cmd`` or ``sh``, geared toward Windows and Linux/BSD/etc.,
      respectively. If *style* is ``None``, then it is picked
      according to the system platform.

See :func:`args2cmd` and :func:`args2sh` for details and example
output for each style.
