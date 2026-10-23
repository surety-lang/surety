boltons.iterutils.chunk_ranges (from boltons 26.2.0)

Signature, in words: `chunk_ranges` takes the required arguments `input_size`, `chunk_size`; the optional arguments `input_offset` (default `0`), `overlap_size` (default `0`), `align` (default `False`).

The docstring:

Generates *chunk_size*-sized chunk ranges for an input with length *input_size*.
Optionally, a start of the input can be set via *input_offset*, and
and overlap between the chunks may be specified via *overlap_size*.
Also, if *align* is set to *True*, any items with *i % (chunk_size-overlap_size) == 0*
are always at the beginning of the chunk.

Returns an iterator of (start, end) tuples, one tuple per chunk.

>>> list(chunk_ranges(input_offset=10, input_size=10, chunk_size=5))
[(10, 15), (15, 20)]
>>> list(chunk_ranges(input_offset=10, input_size=10, chunk_size=5, overlap_size=1))
[(10, 15), (14, 19), (18, 20)]
>>> list(chunk_ranges(input_offset=10, input_size=10, chunk_size=5, overlap_size=2))
[(10, 15), (13, 18), (16, 20)]

>>> list(chunk_ranges(input_offset=4, input_size=15, chunk_size=5, align=False))
[(4, 9), (9, 14), (14, 19)]
>>> list(chunk_ranges(input_offset=4, input_size=15, chunk_size=5, align=True))
[(4, 5), (5, 10), (10, 15), (15, 19)]

>>> list(chunk_ranges(input_offset=2, input_size=15, chunk_size=5, overlap_size=1, align=False))
[(2, 7), (6, 11), (10, 15), (14, 17)]
>>> list(chunk_ranges(input_offset=2, input_size=15, chunk_size=5, overlap_size=1, align=True))
[(2, 5), (4, 9), (8, 13), (12, 17)]
>>> list(chunk_ranges(input_offset=3, input_size=15, chunk_size=5, overlap_size=1, align=True))
[(3, 5), (4, 9), (8, 13), (12, 17), (16, 18)]
