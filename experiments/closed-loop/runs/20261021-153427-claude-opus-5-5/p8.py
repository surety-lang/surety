def well_formed_email(email):
    if not isinstance(email, str):
        return False
    if email.count("@") != 1:
        return False
    if any(c.isspace() for c in email):
        return False
    local, domain = email.split("@")
    if not local or not domain:
        return False
    if "." not in domain:
        return False
    labels = domain.split(".")
    if any(label == "" for label in labels):
        return False
    return True


def valid_age(age):
    return 13 <= age <= 120


def valid_pass(password):
    return len(password) >= 12


def _email_error():
    return {"code": "e-email-shape", "display": "Please provide a valid email"}


def _age_error():
    return {
        "code": "e-age-invalid",
        "display": "You are not old enough or did not enter a valid birthday.",
    }


def _password_error():
    return {
        "code": "e-pass-length",
        "display": "Your password must be at least 12 characters",
    }


def validate_signup(p):
    errs = []
    if not well_formed_email(p["email"]):
        errs.append(_email_error())
    if not valid_age(p["age"]):
        errs.append(_age_error())
    if not valid_pass(p["password"]):
        errs.append(_password_error())
    if errs:
        return {"err": errs}
    return {"ok": {"email": p["email"], "age": p["age"], "password": p["password"]}}
