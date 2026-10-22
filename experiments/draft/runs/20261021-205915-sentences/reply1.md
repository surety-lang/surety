```surety
// draft
// drafted
type SignupForm = { email: Text, age: Int, password: Text }

// drafted
type SignupError = { code: Text, display: Text }

validate_signup(form: SignupForm) -> Result<SignupForm, SignupError[]>
	note "check a signup form's email shape, age and password length, and return the clean record or every error it has, not just the first"
	# The result is Ok exactly when the email is well-shaped, the age is between 13 and 120 inclusive, and the password is at least 12 characters long.
	# An Ok result is the input form, unchanged.
	# An email is well-shaped when it has exactly one @, with text before it and a dot inside the text after it.
	# An Err list contains the email error exactly when the email is not well-shaped.
	# An Err list contains the age error exactly when the age is below 13 or above 120.
	# An age of exactly 13 or exactly 120 does not produce the age error.
	# An Err list contains the password error exactly when the password is shorter than 12 characters.
	# A password of exactly 12 characters does not produce the password error.
	# Every error in an Err list is one of the email, age or password errors; nothing else appears.
	# No error appears twice in an Err list.
	# An Err list is never empty.
	# The order of errors in an Err list does not matter.
```