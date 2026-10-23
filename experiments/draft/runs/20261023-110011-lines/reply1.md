```surety
# The output has exactly as many components as Xarr has.
	len(out) == len(Xarr)
# Applying the sibling dft to the output gives back Xarr component by component, each pair judged close by a sibling closeness check, the way the docstring uses cmath.isclose.
	reaching for: for every position k from 0 up to len(Xarr), close(dft(out)[k], Xarr[k]), where dft is the sibling transform and close is the sibling closeness check named with no body; the notation has the call and the bodiless predicate, but it has no integer position binder, no single-element index and no zip, so I cannot pair the two lists position by position. `out same_order_as` and `same_elements_as` compare by exact equality, not closeness, and `all x in Xarr: any y in dft(out): close(x, y)` would widen the sentence by dropping the pairing.
```