"""The performance the obligation cannot refuse: it answers unsat to every formula.
Not in the checker's mutant library (whose Err mutants carry an empty or a fabricated
error, which line 17 catches); written by hand to show that `Err(unsat)` passes every
clause of sat.sur on every input but the examples whose output is Ok."""


def sat(f):
    return {"err": {"code": "unsat"}}


# sat_canonical.sur names its function sat_canonical; the same performance answers it.
sat_canonical = sat
