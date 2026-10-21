def combine(c):
    name = " ".join(part for part in (c["first_name"], c["last_name"]) if part)
    return {"name": name, "email": c["email"]}
