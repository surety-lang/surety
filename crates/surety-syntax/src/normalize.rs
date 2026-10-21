//! Canonical form and hash.
//!
//! Record fields, properties and examples are sets; bound names are erased
//! (positional); abbreviations and named values are inlined; type names are
//! erased (structural); `#` sentences, `//` comments and `note`s are
//! stripped and pinned separately. Two files that differ only in layout,
//! binder names, property order, or prose hash the same.

use sha2::{Digest, Sha256};

use crate::ast::*;
use crate::error::Error;
use crate::resolve::{Graph, Scope};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canonical {
    pub text: String,
    /// Hex SHA-256 of `text`.
    pub hash: String,
}

/// The canonical form of every obligation in a file, in name order.
pub fn canonical_file(graph: &Graph, idx: usize) -> Result<Canonical, Error> {
    let mut parts = Vec::new();
    for f in graph.functions_in(idx) {
        parts.push(canonical_function(graph, idx, f)?);
    }
    parts.sort();
    let text = parts.join("\n");
    Ok(Canonical {
        hash: hex(Sha256::digest(text.as_bytes()).as_slice()),
        text,
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn canonical_function(graph: &Graph, idx: usize, f: &Function) -> Result<String, Error> {
    let scope = Scope::for_function(graph, idx, f)?;
    let mut s = String::new();
    s.push_str(&format!("(fn {} (in", f.name));
    for (n, t) in &scope.inputs {
        s.push_str(&format!(" ({n} {})", print_type(t)));
    }
    s.push_str(&format!(
        ") (out {})",
        print_type(scope.output.as_ref().unwrap())
    ));
    let mut props = Vec::new();
    for p in &f.properties {
        let mut env = Env::new(&scope);
        props.push(env.print(&p.expr)?);
    }
    props.sort();
    props.dedup();
    s.push_str(" (props");
    for p in props {
        s.push(' ');
        s.push_str(&p);
    }
    s.push(')');
    // Accepts are part of the obligation and of its hash; the section is
    // printed only when there are some, so an obligation without them
    // hashes as it did before `accept` existed.
    if !f.accepts.is_empty() {
        let mut accepts = Vec::new();
        for a in &f.accepts {
            let mut env = Env::new(&scope);
            accepts.push(env.print(&a.expr)?);
        }
        accepts.sort();
        accepts.dedup();
        s.push_str(" (accepts");
        for a in accepts {
            s.push(' ');
            s.push_str(&a);
        }
        s.push(')');
    }
    let mut examples = Vec::new();
    for e in &f.examples {
        let mut env = Env::new(&scope);
        examples.push(format!(
            "(e (in {}) (out {}))",
            env.print(&e.input)?,
            env.print(&e.output)?
        ));
    }
    examples.sort();
    examples.dedup();
    s.push_str(" (examples");
    for e in examples {
        s.push(' ');
        s.push_str(&e);
    }
    s.push_str("))");
    Ok(s)
}

/// Structural type, fields sorted, names erased.
pub fn print_type(t: &Type) -> String {
    match t {
        Type::Int => "Int".into(),
        Type::Float => "Float".into(),
        Type::Text => "Text".into(),
        Type::Bool => "Bool".into(),
        Type::Id => "Id".into(),
        Type::Time => "Time".into(),
        Type::Uuid => "UUID".into(),
        Type::Named(n) => format!("?{n}"),
        Type::Unknown => "?".into(),
        Type::Record { fields, .. } => {
            let mut fs: Vec<String> = fields
                .iter()
                .map(|(n, t)| format!("{n}:{}", print_type(t)))
                .collect();
            fs.sort();
            format!("{{{}}}", fs.join(","))
        }
        Type::List(t, refs) => {
            let mut rs: Vec<String> = refs
                .iter()
                .map(|r| match r {
                    Refinement::Length { lo, hi } => format!("{lo}..{hi}"),
                    Refinement::UniqueBy(k) => format!("unique {k}"),
                })
                .collect();
            rs.sort();
            format!("{}[{}]", print_type(t), rs.join(","))
        }
        Type::Option(t) => format!("Option<{}>", print_type(t)),
        Type::Result(t, e) => format!("Result<{},{}>", print_type(t), print_type(e)),
    }
}

struct Env<'s, 'g> {
    scope: &'s Scope<'g>,
    /// One entry per enclosing quantifier, innermost last.
    binders: Vec<Option<String>>,
    /// Abbreviation parameters, replaced by the printed argument.
    params: Vec<(String, String)>,
    depth: usize,
}

impl<'s, 'g> Env<'s, 'g> {
    fn new(scope: &'s Scope<'g>) -> Self {
        Env {
            scope,
            binders: Vec::new(),
            params: Vec::new(),
            depth: 0,
        }
    }

    fn fresh(&self, params: Vec<(String, String)>) -> Self {
        Env {
            scope: self.scope,
            binders: Vec::new(),
            params,
            depth: self.depth + 1,
        }
    }

    fn name(&mut self, n: &str, line: usize) -> Result<String, Error> {
        if let Some((_, text)) = self.params.iter().rev().find(|(p, _)| p == n) {
            return Ok(text.clone());
        }
        if let Some(pos) = self.binders.iter().rposition(|b| b.as_deref() == Some(n)) {
            return Ok(format!("${}", self.binders.len() - 1 - pos));
        }
        if self.scope.is_input(n) || n == "out" {
            return Ok(n.to_string());
        }
        if let Some(a) = self.scope.abbrev(n) {
            if self.depth > 16 {
                return Err(Error::new(
                    crate::ErrorKind::TypeError,
                    line,
                    "abbreviation refers to itself",
                ));
            }
            return self.fresh(Vec::new()).print(&a.body);
        }
        if let Some(v) = self.scope.graph.named_value(self.scope.file, n) {
            return self.fresh(Vec::new()).print(&v.value);
        }
        Err(Error::new(
            crate::ErrorKind::UnboundName,
            line,
            format!("`{n}` is not bound"),
        ))
    }

    fn print(&mut self, e: &Expr) -> Result<String, Error> {
        Ok(match e {
            Expr::Int(i) => format!("(int {i})"),
            Expr::Float(f) => format!("(float {f:?})"),
            Expr::Text(s) => format!("(text {s:?})"),
            Expr::None => "none".into(),
            Expr::Name(n, l) => self.name(n, *l)?,
            Expr::Field(b, f, _) => format!("(. {} {f})", self.print(b)?),
            Expr::Call(n, args, l) => {
                let printed: Vec<String> = args
                    .iter()
                    .map(|a| self.print(a))
                    .collect::<Result<_, _>>()?;
                if let Some(a) = self.scope.abbrev(n) {
                    if self.depth > 16 {
                        return Err(Error::new(
                            crate::ErrorKind::TypeError,
                            *l,
                            "abbreviation refers to itself",
                        ));
                    }
                    let params = a
                        .params
                        .iter()
                        .map(|(p, _)| p.clone())
                        .zip(printed)
                        .collect();
                    return self.fresh(params).print(&a.body);
                }
                format!("(call {n} {})", printed.join(" "))
            }
            Expr::Ok(x) => format!("(ok {})", self.print(x)?),
            Expr::Err(x) => format!("(err {})", self.print(x)?),
            Expr::List(xs) => {
                let p: Vec<String> = xs.iter().map(|x| self.print(x)).collect::<Result<_, _>>()?;
                format!("(list {})", p.join(" "))
            }
            Expr::Tuple(xs) => {
                let p: Vec<String> = xs.iter().map(|x| self.print(x)).collect::<Result<_, _>>()?;
                format!("(tuple {})", p.join(" "))
            }
            Expr::Record(fields, rest) => {
                let mut p: Vec<String> = Vec::new();
                for (n, x) in fields {
                    p.push(format!("({n} {})", self.print(x)?));
                }
                p.sort();
                if *rest {
                    p.push("(rest)".into());
                }
                format!("(rec {})", p.join(" "))
            }
            Expr::Not(x) => format!("(not {})", self.print(x)?),
            Expr::Binary(op, a, b) => {
                format!("({} {} {})", op.symbol(), self.print(a)?, self.print(b)?)
            }
            Expr::Before { a, b, list } => {
                format!(
                    "(before {} {} {})",
                    self.print(a)?,
                    self.print(b)?,
                    self.print(list)?
                )
            }
            Expr::Relation(rel, a, b) => {
                format!("({} {} {})", rel.symbol(), self.print(a)?, self.print(b)?)
            }
            Expr::SortedBy(x, keys) => {
                let ks: Vec<String> = keys
                    .iter()
                    .map(|(k, d)| format!("({k} {})", if *d == Dir::Asc { "asc" } else { "desc" }))
                    .collect();
                format!("(sorted_by {} {})", self.print(x)?, ks.join(" "))
            }
            Expr::Unordered(x) => format!("(unordered {})", self.print(x)?),
            Expr::Unique(x) => format!("(unique {})", self.print(x)?),
            Expr::Quantifier(q) => {
                let domain = self.print(&q.domain)?;
                let pat = match (q.binder.side, q.binder.name.is_some()) {
                    (None, false) => "_".to_string(),
                    (None, true) => "$".to_string(),
                    (Some(Side::Ok), false) => "ok(_)".to_string(),
                    (Some(Side::Ok), true) => "ok($)".to_string(),
                    (Some(Side::Err), false) => "err(_)".to_string(),
                    (Some(Side::Err), true) => "err($)".to_string(),
                };
                self.binders.push(q.binder.name.clone());
                let mut s = format!(
                    "({} {pat}{} {domain}",
                    q.kind.word(),
                    if q.leftover { " leftover" } else { "" }
                );
                if let Some(w) = &q.where_ {
                    s.push_str(&format!(" (where {})", self.print(w)?));
                }
                if let Some(b) = &q.body {
                    s.push_str(&format!(" (body {})", self.print(b)?));
                }
                if let Some(e) = &q.else_ {
                    s.push_str(&format!(" (else {})", self.print(e)?));
                }
                s.push(')');
                self.binders.pop();
                s
            }
        })
    }
}
