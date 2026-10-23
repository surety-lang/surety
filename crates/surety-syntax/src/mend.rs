//! Hints and errors that carry their edit (slice 6, piece 6). One function,
//! `diagnose`, behind `surety parse --json` and the language server, so the
//! draft tool and the editor read the same list. Three shapes: the
//! standalone-side hint, `pattern_as_value` on `out == Err(x)` or
//! `out == Ok(x)`, and `quantifier_mid_line` on a line whose top-level
//! `and` or `or` has a quantifier operand.
//!
//! Every edit is built on the tree and verified: the node replaced, the
//! line printed with `render`, parsed again, and offered only when the
//! parsed line's canonical form is the intended tree's. A quantifier's body
//! runs to the end of its line, so a text rewrite in place is not safe;
//! the reparse is what makes it safe. A hint never changes the exit code.

use crate::ast::*;
use crate::error::{Edit, Error, ErrorKind};
use crate::parser::{PropertyLine, lenient_property, property_line};
use crate::render;

/// A hint: never an error, never a change to the exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub code: &'static str,
    pub line: usize,
    pub message: String,
    pub edit: Option<Edit>,
}

#[derive(Debug, Clone, Default)]
pub struct Diagnosis {
    /// The errors as given, each carrying its edit where one is verified
    /// and its message naming the form.
    pub errors: Vec<Error>,
    pub hints: Vec<Hint>,
}

/// The errors `parse` reports for `text`, each given the edit that mends it
/// where one is verified, and the hints on the file when it parses.
pub fn diagnose(text: &str, errors: Vec<Error>) -> Diagnosis {
    let errors = errors.into_iter().map(|e| carry(text, e)).collect();
    let hints = match crate::parse(text) {
        Ok(file) => standalone_sides(text, &file),
        Err(_) => Vec::new(),
    };
    Diagnosis { errors, hints }
}

fn carry(text: &str, e: Error) -> Error {
    let mended = match e.kind {
        ErrorKind::PatternAsValue => lenient_property(text, e.line).and_then(|p| pattern(&p)),
        ErrorKind::QuantifierMidLine => lenient_property(text, e.line).and_then(|p| mid_line(&p)),
        _ => None,
    };
    match mended {
        Some((message, edit)) => Error {
            message,
            edit: edit.map(Box::new),
            ..e
        },
        None => e,
    }
}

/// `out == Err(x)` as `exactly_one Err(e) in out: e == x` (the `Ok` mirror
/// with `p`): in place, else for an iff with the sides swapped, else no
/// edit and the abbreviation form in the message.
fn pattern(p: &PropertyLine) -> Option<(String, Option<Edit>)> {
    let mut found: Vec<(Expr, Expr)> = Vec::new();
    let replaced = rebuild(&p.expr, &mut Vec::new(), &mut |e, bound| {
        let (side, x) = out_eq_pattern(e)?;
        let mut taken = p.names.clone();
        taken.extend(bound.iter().cloned());
        let q = exactly_one(side, &fresh(side, &taken), x);
        found.push((e.clone(), q.clone()));
        Some(q)
    });
    let [(was, q)] = &found[..] else {
        return None;
    };
    let mut message = format!(
        "`{}` compares the output with a pattern, and a pattern is not a value: the claim is written `{}`",
        render::expr(was),
        render::expr(q)
    );
    let mut candidates = vec![replaced.clone()];
    if let Expr::Binary(BinOp::Iff, a, b) = &replaced {
        candidates.push(Expr::Binary(BinOp::Iff, b.clone(), a.clone()));
    }
    for cand in candidates {
        if let Some(lines) = verify(std::slice::from_ref(&cand)) {
            let title = format!("Write `{}`", render::expr(q));
            return Some((message, Some(edit(p, title, &lines))));
        }
    }
    // Neither form parses to the meaning: the line holds another
    // quantifier. Name the form; invent no name.
    if let Expr::Binary(BinOp::Iff, a, b) = &p.expr {
        let other = if out_eq_pattern(a).is_some() {
            Some(b)
        } else if out_eq_pattern(b).is_some() {
            Some(a)
        } else {
            None
        };
        if let Some(other) = other {
            message.push_str(&format!(
                "; the line holds another quantifier, so that form cannot stand on it: name the other side, `name = {}`, and write `name <-> ({})`",
                render::expr(other),
                render::expr(q)
            ));
        }
    }
    Some((message, None))
}

/// A line whose top-level `and` has a quantifier operand, as one line per
/// operand; a top-level `or`, `A or Q` and `Q or A` alike, as
/// `not A -> Q`; a top-level `<->` with the quantifier first, the sides
/// swapped.
fn mid_line(p: &PropertyLine) -> Option<(String, Option<Edit>)> {
    const RULE: &str = "a quantifier must be the last operand on its line";
    match &p.expr {
        Expr::Binary(BinOp::And, ..) => {
            let mut parts = Vec::new();
            conjuncts(&p.expr, &mut parts);
            let message = format!(
                "{RULE}: the `and` joins {} claims, and each can be a line of its own",
                parts.len()
            );
            let edit = verify(&parts).map(|lines| {
                edit(
                    p,
                    format!("Split into {} lines, one claim each", lines.len()),
                    &lines,
                )
            });
            Some((message, edit))
        }
        Expr::Binary(BinOp::Or, a, b) => {
            let (other, q) = match (has_quantifier(a), has_quantifier(b)) {
                (false, true) => (a, b),
                (true, false) => (b, a),
                _ => return None,
            };
            let message = format!("{RULE}: `A or Q` is written `not A -> Q`, the quantifier last");
            let intended = Expr::Binary(
                BinOp::Implies,
                Box::new(Expr::Not(other.clone())),
                q.clone(),
            );
            let edit = verify(std::slice::from_ref(&intended)).map(|lines| {
                edit(
                    p,
                    "Write it as `not A -> Q`, the quantifier last".to_string(),
                    &lines,
                )
            });
            Some((message, edit))
        }
        // The second candidate form, for an iff only: the sides swapped.
        Expr::Binary(BinOp::Iff, a, b) if has_quantifier(a) && !has_quantifier(b) => {
            let message = format!("{RULE}: the sides of `<->` swap, the quantifier last");
            let intended = Expr::Binary(BinOp::Iff, b.clone(), a.clone());
            let edit = verify(std::slice::from_ref(&intended)).map(|lines| {
                edit(
                    p,
                    "Swap the sides of `<->`, the quantifier last".to_string(),
                    &lines,
                )
            });
            Some((message, edit))
        }
        _ => None,
    }
}

/// A property line whose whole expression is a bodied `any` or
/// `exactly_one` with an `Ok(..)` or `Err(..)` binder over `out`, `out` a
/// `Result`: it claims that side on every input. No hint inside `<->`,
/// `->`, `and`, `or` or `not`, where the form is a claim of its own.
fn standalone_sides(text: &str, file: &File) -> Vec<Hint> {
    let mut hints = Vec::new();
    for d in &file.decls {
        let Decl::Function(f) = d else {
            continue;
        };
        if !is_result(&f.output, file) {
            continue;
        }
        for prop in &f.properties {
            let Expr::Quantifier(q) = &prop.expr else {
                continue;
            };
            let (Some(side), Some(_)) = (q.binder.side, &q.body) else {
                continue;
            };
            if !matches!(q.kind, QKind::Any | QKind::ExactlyOne)
                || !matches!(&q.domain, Expr::Name(n, _) if n == "out")
            {
                continue;
            }
            let (word, other) = match side {
                Side::Ok => ("Ok", "Err"),
                Side::Err => ("Err", "Ok"),
            };
            let message = format!(
                "this line claims the `{word}` side on every input: `{} {word}(..) in out` is false whenever the output is `{other}`; a claim about the `{word}` side alone is `all`",
                q.kind.word()
            );
            let mut all = (**q).clone();
            all.kind = QKind::All;
            let all = Expr::Quantifier(Box::new(all));
            let edit = lenient_property(text, prop.line).and_then(|p| {
                verify(std::slice::from_ref(&all)).map(|lines| {
                    edit(
                        &p,
                        format!("Claim it only where the output is `{word}`: `all`"),
                        &lines,
                    )
                })
            });
            hints.push(Hint {
                code: "standalone_side",
                line: prop.line,
                message,
                edit,
            });
        }
    }
    hints
}

fn is_result(t: &Type, file: &File) -> bool {
    match t {
        Type::Result(..) => true,
        Type::Named(n) => file.decls.iter().any(
            |d| matches!(d, Decl::Type(td) if td.name == *n && matches!(td.ty, Type::Result(..))),
        ),
        _ => false,
    }
}

/// Each tree printed with `render` and parsed again as a line of its own:
/// the printed lines when every one parses to its tree, else none.
fn verify(trees: &[Expr]) -> Option<Vec<String>> {
    trees
        .iter()
        .map(|t| {
            let line = render::expr(t);
            let parsed = property_line(&line).ok()?;
            (canonical(&parsed) == canonical(t)).then_some(line)
        })
        .collect()
}

fn edit(p: &PropertyLine, title: String, lines: &[String]) -> Edit {
    let text = lines
        .iter()
        .map(|l| format!("{}{l}", p.indent))
        .collect::<Vec<_>>()
        .join("\n");
    Edit {
        title,
        from: p.from,
        to: p.to,
        text,
    }
}

/// `out == Ok(x)` or `out == Err(x)`: the side and `x`.
fn out_eq_pattern(e: &Expr) -> Option<(Side, &Expr)> {
    let Expr::Binary(BinOp::Eq, a, b) = e else {
        return None;
    };
    if !matches!(&**a, Expr::Name(n, _) if n == "out") {
        return None;
    }
    match &**b {
        Expr::Ok(x) => Some((Side::Ok, x)),
        Expr::Err(x) => Some((Side::Err, x)),
        _ => None,
    }
}

fn exactly_one(side: Side, binder: &str, x: &Expr) -> Expr {
    Expr::Quantifier(Box::new(Quantifier {
        kind: QKind::ExactlyOne,
        binder: Binder {
            side: Some(side),
            name: Some(binder.to_string()),
        },
        leftover: false,
        domain: Expr::Name("out".to_string(), 0),
        where_: None,
        body: Some(Expr::Binary(
            BinOp::Eq,
            Box::new(Expr::Name(binder.to_string(), 0)),
            Box::new(x.clone()),
        )),
        else_: None,
        line: 0,
    }))
}

/// The first of `e`, `err`, `e2`, … (`p`, `p2`, … for `Ok`) that names no
/// input, named value, abbreviation or binder in scope: a shadowing
/// binder changes what the body says.
fn fresh(side: Side, taken: &[String]) -> String {
    let (first, stem): (&[&str], &str) = match side {
        Side::Err => (&["e", "err"], "e"),
        Side::Ok => (&["p"], "p"),
    };
    first
        .iter()
        .map(|s| s.to_string())
        .chain((2..).map(|n| format!("{stem}{n}")))
        .find(|n| !taken.contains(n))
        .expect("an unbounded supply")
}

fn conjuncts(e: &Expr, out: &mut Vec<Expr>) {
    match e {
        Expr::Binary(BinOp::And, a, b) => {
            conjuncts(a, out);
            conjuncts(b, out);
        }
        _ => out.push(e.clone()),
    }
}

fn has_quantifier(e: &Expr) -> bool {
    let mut found = false;
    e.walk(&mut |x| found |= matches!(x, Expr::Quantifier(_)));
    found
}

/// The tree with `f` asked first at every node, outermost first, given the
/// binders in scope there; where it answers, its answer replaces the node.
fn rebuild(
    e: &Expr,
    bound: &mut Vec<String>,
    f: &mut dyn FnMut(&Expr, &[String]) -> Option<Expr>,
) -> Expr {
    if let Some(new) = f(e, bound) {
        return new;
    }
    let mut r = |x: &Expr, bound: &mut Vec<String>| Box::new(rebuild(x, bound, f));
    match e {
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Text(_)
        | Expr::None
        | Expr::Bool(_)
        | Expr::Name(..) => e.clone(),
        Expr::Field(b, n, l) => Expr::Field(r(b, bound), n.clone(), *l),
        Expr::Call(n, args, l) => {
            Expr::Call(n.clone(), args.iter().map(|a| *r(a, bound)).collect(), *l)
        }
        Expr::Ok(x) => Expr::Ok(r(x, bound)),
        Expr::Err(x) => Expr::Err(r(x, bound)),
        Expr::List(xs) => Expr::List(xs.iter().map(|x| *r(x, bound)).collect()),
        Expr::Tuple(xs) => Expr::Tuple(xs.iter().map(|x| *r(x, bound)).collect()),
        Expr::Record(fields, rest) => Expr::Record(
            fields
                .iter()
                .map(|(n, x)| (n.clone(), *r(x, bound)))
                .collect(),
            *rest,
        ),
        Expr::Not(x) => Expr::Not(r(x, bound)),
        Expr::Binary(op, a, b) => Expr::Binary(*op, r(a, bound), r(b, bound)),
        Expr::Arith(op, a, b) => Expr::Arith(*op, r(a, bound), r(b, bound)),
        Expr::Relation(op, a, b) => Expr::Relation(*op, r(a, bound), r(b, bound)),
        Expr::Slice { list, from, to } => Expr::Slice {
            list: r(list, bound),
            from: r(from, bound),
            to: r(to, bound),
        },
        Expr::Before { a, b, list } => Expr::Before {
            a: r(a, bound),
            b: r(b, bound),
            list: r(list, bound),
        },
        Expr::SortedBy(x, keys) => Expr::SortedBy(r(x, bound), keys.clone()),
        Expr::Unordered(x) => Expr::Unordered(r(x, bound)),
        Expr::Unique(x) => Expr::Unique(r(x, bound)),
        Expr::Quantifier(q) => {
            let domain = *r(&q.domain, bound);
            let pushed = q.binder.name.clone().map(|n| bound.push(n)).is_some();
            let where_ = q.where_.as_ref().map(|w| *r(w, bound));
            let body = q.body.as_ref().map(|b| *r(b, bound));
            if pushed {
                bound.pop();
            }
            let else_ = q.else_.as_ref().map(|x| *r(x, bound));
            Expr::Quantifier(Box::new(Quantifier {
                domain,
                where_,
                body,
                else_,
                ..(**q).clone()
            }))
        }
    }
}

/// The tree with every source position cleared, printed: what two parses
/// of one meaning share and two meanings never do.
fn canonical(e: &Expr) -> String {
    fn clear(e: &Expr) -> Expr {
        match e {
            Expr::Name(n, _) => Expr::Name(n.clone(), 0),
            Expr::Field(b, n, _) => Expr::Field(Box::new(clear(b)), n.clone(), 0),
            Expr::Call(n, args, _) => Expr::Call(n.clone(), args.iter().map(clear).collect(), 0),
            Expr::Quantifier(q) => Expr::Quantifier(Box::new(Quantifier {
                domain: clear(&q.domain),
                where_: q.where_.as_ref().map(clear),
                body: q.body.as_ref().map(clear),
                else_: q.else_.as_ref().map(clear),
                line: 0,
                ..(**q).clone()
            })),
            // Every other node: its children cleared, itself as it is.
            _ => {
                let mut first = true;
                rebuild(e, &mut Vec::new(), &mut |x, _| {
                    if std::mem::take(&mut first) {
                        None
                    } else {
                        Some(clear(x))
                    }
                })
            }
        }
    }
    format!("{:?}", clear(e))
}
