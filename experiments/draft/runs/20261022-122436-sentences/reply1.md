```surety
// draft
// drafted
type PhoneError = { code: Text, display: Text }
// drafted
unknown_country_error: PhoneError = { code: "e-phone-country", display: "The default country is not a known country." }
// drafted
invalid_number_error: PhoneError = { code: "e-phone-invalid", display: "Please provide a valid phone number." }

normalize_phone(raw: Text, default_country: Text) -> Result<Text, PhoneError>
	note "turn a free-text phone number into its E.164 form, reading it in the default country unless it carries its own country code, or report why it cannot be"
	# The output is Ok exactly when the phone is a valid number, read with its own country code if it has one and with the default country otherwise.
	# An Ok output is a well-formed E.164 number.
	# If the phone carries no country code of its own, an Ok output contains the default country's calling code.
	# Normalizing an Ok output again, with the same default country, gives that same Ok output back.
	# The error is the unknown-country error exactly when the phone carries no country code of its own and the default country is not a known country.
	# The error is the invalid-number error exactly when the phone can be read in a country, either its own or a known default, and is not a valid number there.
	// example wanted: "(415) 555-0132" with default "US": no country code of its own, so the default country's code is applied
	// example wanted: "+44 20 7946 0958" with default "US": the phone's own code decides, and the default is not consulted
	// example wanted: "+44 20 7946 0958" with an unknown default country: the phone carries its own code, so the unknown default must not produce the unknown-country error
	// example wanted: "415 555 0132" with an unknown default country: no code of its own and no usable default, so the unknown-country error
	// example wanted: "020 7946 0958" with default "GB": decides whether the national trunk prefix 0 is dropped
	// example wanted: "0044 20 7946 0958" with default "GB": decides whether the 00 international prefix counts as the phone's own country code
	// example wanted: the empty text with default "US": nothing to read, decides which error is reported
	// example wanted: "+1 415 555 0132 ext 7" with default "US": decides whether an extension makes the number invalid or is dropped
	// example wanted: a number of exactly 15 digits after the "+", and one of 16: the E.164 length limit, crossed from both sides
```