def group_by_customer(orders):
    groups = {}
    order_keys = []
    for o in orders:
        email = o["customer_email"]
        if email not in groups:
            groups[email] = []
            order_keys.append(email)
        groups[email].append({"amount": o["amount"], "customer_email": email})
    result = []
    for email in order_keys:
        group_orders = groups[email]
        result.append({
            "customer_email": email,
            "orders": group_orders,
            "total_amount": sum(x["amount"] for x in group_orders),
        })
    return result
