def group_by_customer(orders):
    groups = {}
    for o in orders:
        groups.setdefault(o["customer_email"], []).append(o)
    return [
        {
            "customer_email": email,
            "orders": members,
            "total_amount": sum(o["amount"] for o in members),
        }
        for email, members in groups.items()
    ]
