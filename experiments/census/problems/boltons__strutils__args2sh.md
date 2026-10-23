boltons.strutils.args2sh (from boltons 26.2.0)

Signature, in words: `args2sh` takes the required argument `args`; the optional argument `sep` (default `' '`).

The docstring:

Return a shell-escaped string version of *args*, separated by
*sep*, based on the rules of sh, bash, and other shells in the
Linux/BSD/MacOS ecosystem.

>>> print(args2sh(['aa', '[bb]', "cc'cc", 'dd"dd']))
aa '[bb]' 'cc'"'"'cc' 'dd"dd'

As you can see, arguments with no special characters are not
escaped, arguments with special characters are quoted with single
quotes, and single quotes themselves are quoted with double
quotes. Double quotes are handled like any other special
character.

Based on code from the :mod:`pipes`/:mod:`shlex` modules. Also
note that :mod:`shlex` and :mod:`argparse` have functions to split
and parse strings escaped in this manner.
