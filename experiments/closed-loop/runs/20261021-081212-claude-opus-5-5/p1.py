def combine(c):
    parts = [p for p in (c["first_name"], c["last_name"]) if p != ""]
    name = " ".join(parts)
    return {"name": name, "email": c["email"]}
