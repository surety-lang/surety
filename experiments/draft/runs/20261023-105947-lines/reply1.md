```surety
# The output has exactly as many components as the input has elements.
	len(out) == len(xarr)
# If the input is not empty, the first component's real part is the sum of the input elements' real parts.
	len(xarr) > 0 -> any c in out[0 .. 1]: c.re == sum(xarr.re)
# If the input is not empty, the first component's imaginary part is the sum of the input elements' imaginary parts.
	len(xarr) > 0 -> any c in out[0 .. 1]: c.im == sum(xarr.im)
# If every input element has a real part of 0 and an imaginary part of 0, every component has a real part of 0 and an imaginary part of 0.
	all c in out: (c.re == 0 and c.im == 0) or any x in xarr: x.re != 0 or x.im != 0
# Applying idft to the output gives back the input, each element close to the one at the same position (closeness is a sibling obligation's claim, and idft is a sibling obligation).
	reaching for: for every position i of xarr, close(idft(out)[i], xarr[i]), where close is a bodiless graph reference to the sibling closeness obligation. The fragment cannot pair two lists element by element by position: it has no index binder and no zip, and a slice like idft(out)[i .. i + 1] needs the position of x, which a binder does not expose. A list-level close_elementwise(idft(out), xarr) would hide the by-position pairing, which is this sentence's claim, inside the sibling's name.
```