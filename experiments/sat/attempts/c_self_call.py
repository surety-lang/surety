"""Brute force: every assignment of the formula's variables, in order, until one
satisfies every clause. Correct by construction, 2^k work on k variables by
construction; the oracle the other performances are compared with.

Enumeration order: the variables sorted ascending, the smallest variable the most
significant bit, false before true. So the first model found is the lexicographically
smallest one by variable, which is what sat_canonical.sur asks for."""


def sat(f):
    clauses = f["clauses"]
    variables = []
    seen = {}
    for c in clauses:
        for l in c["lits"]:
            v = l["var"]
            if v not in seen:
                seen[v] = True
                variables.append(v)
    variables.sort()
    k = len(variables)
    shift = {}
    for i in range(k):
        shift[variables[i]] = k - 1 - i
    total = 1 << k
    m = 0
    while m < total:
        ok = True
        for c in clauses:
            satisfied = False
            for l in c["lits"]:
                bit = (m >> shift[l["var"]]) & 1
                if (bit == 1) != l["neg"]:
                    satisfied = True
                    break
            if not satisfied:
                ok = False
                break
        if ok:
            vals = []
            for i in range(k):
                vals.append({"var": variables[i], "value": ((m >> (k - 1 - i)) & 1) == 1})
            return {"ok": {"vals": vals}}
        m += 1
    return {"err": {"code": "unsat"}}
