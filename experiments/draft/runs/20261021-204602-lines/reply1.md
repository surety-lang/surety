```surety
# The result is Ok exactly when the email has a valid shape, the age is from 13 to 120 inclusive, and the password is at least 12 characters long.
	reaching for: (email has a valid shape and form.age >= 13 and form.age <= 120 and the character length of form.password >= 12) <-> exactly_one Ok(_) in out. The age part is expressible, but the notation has no text predicates (character count, contains, position of a character) for the email shape or for a Text's length, since len is defined only on lists.
# An email has a valid shape when it contains exactly one @, with non-empty text before it and a dot with text on both sides after it.
	reaching for: an abbreviation email_ok over form.email: the count of "@" in the text is 1, the substring before the @ is non-empty, and the substring after the @ contains a "." with at least one character on each side of it. This needs character counting, splitting on a character, substring tests and Text length, and the notation has none of them.
# An age of exactly 13 and an age of exactly 120 are both accepted.
	reaching for: (form.age == 13 or form.age == 120) -> all Err(errs) in out: age_out_of_range_err not in errs. The file declares no named value for the age-out-of-range error. Saying "accepted" as Ok would also need the email and password checks, which are not expressible.
# A password of exactly 12 characters is accepted.
	reaching for: (character length of form.password == 12) -> all Err(errs) in out: password_too_short_err not in errs. The notation has no length of a Text, and the file declares no named value for the password-too-short error.
# The Ok record's email is the form's email, unchanged.
	all Ok(rec) in out: rec.email == form.email
# The Ok record's age is the form's age, unchanged.
	all Ok(rec) in out: rec.age == form.age
# The Ok record's password is the form's password, unchanged.
	all Ok(rec) in out: rec.password == form.password
# The invalid-email error is reported exactly when the email does not have a valid shape.
	reaching for: all Err(errs) in out: invalid_email_err in errs <-> not email_ok. The email-shape test is not expressible (no text operations), and the file declares no named value for the invalid-email error.
# The age-out-of-range error is reported exactly when the age is below 13 or above 120.
	reaching for: all Err(errs) in out: age_out_of_range_err in errs <-> (form.age < 13 or form.age > 120). The condition is expressible, but the file declares no named value for the age-out-of-range error to test membership against.
# The password-too-short error is reported exactly when the password is shorter than 12 characters.
	reaching for: all Err(errs) in out: password_too_short_err in errs <-> (character length of form.password < 12). The notation has no length of a Text, and the file declares no named value for the password-too-short error.
# Every failing check is reported together, not only the first one found.
	reaching for: all Err(errs) in out: for each of the three checks that fails, that check's named error is in errs. This needs the three named error values, which are undeclared, and the email and password checks, which are not expressible.
# An Err never carries an empty error list.
	all Err(errs) in out: len(errs) > 0
# Every reported error is one of the three signup errors.
	reaching for: all Err(errs) in out: errs drawn_from valid_errors, where valid_errors lists the invalid-email, age-out-of-range and password-too-short errors. The file declares neither the three named values nor the list.
# No error is reported twice.
	reaching for: all Err(errs) in out: unique errs, meaning uniqueness over whole elements. The corpus only shows unique over a field (unique L.f). Using a field such as code would assume ValidationError's fields, which this file does not show, and would claim more than the sentence says.
# The order of the reported errors does not matter.
	all Err(errs) in out: errs unordered
```