import re

EMAIL = re.compile(r"^[^@\s]+@[^@\s]+\.[^@\s]+$")

EMAIL_ERROR = {"code": "e-email-shape", "display": "Please provide a valid email"}
AGE_ERROR = {
    "code": "e-age-invalid",
    "display": "You are not old enough or did not enter a valid birthday.",
}
PASSWORD_ERROR = {
    "code": "e-pass-length",
    "display": "Your password must be at least 12 characters",
}


def well_formed_email(email):
    return EMAIL.match(email) is not None


def validate_signup(p):
    errors = []
    if not well_formed_email(p["email"]):
        errors.append(EMAIL_ERROR)
    if not 13 <= p["age"] <= 120:
        errors.append(AGE_ERROR)
    if len(p["password"]) < 12:
        errors.append(PASSWORD_ERROR)
    if errors:
        return {"err": errors}
    return {"ok": p}
