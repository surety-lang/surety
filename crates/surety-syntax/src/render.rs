//! The syntax tree printed back as Surety: the inverse of the parser up to
//! the hash. Types, named values, abbreviations, the signature, the note,
//! the function's abbreviations, each property under its sentence, the
//! accept lines and the examples, in the corpus's layout; comments at the
//! line they were pinned to. With fresh names, binders are renamed in order
//! of appearance, which must not change the hash.

use std::fmt::Write;

use crate::ast::*;

struct Printer {
    fresh: bool,
    counter: usize,
    /// Binder renames in scope, innermost last.
    scope: Vec<(String, String)>,
}

/// Binding strength, higher binds tighter; a child weaker than its parent
/// is parenthesized, and so is a right child of the same strength, so the
/// tree the parser builds is the tree that was printed.
fn strength(e: &Expr) -> u8 {
    match e {
        Expr::Quantifier(_) => 0,
        Expr::Binary(op, _, _) => match op {
            BinOp::Iff => 1,
            BinOp::Implies => 2,
            BinOp::Or => 3,
            BinOp::And => 4,
            _ => 6,
        },
        Expr::Not(_) => 5,
        Expr::Relation(..) | Expr::Before { .. } | Expr::Unordered(_) | Expr::Unique(_) => 6,
        Expr::SortedBy(..) => 6,
        _ => 9,
    }
}

impl Printer {
    fn name(&self, n: &str) -> String {
        self.scope
            .iter()
            .rev()
            .find(|(old, _)| old == n)
            .map(|(_, new)| new.clone())
            .unwrap_or_else(|| n.to_string())
    }

    fn operand(&mut self, e: &Expr, parent: u8, right: bool) -> String {
        let s = strength(e);
        let text = self.expr(e);
        let needs = s < parent || (s == parent && right && s < 9) || (s == 0 && !right);
        if needs { format!("({text})") } else { text }
    }

    fn primary(&mut self, e: &Expr) -> String {
        let text = self.expr(e);
        if strength(e) < 9 {
            format!("({text})")
        } else {
            text
        }
    }

    fn expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Int(i) => i.to_string(),
            Expr::Float(f) => format!("{f:?}"),
            Expr::Text(s) => format!("\"{s}\""),
            Expr::None => "None".to_string(),
            Expr::Name(n, _) => self.name(n),
            Expr::Field(base, f, _) => format!("{}.{f}", self.primary(base)),
            Expr::Call(n, args, _) => {
                let args: Vec<String> = args.iter().map(|a| self.expr(a)).collect();
                format!("{n}({})", args.join(", "))
            }
            Expr::Ok(x) => format!("Ok({})", self.expr(x)),
            Expr::Err(x) => format!("Err({})", self.expr(x)),
            Expr::List(xs) => {
                let xs: Vec<String> = xs.iter().map(|x| self.expr(x)).collect();
                format!("[{}]", xs.join(", "))
            }
            Expr::Record(fields, rest) => {
                let mut parts: Vec<String> = fields
                    .iter()
                    .map(|(n, v)| format!("{n}: {}", self.expr(v)))
                    .collect();
                if *rest {
                    parts.push("...".to_string());
                }
                format!("{{{}}}", parts.join(", "))
            }
            Expr::Tuple(xs) => {
                let xs: Vec<String> = xs.iter().map(|x| self.expr(x)).collect();
                format!("({})", xs.join(", "))
            }
            Expr::Not(x) => format!("not {}", self.operand(x, 5, true)),
            Expr::Binary(op, a, b) => {
                let p = strength(e);
                format!(
                    "{} {} {}",
                    self.operand(a, p, false),
                    op.symbol(),
                    self.operand(b, p, true)
                )
            }
            Expr::Before { a, b, list } => format!(
                "{} before {} in {}",
                self.primary(a),
                self.primary(b),
                self.primary(list)
            ),
            Expr::Relation(op, a, b) => {
                format!("{} {} {}", self.primary(a), op.symbol(), self.primary(b))
            }
            Expr::SortedBy(x, keys) => {
                let keys: Vec<String> = keys
                    .iter()
                    .map(|(k, d)| {
                        format!(
                            "{k} {}",
                            match d {
                                Dir::Asc => "asc",
                                Dir::Desc => "desc",
                            }
                        )
                    })
                    .collect();
                format!("{} sorted_by ({})", self.primary(x), keys.join(", "))
            }
            Expr::Unordered(x) => format!("{} unordered", self.primary(x)),
            Expr::Unique(x) => format!("unique {}", self.primary(x)),
            Expr::Quantifier(q) => self.quantifier(q),
        }
    }

    fn quantifier(&mut self, q: &Quantifier) -> String {
        let domain = self.expr(&q.domain);
        let pushed = match &q.binder.name {
            Some(n) if self.fresh => {
                let new = format!("b{}", self.counter);
                self.counter += 1;
                self.scope.push((n.clone(), new));
                true
            }
            _ => false,
        };
        let inner = match &q.binder.name {
            Some(n) => self.name(n),
            None => "_".to_string(),
        };
        let binder = match q.binder.side {
            Some(Side::Ok) => format!("Ok({inner})"),
            Some(Side::Err) => format!("Err({inner})"),
            None => inner,
        };
        let mut s = format!(
            "{} {binder}{} in {domain}",
            q.kind.word(),
            if q.leftover { " leftover" } else { "" }
        );
        if let Some(w) = &q.where_ {
            let _ = write!(s, " where {}", self.expr(w));
        }
        if let Some(b) = &q.body {
            let _ = write!(s, ": {}", self.expr(b));
        }
        if let Some(e) = &q.else_ {
            let _ = write!(s, " else {}", self.expr(e));
        }
        if pushed {
            self.scope.pop();
        }
        s
    }

    fn abbrev(&mut self, a: &Abbrev) -> String {
        let body = self.expr(&a.body);
        if a.params.is_empty() {
            format!("{} = {body}", a.name)
        } else {
            let ps: Vec<String> = a.params.iter().map(|(p, t)| format!("{p}: {t}")).collect();
            format!("{}({}) = {body}", a.name, ps.join(", "))
        }
    }
}

enum Item<'a> {
    Abbrev(&'a Abbrev),
    Property(&'a Property),
    Accept(&'a Property),
    Example(&'a Example),
    Comment(&'a str),
}

fn line_of(i: &Item, pins: &[(usize, &str)]) -> usize {
    match i {
        Item::Abbrev(a) => a.line,
        Item::Property(p) | Item::Accept(p) => p.line,
        Item::Example(e) => e.line,
        Item::Comment(t) => pins
            .iter()
            .find(|(_, x)| x == t)
            .map(|(l, _)| *l)
            .unwrap_or(0),
    }
}

/// The whole file. `fresh` renames every binder in order of appearance.
pub fn file(f: &File, fresh: bool) -> String {
    let mut p = Printer {
        fresh,
        counter: 0,
        scope: Vec::new(),
    };
    let mut out = String::new();
    for (_, l) in &f.header {
        out.push_str(l);
        out.push('\n');
    }
    let comments: Vec<(usize, &str)> = f
        .pins
        .iter()
        .filter(|x| x.kind == PinKind::Comment)
        .map(|x| (x.line, x.text.as_str()))
        .collect();
    let mut last_was_type = false;
    for d in &f.decls {
        match d {
            Decl::Type(t) => {
                let _ = writeln!(out, "type {} = {}", t.name, t.ty);
                last_was_type = true;
            }
            Decl::NamedValue(nv) => {
                if last_was_type {
                    out.push('\n');
                }
                let value = p.expr(&nv.value);
                match &nv.ty {
                    Some(t) => {
                        let _ = writeln!(out, "{}: {t} = {value}", nv.name);
                    }
                    None => {
                        let _ = writeln!(out, "{} = {value}", nv.name);
                    }
                }
                last_was_type = false;
            }
            Decl::Abbrev(a) => {
                if last_was_type {
                    out.push('\n');
                }
                let line = p.abbrev(a);
                out.push_str(&line);
                out.push('\n');
                last_was_type = false;
            }
            Decl::Function(fun) => {
                out.push('\n');
                let before: Vec<&(usize, &str)> =
                    comments.iter().filter(|(l, _)| *l < fun.line).collect();
                for (_, c) in before {
                    let _ = writeln!(out, "// {c}");
                }
                let ins: Vec<String> = fun
                    .inputs
                    .iter()
                    .map(|(n, t)| format!("{n}: {t}"))
                    .collect();
                let _ = writeln!(out, "{}({}) -> {}", fun.name, ins.join(", "), fun.output);
                if let Some(n) = &fun.note {
                    let _ = writeln!(out, "\tnote \"{n}\"");
                }
                let mut items: Vec<Item> = Vec::new();
                items.extend(fun.abbrevs.iter().map(Item::Abbrev));
                items.extend(fun.properties.iter().map(Item::Property));
                items.extend(fun.accepts.iter().map(Item::Accept));
                items.extend(fun.examples.iter().map(Item::Example));
                items.extend(
                    comments
                        .iter()
                        .filter(|(l, _)| *l > fun.line)
                        .map(|(_, c)| Item::Comment(c)),
                );
                items.sort_by_key(|i| line_of(i, &comments));
                for item in items {
                    match item {
                        Item::Abbrev(a) => {
                            let line = p.abbrev(a);
                            let _ = writeln!(out, "\t{line}");
                        }
                        Item::Property(pr) => {
                            if let Some(s) = &pr.sentence {
                                let _ = writeln!(out, "\t# {s}");
                            }
                            let e = p.expr(&pr.expr);
                            let _ = writeln!(out, "\t{e}");
                        }
                        Item::Accept(pr) => {
                            if let Some(s) = &pr.sentence {
                                let _ = writeln!(out, "\t# {s}");
                            }
                            let e = p.expr(&pr.expr);
                            let _ = writeln!(out, "\taccept {e}");
                        }
                        Item::Example(ex) => {
                            out.push_str("\n\te:\n");
                            if let Some(n) = &ex.note {
                                let _ = writeln!(out, "\t\tnote \"{n}\"");
                            }
                            let i = p.expr(&ex.input);
                            let o = p.expr(&ex.output);
                            let _ = writeln!(out, "\t\tin: {i}");
                            let _ = writeln!(out, "\t\tout: {o}");
                        }
                        Item::Comment(c) => {
                            let _ = writeln!(out, "\t// {c}");
                        }
                    }
                }
                last_was_type = false;
            }
        }
    }
    out
}
