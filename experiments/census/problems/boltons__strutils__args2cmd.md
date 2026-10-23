boltons.strutils.args2cmd (from boltons 26.2.0)

Signature, in words: `args2cmd` takes the required argument `args`; the optional argument `sep` (default `' '`).

The docstring:

Return a shell-escaped string version of *args*, separated by
*sep*, using the same rules as the Microsoft C runtime.

>>> print(args2cmd(['aa', '[bb]', "cc'cc", 'dd"dd']))
aa [bb] cc'cc dd\"dd

As you can see, escaping is through backslashing and not quoting,
and double quotes are the only special character. See the comment
in the code for more details. Based on internal code from the
:mod:`subprocess` module.
