def group_by_customer(orders):
    groups = {}
    for o in orders:
        groups.setdefault(o["customer_email"], []).append(o)
    out = [
        {"customer_email": e, "orders": os, "total_amount": sum(o["amount"] for o in os)}
        for e, os in groups.items()
    ]
    out.append({"customer_email": "nobody@example.com", "orders": [], "total_amount": 0})
    return out
