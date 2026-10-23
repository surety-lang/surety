boltons.strutils.unwrap_text (from boltons 26.2.0)

Signature, in words: `unwrap_text` takes the required argument `text`; the optional argument `ending` (default `'\n\n'`).

The docstring:

Unwrap text, the natural complement to :func:`textwrap.wrap`.

>>> text = "Short \n lines  \nwrapped\nsmall.\n\nAnother\nparagraph."
>>> unwrap_text(text)
'Short lines wrapped small.\n\nAnother paragraph.'

Args:
   text: A string to unwrap.
   ending (str): The string to join all unwrapped paragraphs
      by. Pass ``None`` to get the list. Defaults to '\n\n' for
      compatibility with Markdown and RST.
