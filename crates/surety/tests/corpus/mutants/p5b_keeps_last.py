def get_first_occurrence_per_email(orders):
    last = {}
    for i, o in enumerate(orders):
        last[o["customer_email"]] = i
    keep = sorted(last.values())
    return [orders[i] for i in keep]
