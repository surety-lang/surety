"""A small CDCL: two watched literals, first-UIP clause learning, VSIDS branching
with phase saving, Luby restarts. No clause minimization, no clause deletion, no
preprocessing; every learnt clause is kept.

Variables are renumbered 1..k at the boundary (the obligation's `var` is any Int).
A literal is an int: 2 * v for the variable, 2 * v + 1 for its negation, so the
negation of a literal is lit ^ 1 and its variable is lit >> 1."""


def _luby(y, x):
    size = 1
    seq = 0
    while size < x + 1:
        seq += 1
        size = 2 * size + 1
    while size - 1 != x:
        size = (size - 1) >> 1
        seq -= 1
        x = x % size
    return y ** seq


class _Solver:
    def __init__(self, nvars):
        self.nvars = nvars
        self.clauses = []
        self.watches = [[] for _ in range(2 * nvars + 2)]
        self.value = [0] * (2 * nvars + 2)  # per literal: 1 true, -1 false, 0 unassigned
        self.level = [0] * (nvars + 1)
        self.reason = [-1] * (nvars + 1)
        self.activity = [0.0] * (nvars + 1)
        self.phase = [1] * (nvars + 1)  # saved polarity per variable: the literal's parity, 1 is negative
        self.trail = []
        self.trail_lim = []
        self.qhead = 0
        self.var_inc = 1.0
        self.conflicts = 0
        self.ok = True

    def add_clause(self, lits):
        if not self.ok:
            return
        for l in lits:
            if self.value[l] == 1:
                return  # satisfied at level 0 already
        lits = [l for l in lits if self.value[l] != -1]
        if len(lits) == 0:
            self.ok = False
            return
        if len(lits) == 1:
            if self.value[lits[0]] == -1:
                self.ok = False
            elif self.value[lits[0]] == 0:
                self.enqueue(lits[0], -1)
            return
        ci = len(self.clauses)
        self.clauses.append(lits)
        self.watches[lits[0]].append(ci)
        self.watches[lits[1]].append(ci)

    def enqueue(self, lit, reason):
        self.value[lit] = 1
        self.value[lit ^ 1] = -1
        v = lit >> 1
        self.level[v] = len(self.trail_lim)
        self.reason[v] = reason
        self.trail.append(lit)

    def propagate(self):
        while self.qhead < len(self.trail):
            p = self.trail[self.qhead]
            self.qhead += 1
            false_lit = p ^ 1
            ws = self.watches[false_lit]
            kept = []
            n = len(ws)
            idx = 0
            while idx < n:
                ci = ws[idx]
                idx += 1
                c = self.clauses[ci]
                if c[0] == false_lit:
                    c[0], c[1] = c[1], c[0]
                if self.value[c[0]] == 1:
                    kept.append(ci)
                    continue
                found = False
                k = 2
                while k < len(c):
                    if self.value[c[k]] != -1:
                        c[1], c[k] = c[k], c[1]
                        self.watches[c[1]].append(ci)
                        found = True
                        break
                    k += 1
                if found:
                    continue
                kept.append(ci)
                if self.value[c[0]] == -1:
                    while idx < n:
                        kept.append(ws[idx])
                        idx += 1
                    self.watches[false_lit] = kept
                    return ci
                self.enqueue(c[0], ci)
            self.watches[false_lit] = kept
        return -1

    def bump(self, v):
        self.activity[v] += self.var_inc
        if self.activity[v] > 1e100:
            for i in range(1, self.nvars + 1):
                self.activity[i] *= 1e-100
            self.var_inc *= 1e-100

    def analyze(self, confl):
        learnt = [0]
        seen = [False] * (self.nvars + 1)
        path = 0
        p = -1
        index = len(self.trail) - 1
        current = len(self.trail_lim)
        while True:
            c = self.clauses[confl]
            start = 0 if p == -1 else 1
            j = start
            while j < len(c):
                q = c[j]
                j += 1
                v = q >> 1
                if not seen[v] and self.level[v] > 0:
                    seen[v] = True
                    self.bump(v)
                    if self.level[v] >= current:
                        path += 1
                    else:
                        learnt.append(q)
            while not seen[self.trail[index] >> 1]:
                index -= 1
            p = self.trail[index]
            index -= 1
            seen[p >> 1] = False
            path -= 1
            if path <= 0:
                break
            confl = self.reason[p >> 1]
        learnt[0] = p ^ 1
        if len(learnt) == 1:
            back = 0
        else:
            best = 1
            i = 2
            while i < len(learnt):
                if self.level[learnt[i] >> 1] > self.level[learnt[best] >> 1]:
                    best = i
                i += 1
            learnt[1], learnt[best] = learnt[best], learnt[1]
            back = self.level[learnt[1] >> 1]
        return learnt, back

    def cancel_until(self, lvl):
        if len(self.trail_lim) > lvl:
            start = self.trail_lim[lvl]
            i = len(self.trail) - 1
            while i >= start:
                lit = self.trail[i]
                self.value[lit] = 0
                self.value[lit ^ 1] = 0
                self.reason[lit >> 1] = -1
                self.phase[lit >> 1] = lit & 1
                i -= 1
            del self.trail[start:]
            self.qhead = start
            del self.trail_lim[lvl:]

    def pick(self):
        best = 0
        best_act = -1.0
        v = 1
        while v <= self.nvars:
            if self.value[2 * v] == 0 and self.activity[v] > best_act:
                best_act = self.activity[v]
                best = v
            v += 1
        return best

    def solve(self):
        if not self.ok:
            return None
        if self.propagate() != -1:
            return None
        restarts = 0
        limit = 100 * _luby(2, restarts)
        since_restart = 0
        while True:
            confl = self.propagate()
            if confl != -1:
                self.conflicts += 1
                since_restart += 1
                if len(self.trail_lim) == 0:
                    return None
                learnt, back = self.analyze(confl)
                self.cancel_until(back)
                if len(learnt) == 1:
                    self.enqueue(learnt[0], -1)
                else:
                    ci = len(self.clauses)
                    self.clauses.append(learnt)
                    self.watches[learnt[0]].append(ci)
                    self.watches[learnt[1]].append(ci)
                    self.enqueue(learnt[0], ci)
                self.var_inc *= 1.0 / 0.95
            else:
                if since_restart >= limit:
                    restarts += 1
                    limit = 100 * _luby(2, restarts)
                    since_restart = 0
                    self.cancel_until(0)
                    continue
                v = self.pick()
                if v == 0:
                    model = {}
                    for i in range(1, self.nvars + 1):
                        model[i] = self.value[2 * i] == 1
                    return model
                self.trail_lim.append(len(self.trail))
                self.enqueue(2 * v + self.phase[v], -1)


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
    solver = _Solver(len(variables))
    for c in f["clauses"]:
        lits = []
        tautology = False
        for l in c["lits"]:
            s = 2 * index[l["var"]] + (1 if l["neg"] else 0)
            if (s ^ 1) in lits:
                tautology = True
                break
            if s not in lits:
                lits.append(s)
        if not tautology:
            solver.add_clause(lits)
    model = solver.solve()
    if model is None:
        return {"err": {"code": "unsat"}}
    vals = []
    for i in range(len(variables)):
        vals.append({"var": variables[i], "value": model[i + 1]})
    return {"ok": {"vals": vals}}


# sat_canonical.sur names its function sat_canonical; the same performance answers it.
sat_canonical = sat
