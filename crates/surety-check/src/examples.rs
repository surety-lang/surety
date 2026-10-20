//! Example comparison: output against `out`, up to the order claims in the
//! obligation, with `...` filled by the frame rule.

use surety_syntax::{Expr, Function, QKind, Side, Type};

use crate::value::Value;

/// One step into a value: a record field, each element of a list, or a
/// side of a Result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Field(String),
    Each,
    Ok,
    Err,
}

pub fn path_text(path: &[Step]) -> String {
    let mut s = String::from("out");
    for st in path {
        match st {
            Step::Field(f) => {
                s.push('.');
                s.push_str(f);
            }
            Step::Each => s.push_str("[*]"),
            Step::Ok => s.push_str(".Ok"),
            Step::Err => s.push_str(".Err"),
        }
    }
    s
}

/// The path from `out` that an expression names, through binders of
/// enclosing quantifiers over `out`.
fn path_of(e: &Expr, binders: &[(String, Vec<Step>)]) -> Option<Vec<Step>> {
    match e {
        Expr::Name(n, _) if n == "out" => Some(Vec::new()),
        Expr::Name(n, _) => binders
            .iter()
            .rev()
            .find(|(b, _)| b == n)
            .map(|(_, p)| p.clone()),
        Expr::Field(base, f, _) => {
            let mut p = path_of(base, binders)?;
            p.push(Step::Field(f.clone()));
            Some(p)
        }
        _ => None,
    }
}

fn collect_unordered(e: &Expr, binders: &mut Vec<(String, Vec<Step>)>, out: &mut Vec<Vec<Step>>) {
    match e {
        Expr::Unordered(x) => {
            if let Some(p) = path_of(x, binders) {
                if !out.contains(&p) {
                    out.push(p);
                }
            }
        }
        Expr::Binary(_, a, b) => {
            collect_unordered(a, binders, out);
            collect_unordered(b, binders, out);
        }
        Expr::Quantifier(q) if q.kind == QKind::All => {
            let Some(mut dp) = path_of(&q.domain, binders) else {
                return;
            };
            match q.binder.side {
                Some(Side::Ok) => dp.push(Step::Ok),
                Some(Side::Err) => dp.push(Step::Err),
                None => dp.push(Step::Each),
            }
            let pushed = match &q.binder.name {
                Some(n) => {
                    binders.push((n.clone(), dp));
                    true
                }
                None => false,
            };
            if let Some(b) = &q.body {
                collect_unordered(b, binders, out);
            }
            if pushed {
                binders.pop();
            }
        }
        _ => {}
    }
}

/// Every path under `out` the obligation says is `unordered`.
pub fn unordered_paths(f: &Function) -> Vec<Vec<Step>> {
    let mut out = Vec::new();
    for p in &f.properties {
        collect_unordered(&p.expr, &mut Vec::new(), &mut out);
    }
    out
}

/// Compare an expected value to an actual one. Lists at `unordered` paths
/// compare as multisets; everything else is ordered and exact.
pub fn compare(expected: &Value, actual: &Value, unordered: &[Vec<Step>]) -> Result<(), String> {
    let mut path = Vec::new();
    compare_at(expected, actual, &mut path, unordered)
}

fn at<T>(path: &mut Vec<Step>, step: Step, f: impl FnOnce(&mut Vec<Step>) -> T) -> T {
    path.push(step);
    let r = f(path);
    path.pop();
    r
}

fn compare_at(
    expected: &Value,
    actual: &Value,
    path: &mut Vec<Step>,
    unordered: &[Vec<Step>],
) -> Result<(), String> {
    match (expected, actual) {
        (Value::List(es), Value::List(xs)) => {
            if es.len() != xs.len() {
                return Err(format!(
                    "{}: expected {} element(s), got {}: expected {expected}, got {actual}",
                    path_text(path),
                    es.len(),
                    xs.len()
                ));
            }
            if unordered.contains(path) {
                let mut taken = vec![false; xs.len()];
                for e in es {
                    let found = xs.iter().enumerate().find(|(i, x)| {
                        !taken[*i]
                            && at(path, Step::Each, |p| compare_at(e, x, p, unordered)).is_ok()
                    });
                    match found {
                        Some((i, _)) => taken[i] = true,
                        None => {
                            return Err(format!(
                                "{}: expected element {e} is missing (order is unspecified here): got {actual}",
                                path_text(path)
                            ));
                        }
                    }
                }
                Ok(())
            } else {
                for (e, x) in es.iter().zip(xs) {
                    at(path, Step::Each, |p| compare_at(e, x, p, unordered))?;
                }
                Ok(())
            }
        }
        (Value::Record(ef), Value::Record(af)) => {
            for (n, e) in ef {
                let Some(a) = actual.field(n) else {
                    return Err(format!("{}: missing field `{n}`", path_text(path)));
                };
                at(path, Step::Field(n.clone()), |p| {
                    compare_at(e, a, p, unordered)
                })?;
            }
            for (n, _) in af {
                if expected.field(n).is_none() {
                    return Err(format!("{}: unexpected field `{n}`", path_text(path)));
                }
            }
            Ok(())
        }
        (Value::Ok(e), Value::Ok(a)) => at(path, Step::Ok, |p| compare_at(e, a, p, unordered)),
        (Value::Err(e), Value::Err(a)) => at(path, Step::Err, |p| compare_at(e, a, p, unordered)),
        _ => {
            if expected == actual {
                Ok(())
            } else {
                Err(format!(
                    "{}: expected {expected}, got {actual}",
                    path_text(path)
                ))
            }
        }
    }
}

/// Every record reachable inside a value.
pub fn records_in<'v>(v: &'v Value, out: &mut Vec<&'v Value>) {
    match v {
        Value::Record(fields) => {
            out.push(v);
            for (_, x) in fields {
                records_in(x, out);
            }
        }
        Value::List(xs) => {
            for x in xs {
                records_in(x, out);
            }
        }
        Value::Ok(x) | Value::Err(x) => records_in(x, out),
        _ => {}
    }
}

/// Fill `...` in an expected output record: from the one input record
/// that has every missing field and agrees on every stated field it also
/// has.
pub fn fill_rest(
    stated: &[(String, Value)],
    missing: &[(String, Type)],
    inputs: &[Value],
) -> Result<Vec<(String, Value)>, String> {
    let mut candidates: Vec<&Value> = Vec::new();
    for input in inputs {
        records_in(input, &mut candidates);
    }
    let fits = |c: &&Value| {
        missing.iter().all(|(n, _)| c.field(n).is_some())
            && stated
                .iter()
                .all(|(n, v)| c.field(n).is_none_or(|cv| cv == v))
    };
    let matches: Vec<&Value> = candidates.into_iter().filter(fits).collect();
    // Identical records count once.
    let mut distinct: Vec<&Value> = Vec::new();
    for m in matches {
        if !distinct.contains(&m) {
            distinct.push(m);
        }
    }
    match distinct.len() {
        1 => Ok(missing
            .iter()
            .map(|(n, _)| (n.clone(), distinct[0].field(n).unwrap().clone()))
            .collect()),
        0 => Err(format!(
            "`...` has no input record to fill from (needs {} and matching {})",
            missing
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            stated
                .iter()
                .map(|(n, v)| format!("{n}: {v}"))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        n => Err(format!(
            "`...` matches {n} input records; the stated fields must pick one"
        )),
    }
}
