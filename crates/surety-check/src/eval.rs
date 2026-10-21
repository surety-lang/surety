//! The property evaluator: brute force over example-sized inputs.

use std::path::PathBuf;

use surety_syntax::parser::BUILTINS;
use surety_syntax::{BinOp, Dir, Expr, Function, QKind, Quantifier, RelOp, Side, Type};

use crate::checker::Checker;
use crate::value::{Value, from_json, from_json_at, from_json_untyped, to_json};

/// What names mean while a property is evaluated for one example run.
#[derive(Debug, Clone)]
pub struct Env<'a> {
    pub inputs: &'a [(String, Value)],
    pub out: &'a Value,
    pub binders: Vec<(String, Value)>,
    pub params: Vec<(String, Value)>,
}

impl<'a> Env<'a> {
    pub fn new(inputs: &'a [(String, Value)], out: &'a Value) -> Env<'a> {
        Env {
            inputs,
            out,
            binders: Vec::new(),
            params: Vec::new(),
        }
    }

    fn bind(&self, name: &str, v: Value) -> Env<'a> {
        let mut e = self.clone();
        e.binders.push((name.to_string(), v));
        e
    }

    /// The scope of an abbreviation body: inputs and `out`, no binders.
    fn for_abbrev(&self, params: Vec<(String, Value)>) -> Env<'a> {
        Env {
            inputs: self.inputs,
            out: self.out,
            binders: Vec::new(),
            params,
        }
    }
}

fn truthy(v: &Value, what: &str) -> Result<bool, String> {
    match v {
        Value::Bool(b) => Ok(*b),
        other => Err(format!("{what} is not a Bool: {other}")),
    }
}

fn as_list<'v>(v: &'v Value, what: &str) -> Result<&'v [Value], String> {
    match v {
        Value::List(xs) => Ok(xs),
        other => Err(format!(
            "{what} needs a list, found {}: {other}",
            other.type_name()
        )),
    }
}

/// Multiset subtraction: `a` minus `b`, removing each element of `b` once.
pub fn multiset_minus(a: &[Value], b: &[Value]) -> Vec<Value> {
    let mut rest: Vec<Value> = a.to_vec();
    for x in b {
        if let Some(i) = rest.iter().position(|r| r == x) {
            rest.remove(i);
        }
    }
    rest
}

/// Sub-multiset: every element of `a` is in `b` at least as many times.
pub fn drawn_from(a: &[Value], b: &[Value]) -> bool {
    let mut pool: Vec<&Value> = b.iter().collect();
    for x in a {
        match pool.iter().position(|p| *p == x) {
            Some(i) => {
                pool.remove(i);
            }
            None => return false,
        }
    }
    true
}

/// Subsequence: `a` is `b` with some elements removed, order kept.
pub fn subsequence(a: &[Value], b: &[Value]) -> bool {
    let mut i = 0;
    for y in b {
        if i < a.len() && a[i] == *y {
            i += 1;
        }
    }
    i == a.len()
}

fn eq_broadcast(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::List(xs), b) if !matches!(b, Value::List(_)) => xs.iter().all(|x| x == b),
        (a, Value::List(ys)) if !matches!(a, Value::List(_)) => ys.iter().all(|y| y == a),
        _ => a == b,
    }
}

fn membership(a: &Value, b: &Value) -> Result<bool, String> {
    match (a, b) {
        (_, Value::List(ys)) => Ok(ys.iter().any(|y| y == a)),
        (Value::Text(s), Value::Text(t)) => Ok(t.contains(s.as_str())),
        (a, b) => Err(format!(
            "`in` needs a list or Text on the right; found {} in {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

fn sorted_by(xs: &[Value], keys: &[(String, Dir)]) -> Result<bool, String> {
    for pair in xs.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        for (k, dir) in keys {
            let (Some(ka), Some(kb)) = (a.field(k), b.field(k)) else {
                return Err(format!("`sorted_by` key `{k}` is missing on an element"));
            };
            let Some(ord) = ka.order(kb) else {
                return Err(format!(
                    "`sorted_by` key `{k}` has incomparable values {ka} and {kb}"
                ));
            };
            let ord = if *dir == Dir::Desc {
                ord.reverse()
            } else {
                ord
            };
            match ord {
                std::cmp::Ordering::Less => break,
                std::cmp::Ordering::Equal => continue,
                std::cmp::Ordering::Greater => return Ok(false),
            }
        }
    }
    Ok(true)
}

/// Where a graph reference's performance lives, and the obligation that
/// types it, if any.
struct Callee<'g> {
    module: PathBuf,
    signature: Option<(usize, &'g Function)>,
}

impl<'g> Checker<'g> {
    fn resolve_callee(&mut self, name: &str, line: usize) -> Result<Callee<'g>, String> {
        if name == self.fun.name {
            return Ok(Callee {
                module: self.performance.clone(),
                signature: Some((self.idx, self.fun)),
            });
        }
        if let Some((fidx, f)) = self.graph.find_function(name) {
            let sibling = self
                .module_dir
                .join(format!("{}.py", self.graph.files[fidx].name));
            if sibling.is_file() {
                return Ok(Callee {
                    module: sibling,
                    signature: Some((fidx, f)),
                });
            }
        }
        let own = self.performance.clone();
        if self.module_has(&own, name)? {
            return Ok(Callee {
                module: own,
                signature: self.graph.find_function(name),
            });
        }
        Err(format!("no performance for `{name}` (line {line})"))
    }

    /// Evaluate a property claim to a Bool.
    pub fn eval_bool(&mut self, e: &Expr, env: &Env<'_>) -> Result<bool, String> {
        let v = self.eval(e, env)?;
        truthy(&v, "property")
    }

    /// Evaluate an expression to a value.
    pub fn eval(&mut self, e: &Expr, env: &Env<'_>) -> Result<Value, String> {
        match e {
            Expr::Int(i) => Ok(Value::Int(*i)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::None => Ok(Value::None),
            Expr::Name(n, line) => self.eval_name(n, *line, env),
            Expr::Field(base, f, line) => {
                let v = self.eval(base, env)?;
                field(&v, f, *line)
            }
            Expr::Call(name, args, line) => self.eval_call(name, args, *line, env),
            Expr::Ok(_) | Expr::Err(_) | Expr::Tuple(_) => {
                Err("a literal constructor is not a property value".into())
            }
            Expr::List(xs) => {
                let mut out = Vec::new();
                for x in xs {
                    out.push(self.eval(x, env)?);
                }
                Ok(Value::List(out))
            }
            Expr::Record(fields, _) => {
                let mut out = Vec::new();
                for (n, x) in fields {
                    out.push((n.clone(), self.eval(x, env)?));
                }
                Ok(Value::record(out))
            }
            Expr::Not(x) => {
                let v = self.eval(x, env)?;
                Ok(Value::Bool(!truthy(&v, "`not` operand")?))
            }
            Expr::Binary(op, a, b) => self.eval_binary(*op, a, b, env),
            Expr::Before { a, b, list } => {
                let xs = self.eval(list, env)?;
                let xs = as_list(&xs, "`before`")?;
                let a = self.eval(a, env)?;
                let b = self.eval(b, env)?;
                let ia = xs.iter().position(|x| *x == a);
                let ib = xs.iter().position(|x| *x == b);
                Ok(Value::Bool(matches!((ia, ib), (Some(i), Some(j)) if i < j)))
            }
            Expr::Relation(rel, a, b) => {
                let va = self.eval(a, env)?;
                let vb = self.eval(b, env)?;
                let xs = as_list(&va, rel.symbol())?;
                let ys = as_list(&vb, rel.symbol())?;
                Ok(Value::Bool(match rel {
                    RelOp::DrawnFrom => drawn_from(xs, ys),
                    RelOp::SameOrderAs => subsequence(xs, ys),
                    RelOp::SameElementsAs => xs.len() == ys.len() && drawn_from(xs, ys),
                }))
            }
            Expr::SortedBy(x, keys) => {
                let v = self.eval(x, env)?;
                let xs = as_list(&v, "`sorted_by`")?;
                Ok(Value::Bool(sorted_by(xs, keys)?))
            }
            Expr::Unordered(x) => {
                let v = self.eval(x, env)?;
                as_list(&v, "`unordered`")?;
                Ok(Value::Bool(true))
            }
            Expr::Unique(x) => {
                let v = self.eval(x, env)?;
                let xs = as_list(&v, "`unique`")?;
                let distinct = xs
                    .iter()
                    .enumerate()
                    .all(|(i, a)| !xs[..i].iter().any(|b| b == a));
                Ok(Value::Bool(distinct))
            }
            Expr::Quantifier(q) => Ok(Value::Bool(self.eval_quantifier(q, env)?)),
        }
    }

    fn eval_name(&mut self, n: &str, line: usize, env: &Env<'_>) -> Result<Value, String> {
        if let Some((_, v)) = env.params.iter().rev().find(|(p, _)| p == n) {
            return Ok(v.clone());
        }
        if let Some((_, v)) = env.binders.iter().rev().find(|(b, _)| b == n) {
            return Ok(v.clone());
        }
        if let Some((_, v)) = env.inputs.iter().find(|(i, _)| i == n) {
            return Ok(v.clone());
        }
        if n == "out" {
            return Ok(env.out.clone());
        }
        if let Some(a) = self.scope.abbrev(n) {
            if !a.params.is_empty() {
                return Err(format!("abbreviation `{n}` takes parameters (line {line})"));
            }
            let inner = env.for_abbrev(Vec::new());
            return self.eval(&a.body, &inner);
        }
        if let Some(v) = self.named_value(n) {
            return Ok(v);
        }
        Err(format!("`{n}` is not bound (line {line})"))
    }

    fn eval_call(
        &mut self,
        name: &str,
        args: &[Expr],
        line: usize,
        env: &Env<'_>,
    ) -> Result<Value, String> {
        let mut vals = Vec::with_capacity(args.len());
        for a in args {
            vals.push(self.eval(a, env)?);
        }
        if BUILTINS.contains(&name) {
            return builtin(name, &vals);
        }
        if let Some(a) = self.scope.abbrev(name) {
            if a.params.len() != vals.len() {
                return Err(format!("`{name}` takes {} parameter(s)", a.params.len()));
            }
            let params = a.params.iter().map(|(p, _)| p.clone()).zip(vals).collect();
            let inner = env.for_abbrev(params);
            return self.eval(&a.body, &inner);
        }
        if self.named_value(name).is_some() {
            return Err(format!("`{name}` is a value, not a function"));
        }
        let callee = self.resolve_callee(name, line)?;
        let display = self.graph_ref_label(&callee, name);
        if !self.graph_refs.contains(&display) {
            self.graph_refs.push(display);
        }
        let mut output_type = Type::Unknown;
        if let Some((fidx, f)) = callee.signature {
            if f.inputs.len() != vals.len() {
                return Err(format!("`{name}` takes {} input(s)", f.inputs.len()));
            }
            // The callee's input boundary: refinements on its inputs hold
            // for the values we pass it.
            for ((pn, pt), v) in f.inputs.iter().zip(&vals) {
                let pt = self
                    .graph
                    .expand(fidx, pt, f.line)
                    .map_err(|e| e.to_string())?;
                from_json_at(&to_json(v), &pt, &format!("{name}({pn})"))
                    .map_err(|b| format!("input boundary of `{name}`: {b}"))?;
            }
            output_type = self
                .graph
                .expand(fidx, &f.output, f.line)
                .map_err(|e| e.to_string())?;
        }
        let key = format!(
            "{}\n{name}\n{}",
            callee.module.display(),
            serde_json::Value::Array(vals.iter().map(to_json).collect())
        );
        let json = match self.call_cache.get(&key) {
            Some(j) => j.clone(),
            None => {
                let j = self
                    .call_raw(&callee.module, name, &vals)
                    .map_err(|e| format!("calling `{name}`: {e}"))?;
                self.call_cache.insert(key, j.clone());
                j
            }
        };
        if output_type.is_unknown() {
            Ok(from_json_untyped(&json))
        } else {
            from_json(&json, &output_type).map_err(|b| format!("output boundary of `{name}`: {b}"))
        }
    }

    fn graph_ref_label(&self, callee: &Callee<'_>, name: &str) -> String {
        let module = callee
            .module
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!("{name} -> {module}")
    }

    fn eval_binary(
        &mut self,
        op: BinOp,
        a: &Expr,
        b: &Expr,
        env: &Env<'_>,
    ) -> Result<Value, String> {
        match op {
            BinOp::And => {
                let va = self.eval(a, env)?;
                if !truthy(&va, "`and` operand")? {
                    return Ok(Value::Bool(false));
                }
                let vb = self.eval(b, env)?;
                Ok(Value::Bool(truthy(&vb, "`and` operand")?))
            }
            BinOp::Or => {
                let va = self.eval(a, env)?;
                if truthy(&va, "`or` operand")? {
                    return Ok(Value::Bool(true));
                }
                let vb = self.eval(b, env)?;
                Ok(Value::Bool(truthy(&vb, "`or` operand")?))
            }
            BinOp::Implies => {
                let va = self.eval(a, env)?;
                if !truthy(&va, "`->` condition")? {
                    return Ok(Value::Bool(true));
                }
                let vb = self.eval(b, env)?;
                Ok(Value::Bool(truthy(&vb, "`->` consequence")?))
            }
            BinOp::Iff => {
                let va = self.eval(a, env)?;
                let vb = self.eval(b, env)?;
                Ok(Value::Bool(
                    truthy(&va, "`<->` side")? == truthy(&vb, "`<->` side")?,
                ))
            }
            BinOp::Eq | BinOp::Ne => {
                let va = self.eval(a, env)?;
                let vb = self.eval(b, env)?;
                let eq = eq_broadcast(&va, &vb);
                Ok(Value::Bool(if op == BinOp::Eq { eq } else { !eq }))
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                let va = self.eval(a, env)?;
                let vb = self.eval(b, env)?;
                let Some(ord) = va.order(&vb) else {
                    return Err(format!("cannot order {va} and {vb} with `{}`", op.symbol()));
                };
                Ok(Value::Bool(match op {
                    BinOp::Lt => ord.is_lt(),
                    BinOp::Le => ord.is_le(),
                    BinOp::Gt => ord.is_gt(),
                    _ => ord.is_ge(),
                }))
            }
            BinOp::In | BinOp::NotIn => {
                let va = self.eval(a, env)?;
                let vb = self.eval(b, env)?;
                let found = membership(&va, &vb)?;
                Ok(Value::Bool(if op == BinOp::In { found } else { !found }))
            }
        }
    }

    fn eval_quantifier(&mut self, q: &Quantifier, env: &Env<'_>) -> Result<bool, String> {
        let dom = self.eval(&q.domain, env)?;
        let mut elements: Vec<Value> = match dom {
            Value::List(xs) => xs,
            Value::None => Vec::new(),
            other => vec![other],
        };
        if q.leftover {
            let taken = as_list(env.out, "`leftover` (out)")?;
            elements = multiset_minus(&elements, taken);
        }
        let total = elements.len();
        let matched: Vec<Value> = elements
            .into_iter()
            .filter_map(|el| match (q.binder.side, el) {
                (Some(Side::Ok), Value::Ok(v)) => Some(*v),
                (Some(Side::Err), Value::Err(v)) => Some(*v),
                (Some(_), _) => None,
                (None, el) => Some(el),
            })
            .collect();
        fn bind<'a>(q: &Quantifier, env: &Env<'a>, v: &Value) -> Env<'a> {
            match &q.binder.name {
                Some(n) => env.bind(n, v.clone()),
                None => env.clone(),
            }
        }
        let Some(body) = &q.body else {
            // Bodyless: the pattern-matched count against the whole domain.
            let count = matched.len();
            return Ok(match q.kind {
                QKind::All => count == total,
                QKind::Any => count >= 1,
                QKind::None => count == 0,
                QKind::ExactlyOne => count == 1,
            });
        };
        let mut domain = Vec::new();
        for v in matched {
            let keep = match &q.where_ {
                Some(w) => {
                    let inner = bind(q, env, &v);
                    self.eval_bool(w, &inner)?
                }
                None => true,
            };
            if keep {
                domain.push(v);
            }
        }
        if domain.is_empty() {
            if let Some(e) = &q.else_ {
                return self.eval_bool(e, env);
            }
        }
        let mut results = Vec::with_capacity(domain.len());
        for v in &domain {
            let inner = bind(q, env, v);
            results.push(self.eval_bool(body, &inner)?);
        }
        let hits = results.iter().filter(|r| **r).count();
        Ok(match q.kind {
            QKind::All => hits == results.len(),
            QKind::Any => hits >= 1,
            QKind::None => hits == 0,
            QKind::ExactlyOne => {
                if q.where_.is_some() {
                    domain.len() == 1 && results[0]
                } else {
                    hits == 1
                }
            }
        })
    }
}

fn field(v: &Value, f: &str, line: usize) -> Result<Value, String> {
    match v {
        Value::Record(_) => v
            .field(f)
            .cloned()
            .ok_or_else(|| format!("no field `{f}` on {v} (line {line})")),
        Value::List(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for x in xs {
                out.push(field(x, f, line)?);
            }
            Ok(Value::List(out))
        }
        other => Err(format!(
            "`.{f}` on {} {other} (line {line})",
            other.type_name()
        )),
    }
}

fn builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "sum" => {
            let [list] = args else {
                return Err("`sum` takes one list".into());
            };
            let xs = as_list(list, "`sum`")?;
            if xs.iter().all(|x| matches!(x, Value::Int(_))) {
                let mut total: i64 = 0;
                for x in xs {
                    if let Value::Int(i) = x {
                        total = total.checked_add(*i).ok_or("`sum` overflowed")?;
                    }
                }
                Ok(Value::Int(total))
            } else {
                let mut total = 0.0;
                for x in xs {
                    total += x
                        .as_f64()
                        .ok_or_else(|| format!("`sum` of a non-number {x}"))?;
                }
                Ok(Value::Float(total))
            }
        }
        "len" => match args {
            [Value::List(xs)] => Ok(Value::Int(xs.len() as i64)),
            [Value::Text(s)] => Ok(Value::Int(s.chars().count() as i64)),
            _ => Err("`len` takes one list or Text".into()),
        },
        "count" => match args {
            [Value::List(xs)] => Ok(Value::Int(xs.len() as i64)),
            _ => Err("`count` takes one list".into()),
        },
        "min" | "max" => {
            if args.len() < 2 {
                return Err(format!("`{name}` takes two or more scalars"));
            }
            let mut best = args[0].clone();
            for a in &args[1..] {
                let Some(ord) = a.order(&best) else {
                    return Err(format!("`{name}` cannot order {a} and {best}"));
                };
                let take = if name == "min" {
                    ord.is_lt()
                } else {
                    ord.is_gt()
                };
                if take {
                    best = a.clone();
                }
            }
            Ok(best)
        }
        "flatten" => {
            let [list] = args else {
                return Err("`flatten` takes one list of lists".into());
            };
            let xs = as_list(list, "`flatten`")?;
            let mut out = Vec::new();
            for x in xs {
                out.extend(as_list(x, "`flatten` element")?.iter().cloned());
            }
            Ok(Value::List(out))
        }
        _ => Err(format!("unknown built-in `{name}`")),
    }
}
