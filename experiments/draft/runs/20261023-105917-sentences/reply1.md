```surety
// draft
// drafted
type ComplexInt = { re: Int, im: Int }
// drafted
type Complex = { re: Float, im: Float }

dft(xarr: ComplexInt[]) -> Complex[]
	note "the discrete Fourier transform of xarr: one frequency-domain component per time-domain element"
	# The output has exactly as many components as the input has elements.

	# If the input is not empty, the first component's real part is the sum of the input elements' real parts.

	# If the input is not empty, the first component's imaginary part is the sum of the input elements' imaginary parts.

	# If every input element has a real part of 0 and an imaginary part of 0, every component has a real part of 0 and an imaginary part of 0.

	# Applying idft to the output gives back the input, each element close to the one at the same position (closeness is a sibling obligation's claim, and idft is a sibling obligation).

	// example wanted: the empty input; what decides it is that there are no components to yield
	// example wanted: a single element such as 3-2j; what decides it is that the only root of unity is exactly 1, so the one component is the element itself with no rounding
	// example wanted: the docstring's time-domain vector [1, 2-1j, -1j, -1+2j]; what decides it is comparison against the docstring's frequency-domain vector by closeness, not equality, since the roots of unity are inexact floats
	// example wanted: the real input [1, 1]; what decides it is that the second component multiplies by e^(-i*pi), whose imaginary part is a tiny nonzero float, so the expected value has to say whether that residue is part of the answer
```