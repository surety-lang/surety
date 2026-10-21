//! The frame rule: fields preserved by default.
//!
//! A pair forms when the function has one record input and a record
//! output, or in `all x in L1: exactly_one y in L2 [where M]: B` when
//! exactly one of L1, L2 is reached from `out` and the other from an
//! input. A same-named field is preserved unless M or B names it on either
//! binder. Each pair becomes an implicit property the checker evaluates.
//!
//! Each pair also states a correspondence: which output list (or the
//! output record) goes with which input list (or input record), matched on
//! which fields, preserving which. The skeleton mutants are built from it.

use surety_syntax::resolve::Scope;
use surety_syntax::{BinOp, Binder, Expr, Function, QKind, Quantifier, Side, Type};

use crate::examples::Step;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Root {
    Out,
    Input(usize),
}

/// What a frame pair states structurally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correspondence {
    /// From `out` to the output list, or empty for the output itself.
    pub out_path: Vec<Step>,
    pub input: usize,
    /// From that input to the list, or empty for the input itself.
    pub input_path: Vec<Step>,
    /// `(output field, input field)` pairs the match equates.
    pub matched: Vec<(String, String)>,
    pub preserved: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FramePair {
    /// Human-readable: `o ~ i_o (line 12)`.
    pub description: String,
    pub line: usize,
    pub preserved: Vec<String>,
    /// The implicit property: every preserved field is equal on the pair.
    pub implicit: Expr,
    pub correspondence: Correspondence,
}

type Bound = (String, Root, Vec<Step>);

fn origin(e: &Expr, scope: &Scope<'_>, binders: &[Bound]) -> Option<(Root, Vec<Step>)> {
    match e {
        Expr::Name(n, _) if n == "out" => Some((Root::Out, Vec::new())),
        Expr::Name(n, _) if scope.is_input(n) => {
            let pos = scope.inputs.iter().position(|(m, _)| m == n)?;
            Some((Root::Input(pos), Vec::new()))
        }
        Expr::Name(n, _) => binders
            .iter()
            .rev()
            .find(|(b, _, _)| b == n)
            .map(|(_, r, p)| (*r, p.clone())),
        Expr::Field(base, f, _) => {
            let (r, mut p) = origin(base, scope, binders)?;
            p.push(Step::Field(f.clone()));
            Some((r, p))
        }
        _ => None,
    }
}

fn field_names(t: &Type) -> Vec<String> {
    match t {
        Type::Record { fields, .. } => fields.iter().map(|(n, _)| n.clone()).collect(),
        _ => Vec::new(),
    }
}

/// Fields `x.f` named anywhere in an expression, for a given root name.
fn fields_named(e: &Expr, root: &str, out: &mut Vec<String>) {
    e.walk(&mut |sub| {
        if let Expr::Field(base, f, _) = sub {
            if let Expr::Name(n, _) = &**base {
                if n == root && !out.contains(f) {
                    out.push(f.clone());
                }
            }
        }
    });
}

/// `(out field, input field)` pairs that `x.f == y.g` equalities state,
/// given which binder is the output side.
fn matched_fields(e: &Expr, out_binder: &str, in_binder: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    e.walk(&mut |sub| {
        let Expr::Binary(BinOp::Eq, a, b) = sub else {
            return;
        };
        let (Expr::Field(ba, fa, _), Expr::Field(bb, fb, _)) = (&**a, &**b) else {
            return;
        };
        let name = |base: &Expr| match base {
            Expr::Name(n, _) => Some(n.clone()),
            _ => None,
        };
        let (Some(na), Some(nb)) = (name(ba), name(bb)) else {
            return;
        };
        let pair = if na == out_binder && nb == in_binder {
            Some((fa.clone(), fb.clone()))
        } else if nb == out_binder && na == in_binder {
            Some((fb.clone(), fa.clone()))
        } else {
            None
        };
        if let Some(p) = pair {
            if !out.contains(&p) {
                out.push(p);
            }
        }
    });
    out
}

fn field_of(root: &str, f: &str, line: usize) -> Expr {
    Expr::Field(
        Box::new(Expr::Name(root.to_string(), line)),
        f.to_string(),
        line,
    )
}

fn all_equal(x: &str, y: &str, fields: &[String], line: usize) -> Expr {
    let mut iter = fields.iter();
    let first = iter.next().expect("at least one preserved field");
    let mut e = Expr::Binary(
        BinOp::Eq,
        Box::new(field_of(x, first, line)),
        Box::new(field_of(y, first, line)),
    );
    for f in iter {
        let eq = Expr::Binary(
            BinOp::Eq,
            Box::new(field_of(x, f, line)),
            Box::new(field_of(y, f, line)),
        );
        e = Expr::Binary(BinOp::And, Box::new(e), Box::new(eq));
    }
    e
}

/// Every frame pair in an obligation.
pub fn frame_pairs(scope: &Scope<'_>, f: &Function) -> Vec<FramePair> {
    let mut pairs = Vec::new();
    // One record input and a record output.
    if let [(input, in_ty)] = scope.inputs.as_slice() {
        let out_ty = scope.output.as_ref().unwrap();
        if matches!(in_ty, Type::Record { .. }) && matches!(out_ty, Type::Record { .. }) {
            let out_fields = field_names(out_ty);
            let mut named = Vec::new();
            for p in &f.properties {
                fields_named(&p.expr, "out", &mut named);
                fields_named(&p.expr, input, &mut named);
            }
            let preserved: Vec<String> = field_names(in_ty)
                .into_iter()
                .filter(|n| out_fields.contains(n) && !named.contains(n))
                .collect();
            if !preserved.is_empty() {
                pairs.push(FramePair {
                    description: format!("out ~ {input} (single record input)"),
                    line: f.line,
                    implicit: all_equal("out", input, &preserved, f.line),
                    correspondence: Correspondence {
                        out_path: Vec::new(),
                        input: 0,
                        input_path: Vec::new(),
                        matched: Vec::new(),
                        preserved: preserved.clone(),
                    },
                    preserved,
                });
            }
        }
    }
    for p in &f.properties {
        scan(&p.expr, scope, &mut Vec::new(), &mut pairs);
    }
    pairs
}

fn plain_name(b: &Binder) -> Option<&str> {
    match (b.side, &b.name) {
        (None, Some(n)) => Some(n),
        _ => None,
    }
}

fn binder_path(q: &Quantifier, domain: &[Step]) -> Vec<Step> {
    let mut p = domain.to_vec();
    p.push(match q.binder.side {
        Some(Side::Ok) => Step::Ok,
        Some(Side::Err) => Step::Err,
        None => Step::Each,
    });
    p
}

fn scan(e: &Expr, scope: &Scope<'_>, binders: &mut Vec<Bound>, pairs: &mut Vec<FramePair>) {
    match e {
        Expr::Quantifier(q) => {
            let dom = origin(&q.domain, scope, binders);
            let Ok(elem) = scope.domain_elem(q) else {
                return;
            };
            let inner_scope = match &q.binder.name {
                Some(n) => scope.bind(n, elem.clone()),
                None => scope.clone(),
            };
            // The pair shape: `all x in L1: exactly_one y in L2 [where M]: B`.
            if q.kind == QKind::All {
                if let (Some(x), Some(Expr::Quantifier(inner))) =
                    (plain_name(&q.binder), q.body.as_ref())
                {
                    if inner.kind == QKind::ExactlyOne && !inner.leftover {
                        if let Some(y) = plain_name(&inner.binder) {
                            let oy = origin(&inner.domain, scope, binders);
                            if let (Some((rx, px)), Some((ry, py))) = (&dom, &oy) {
                                let crossed = matches!(
                                    (rx, ry),
                                    (Root::Out, Root::Input(_)) | (Root::Input(_), Root::Out)
                                );
                                if crossed {
                                    if let Ok(ty) = inner_scope.domain_elem(inner.as_ref()) {
                                        let (out_side, in_side) = if *rx == Root::Out {
                                            ((x, px.clone()), (y, *ry, py.clone()))
                                        } else {
                                            ((y, py.clone()), (x, *rx, px.clone()))
                                        };
                                        pair(
                                            q,
                                            inner.as_ref(),
                                            x,
                                            y,
                                            &elem,
                                            &ty,
                                            out_side,
                                            in_side,
                                            pairs,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let pushed = match (&q.binder.name, &dom) {
                (Some(n), Some((r, p))) => {
                    binders.push((n.clone(), *r, binder_path(q, p)));
                    true
                }
                _ => false,
            };
            for part in [&q.where_, &q.body, &q.else_].into_iter().flatten() {
                scan(part, &inner_scope, binders, pairs);
            }
            if pushed {
                binders.pop();
            }
        }
        Expr::Binary(_, a, b) => {
            scan(a, scope, binders, pairs);
            scan(b, scope, binders, pairs);
        }
        Expr::Not(a) => scan(a, scope, binders, pairs),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn pair(
    outer: &Quantifier,
    inner: &Quantifier,
    x: &str,
    y: &str,
    tx: &Type,
    ty: &Type,
    out_side: (&str, Vec<Step>),
    in_side: (&str, Root, Vec<Step>),
    pairs: &mut Vec<FramePair>,
) {
    let fx = field_names(tx);
    let fy = field_names(ty);
    if fx.is_empty() || fy.is_empty() {
        return;
    }
    let mut named = Vec::new();
    for part in [&inner.where_, &inner.body].into_iter().flatten() {
        fields_named(part, x, &mut named);
        fields_named(part, y, &mut named);
    }
    let preserved: Vec<String> = fx
        .into_iter()
        .filter(|n| fy.contains(n) && !named.contains(n))
        .collect();
    // The match is `where M` when present, else the body itself.
    let matcher = inner
        .where_
        .clone()
        .or_else(|| inner.body.clone())
        .expect("exactly_one has a match");
    let (out_binder, out_path) = out_side;
    let (in_binder, in_root, input_path) = in_side;
    let Root::Input(input) = in_root else {
        return;
    };
    let correspondence = Correspondence {
        out_path,
        input,
        input_path,
        matched: matched_fields(&matcher, out_binder, in_binder),
        preserved: preserved.clone(),
    };
    if preserved.is_empty() {
        // No implicit property, but the correspondence still holds for the
        // skeleton mutants: record it on a pair with a vacuous claim.
        pairs.push(FramePair {
            description: format!("{x} ~ {y} (line {})", inner.line),
            line: inner.line,
            preserved,
            implicit: Expr::Binary(BinOp::Eq, Box::new(Expr::Int(0)), Box::new(Expr::Int(0))),
            correspondence,
        });
        return;
    }
    let line = inner.line;
    let implicit = Expr::Quantifier(Box::new(Quantifier {
        kind: QKind::All,
        binder: outer.binder.clone(),
        leftover: outer.leftover,
        domain: outer.domain.clone(),
        where_: None,
        body: Some(Expr::Quantifier(Box::new(Quantifier {
            kind: QKind::All,
            binder: inner.binder.clone(),
            leftover: false,
            domain: inner.domain.clone(),
            where_: Some(matcher),
            body: Some(all_equal(x, y, &preserved, line)),
            else_: None,
            line,
        }))),
        else_: None,
        line: outer.line,
    }));
    pairs.push(FramePair {
        description: format!("{x} ~ {y} (line {line})"),
        line,
        preserved,
        implicit,
        correspondence,
    });
}
