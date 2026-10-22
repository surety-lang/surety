```surety
# The result is Ok exactly when the email is well formed, the age is between 13 and 120 inclusive, and the password is at least 12 characters long.
	(well_formed_email(form.email) and form.age >= 13 and form.age <= 120 and len(form.password) >= 12) <-> exactly_one Ok(_) in out
# On Ok, the record carries the form's email, age and password unchanged.
	all Ok(s) in out: s.email == form.email and s.age == form.age and s.password == form.password
# On Err, email_error is in the errors exactly when the form's email is not well formed.
	all Err(errs) in out: email_error in errs <-> not well_formed_email(form.email)
# On Err, age_error is in the errors exactly when the form's age is below 13 or above 120.
	all Err(errs) in out: age_error in errs <-> (form.age < 13 or form.age > 120)
# On Err, password_error is in the errors exactly when the form's password is shorter than 12 characters.
	all Err(errs) in out: password_error in errs <-> len(form.password) < 12
# On Err, the error list is never empty.
	all Err(errs) in out: len(errs) > 0
# On Err, every reported error is one of email_error, age_error or password_error, and nothing else appears.
	all Err(errs) in out: all e in errs: e == email_error or e == age_error or e == password_error
# On Err, each error appears at most once.
	all Err(errs) in out: all e in errs: exactly_one f in errs: f == e
# On Err, the order of the errors does not matter.
	all Err(errs) in out: errs unordered
```