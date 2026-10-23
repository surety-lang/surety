```surety
// draft
// drafted
type Complex = { re: Float, im: Float }

idft(Xarr: Complex[]) -> Complex[]
	note "yield the inverse discrete Fourier transform of Xarr, one time-domain component per frequency-domain component; floats because the twiddle factors are irrational, so no exact integer representation exists"
	# The output has exactly as many components as Xarr has.

	# Applying the sibling dft to the output gives back Xarr component by component, each pair judged close by a sibling closeness check, the way the docstring uses cmath.isclose.

	// example wanted: the docstring's frequency vector [2, -2-2j, -2j, 4+4j]; the docstring's time-domain vector decides it, compared with cmath.isclose
	// example wanted: an empty Xarr; the length sentence decides it
	// example wanted: a single component; the transform of length one divides by 1 and turns by angle 0
	// example wanted: four equal components; the constant spectrum transforms to an impulse at position 0, and closeness decides the other positions
	// example wanted: Xarr all zeros; every term of every sum is zero, so the output is exact rather than close
	// example wanted: purely real integer inputs, such as [1, 2, 3], given with im 0; the docstring admits int, and the round-trip sentence decides it
```