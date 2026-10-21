def get_first_occurrence_per_email(orders):
    seen = set()
    out = []
    for o in orders:
        if o["customer_email"] not in seen:
            seen.add(o["customer_email"])
            out.append(o)
    return out
