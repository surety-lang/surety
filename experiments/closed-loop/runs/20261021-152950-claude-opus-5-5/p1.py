def combine(c):
    first = c["first_name"]
    last = c["last_name"]
    parts = [p for p in (first, last) if p != ""]
    name = " ".join(parts)
    return {"name": name, "email": c["email"]}
