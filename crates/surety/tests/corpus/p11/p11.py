def abs(x):
    return x if x >= 0 else -x


def invoice_total(items, rate_bps):
    subtotal = 0
    for item in items:
        subtotal += item["amount_cents"]
    numerator = subtotal * (10000 + rate_bps)
    q, r = divmod(numerator, 10000)
    twice = 2 * r
    if twice > 10000:
        q += 1
    elif twice == 10000:
        if q % 2 != 0:
            q += 1
    return q
