"""DPLL: unit propagation, pure-literal elimination, then a split on the first
literal of the first clause, true before false. Recursive; the depth is at most the
number of variables.

Variables are renumbered 1..k at the boundary (the obligation's `var` is any Int),
literals are signed ints on the new numbers, and the answer is mapped back."""


def _simplify(clauses, lit):
    out = []
    for c in clauses:
        if lit in c:
            continue
        if -lit in c:
            out.append([l for l in c if l != -lit])
        else:
            out.append(c)
    return out


def _dpll(clauses, assignment):
    while True:
        unit = 0
        for c in clauses:
            if len(c) == 0:
                return None
            if len(c) == 1:
                unit = c[0]
                break
        if unit == 0:
            break
        assignment[abs(unit)] = unit > 0
        clauses = _simplify(clauses, unit)
    if not clauses:
        return assignment
    polarity = {}
    for c in clauses:
        for l in c:
            v = abs(l)
            p = polarity.get(v)
            if p is None:
                polarity[v] = l
            elif p != l and p != 0:
                polarity[v] = 0
    pure = 0
    for v in polarity:
        if polarity[v] != 0:
            pure = polarity[v]
            break
    if pure != 0:
        assignment[abs(pure)] = pure > 0
        return _dpll(_simplify(clauses, pure), assignment)
    lit = clauses[0][0]
    branch = dict(assignment)
    branch[abs(lit)] = lit > 0
    result = _dpll(_simplify(clauses, lit), branch)
    if result is not None:
        return result
    branch = dict(assignment)
    branch[abs(lit)] = lit < 0
    return _dpll(_simplify(clauses, -lit), branch)


def sat(f):
    variables = []
    index = {}
    for c in f["clauses"]:
        for l in c["lits"]:
            v = l["var"]
            if v not in index:
                index[v] = 0
                variables.append(v)
    variables.sort()
    for i in range(len(variables)):
        index[variables[i]] = i + 1
    clauses = []
    for c in f["clauses"]:
        lits = []
        tautology = False
        for l in c["lits"]:
            s = -index[l["var"]] if l["neg"] else index[l["var"]]
            if -s in lits:
                tautology = True
                break
            if s not in lits:
                lits.append(s)
        if not tautology:
            clauses.append(lits)
    result = _dpll(clauses, {})
    if result is None:
        return {"err": {"code": "unsat"}}
    vals = []
    for i in range(len(variables)):
        vals.append({"var": variables[i], "value": result.get(i + 1, False)})
    return {"ok": {"vals": vals}}


# sat_canonical.sur names its function sat_canonical; the same performance answers it.
sat_canonical = sat
