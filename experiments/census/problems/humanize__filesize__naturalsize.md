humanize.filesize.naturalsize (from humanize 4.16.0)

Signature, in words: `naturalsize` takes the required argument `value`; the optional arguments `binary` (default `False`), `gnu` (default `False`), `format` (default `'%.1f'`). It returns a value of type `str`.

The docstring:

Format a number of bytes like a human-readable filesize (e.g. 10 kB).

By default, decimal suffixes (kB, MB) are used.

Non-GNU modes are compatible with jinja2's `filesizeformat` filter.

Examples:
    ```pycon
    >>> naturalsize(3000000)
    '3.0 MB'
    >>> naturalsize(300, False, True)
    '300B'
    >>> naturalsize(3000, False, True)
    '2.9K'
    >>> naturalsize(3000, False, True, "%.3f")
    '2.930K'
    >>> naturalsize(3000, True)
    '2.9 KiB'
    >>> naturalsize(10**28)
    '10.0 RB'
    >>> naturalsize(10**34 * 3)
    '30000.0 QB'
    >>> naturalsize(-4096, True)
    '-4.0 KiB'

    ```

Args:
    value (int, float, str): Integer to convert.
    binary (bool): If `True`, uses binary suffixes (KiB, MiB) with base
        2<sup>10</sup> instead of 10<sup>3</sup>.
    gnu (bool): If `True`, the binary argument is ignored and GNU-style
        (`ls -sh` style) prefixes are used (K, M) with the 2**10 definition.
    format (str): Custom formatter.

Returns:
    str: Human readable representation of a filesize.
