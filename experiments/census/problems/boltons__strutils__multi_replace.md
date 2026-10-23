boltons.strutils.multi_replace (from boltons 26.2.0)

Signature, in words: `multi_replace` takes the required arguments `text`, `sub_map`; any keyword arguments `kwargs`.

The docstring:

Shortcut function to invoke MultiReplace in a single call.

Example Usage::

    from boltons.strutils import multi_replace
    new = multi_replace(
        'The foo bar cat ate a bat',
        {'foo': 'zoo', 'cat': 'hat', 'bat': 'kraken'}
    )
    new == 'The zoo bar hat ate a kraken'
