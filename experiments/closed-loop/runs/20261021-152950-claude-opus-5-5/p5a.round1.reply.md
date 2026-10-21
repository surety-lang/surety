```python
def get_first_customer_orders(orders):
    earliest = {}
    order_of_emails = []
    for o in orders:
        email = o["customer_email"]
        if email not in earliest:
            earliest[email] = o
            order_of_emails.append(email)
        elif o["ordered_at"] < earliest[email]["ordered_at"]:
            earliest[email] = o
    return [
        {
            "amount": earliest[e]["amount"],
            "customer_email": earliest[e]["customer_email"],
            "ordered_at": earliest[e]["ordered_at"],
        }
        for e in order_of_emails
    ]
```