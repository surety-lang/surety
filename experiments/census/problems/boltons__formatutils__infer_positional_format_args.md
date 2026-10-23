boltons.formatutils.infer_positional_format_args (from boltons 26.2.0)

Signature, in words: `infer_positional_format_args` takes the required argument `fstr`.

The docstring:

Takes format strings with anonymous positional arguments, (e.g.,
"{}" and {:d}), and converts them into numbered ones for explicitness and
compatibility with 2.6.

Returns a string with the inferred positional arguments.
