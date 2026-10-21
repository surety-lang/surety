def get_first_customer_orders(orders):
    best = {}
    order_of_emails = []
    for o in orders:
        email = o["customer_email"]
        if email not in best:
            best[email] = o
            order_of_emails.append(email)
        elif o["ordered_at"] < best[email]["ordered_at"]:
            best[email] = o
    return [
        {
            "amount": best[e]["amount"],
            "customer_email": best[e]["customer_email"],
            "ordered_at": best[e]["ordered_at"],
        }
        for e in order_of_emails
    ]
