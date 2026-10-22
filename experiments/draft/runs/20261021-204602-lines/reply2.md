```surety
# The result is Ok exactly when the email has a valid shape, the age is from 13 to 120 inclusive, and the password is at least 12 characters long.
	reaching for: (email_shape_ok(form.email) and form.age >= 13 and form.age <= 120 and length_of_text(form.password) >= 12) <-> exactly_one Ok(_) in out. The notation has no way to look inside a Text or count its characters, so neither the email-shape test nor the password-length test can be written.
# An email has a valid shape when it contains exactly one @, with non-empty text before it and a dot with text on both sides after it.
	reaching for: a test on a Text value that counts the occurrences of "@" (exactly one), checks that the part before it is non-empty, and checks that the part after it contains a "." with non-empty text on each side. The notation has no character count and no split or substring operations on Text.
# An age of exactly 13 and an age of exactly 120 are both accepted.
	reaching for: (form.age == 13 or form.age == 120) -> all Err(errs) in out: age_out_of_range_err not in errs. No named value exists for the age-out-of-range error, and none can be declared because its type, ValidationError, is unknown to the checker.
# A password of exactly 12 characters is accepted.
	reaching for: length_of_text(form.password) == 12 -> all Err(errs) in out: password_too_short_err not in errs. The notation has no length of a Text, and no named value exists for the password-too-short error.
# The Ok record's email is the form's email, unchanged.
	all Ok(r) in out: r.email == form.email
# The Ok record's age is the form's age, unchanged.
	all Ok(r) in out: r.age == form.age
# The Ok record's password is the form's password, unchanged.
	all Ok(r) in out: r.password == form.password
# The invalid-email error is reported exactly when the email does not have a valid shape.
	reaching for: all Err(errs) in out: invalid_email_err in errs <-> not email_shape_ok(form.email). No named value exists for the invalid-email error, because its type ValidationError is unknown, and the email-shape test cannot be written on Text.
# The age-out-of-range error is reported exactly when the age is below 13 or above 120.
	reaching for: all Err(errs) in out: age_out_of_range_err in errs <-> (form.age < 13 or form.age > 120). No named value exists for the age-out-of-range error, because its type ValidationError is unknown.
# The password-too-short error is reported exactly when the password is shorter than 12 characters.
	reaching for: all Err(errs) in out: password_too_short_err in errs <-> length_of_text(form.password) < 12. No named value exists for the password-too-short error, and the notation has no length of a Text.
# Every failing check is reported together, not only the first one found.
	reaching for: all Err(errs) in out: every one of the three errors whose check fails is in errs. Stating this needs named values for the three errors, which cannot exist while ValidationError is unknown, and it needs the email-shape and password-length tests on Text.
# An Err never carries an empty error list.
	all Err(errs) in out: len(errs) > 0
# Every reported error is one of the three signup errors.
	reaching for: all Err(errs) in out: errs drawn_from [invalid_email_err, age_out_of_range_err, password_too_short_err]. None of the three error values is declared, and none can be declared because their type, ValidationError, is unknown to the checker.
# No error is reported twice.
	all Err(errs) in out: unique errs
# The order of the reported errors does not matter.
	all Err(errs) in out: errs unordered
```