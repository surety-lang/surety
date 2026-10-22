```surety
# The result is Ok exactly when the email has a valid shape, the age is from 13 to 120 inclusive, and the password is at least 12 characters long.
	reaching for: (email has a valid shape and 13 <= form.age and form.age <= 120 and form.password has at least 12 characters) <-> exactly_one Ok(_) in out. The notation has no way to look inside Text for a shape or to count its characters.
# An email has a valid shape when it contains exactly one @, with non-empty text before it and a dot with text on both sides after it.
	reaching for: a predicate on form.email that holds exactly when the text has one "@", non-empty text before it, and after it a "." with non-empty text on both sides. The notation has no operations for counting characters, splitting text, or testing what comes before or after a character.
# An age of exactly 13 and an age of exactly 120 are both accepted.
	reaching for: all Err(errs) in out: (form.age == 13 or form.age == 120) -> age_out_of_range_err not in errs. The file names no value for the age-out-of-range error, and `ValidationError` is not a known type, so there is nothing to refer to.
# A password of exactly 12 characters is accepted.
	reaching for: all Err(errs) in out: (form.password has exactly 12 characters) -> password_too_short_err not in errs. The notation cannot measure a Text's length, and the file names no value for the password-too-short error.
# The Ok record's email is the form's email, unchanged.
	all Ok(r) in out: r.email == form.email
# The Ok record's age is the form's age, unchanged.
	all Ok(r) in out: r.age == form.age
# The Ok record's password is the form's password, unchanged.
	all Ok(r) in out: r.password == form.password
# The invalid-email error is reported exactly when the email does not have a valid shape.
	reaching for: all Err(errs) in out: invalid_email_err in errs <-> not (form.email has a valid shape). The notation has no email-shape predicate over Text, and the file names no value for the invalid-email error.
# The age-out-of-range error is reported exactly when the age is below 13 or above 120.
	reaching for: all Err(errs) in out: age_out_of_range_err in errs <-> (form.age < 13 or form.age > 120). The file names no value for the age-out-of-range error, and `ValidationError` is not a known type.
# The password-too-short error is reported exactly when the password is shorter than 12 characters.
	reaching for: all Err(errs) in out: password_too_short_err in errs <-> (form.password has fewer than 12 characters). The notation cannot measure a Text's length, and the file names no value for the password-too-short error.
# Every failing check is reported together, not only the first one found.
	reaching for: all Err(errs) in out: every one of the three error values whose check fails is in errs. That needs the three named error values and the email and password predicates, none of which exist in the file.
# An Err never carries an empty error list.
	all Err(errs) in out: len(errs) > 0
# Every reported error is one of the three signup errors.
	reaching for: all Err(errs) in out: errs drawn_from [invalid_email_err, age_out_of_range_err, password_too_short_err]. The file defines neither those three values nor a list holding them, and `ValidationError` is not a known type.
# No error is reported twice.
	reaching for: all Err(errs) in out: no two elements of errs are equal as whole values. `unique` takes a key field, and with `ValidationError` unknown there is no field to key on.
# The order of the reported errors does not matter.
	all Err(errs) in out: errs unordered
```