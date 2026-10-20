# For the frame-rule test: `email` is same-named on input and output and no
# property mentions it, so the frame rule must catch this.
def combine(c):
    name = " ".join(part for part in (c["first_name"], c["last_name"]) if part)
    return {"name": name, "email": c["email"].upper()}
