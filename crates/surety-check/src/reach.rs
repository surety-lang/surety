//! The reach rule: whether an example tests a clause (slice 6, piece 1).
//!
//! An example reaches a clause when, evaluated on the example's own input
//! and output, flipping the value of some of the clause's output-reading
//! parts at some of their evaluations makes the clause false. A part never
//! evaluated on that pair cannot be flipped, which is what vacuous means:
//! `all Ok(p) in out: …` on an error example has no binding, so nothing in
//! its body can be flipped, and no example of the other side tests it.
//!
//! Two passes over one evaluation, with the evaluator doing the work. The
//! trace evaluates every operand and every binding's body with no
//! short-circuit, since a part the evaluator skipped is exactly a part a
//! flip may need. Each part that is not a connective or a bodied quantifier
//! is a leaf: free when it reads the output (it names `out`, ranges over
//! `leftover`, calls the obligation, or mentions a binder whose domain
//! reads the output), fixed at its value when it does not, and without a
//! value when its evaluation failed, which only a part the evaluator would
//! have skipped can do. The same part under two bindings is two leaves. A
//! bodiless quantifier is one leaf; a bodied one's pattern count is not
//! flippable, its body's evaluations are. The second pass gives each node
//! whether it can be true and whether it can be false, from its children,
//! by the connectives' truth tables and the quantifiers' count ranges; the
//! example reaches the clause when the root can be false.
//!
//! The slots are desugared as the evaluator reads them: `all x in L where
//! w: b` is `all x in L: w -> b`; `any` and `none` with `where w: b` take
//! `w and b`; `exactly_one x in L where w: b` is `(exactly_one x in L: w)
//! and (all x in L: w -> b)`; and `Q x in L where w: b else e` is `((any x
//! in L: w) and (Q x in L where w: b)) or ((none x in L: w) and e)`, `e` one
//! evaluation in the outer scope. A filter's leaves stand in two places of
//! the desugared tree and are treated as independent in each, which can
//! only widen what a node can be: a clause is never called unreached that
//! a consistent flip would reach.

use surety_syntax::{BinOp, Expr, QKind, Quantifier};

use crate::checker::Checker;
use crate::eval::{Env, bind};

/// One evaluation of a clause, as far as flips go.
#[derive(Debug, Clone)]
enum Node {
    /// A part evaluated as a whole: its value (none when the evaluation
    /// failed) and whether it reads the output.
    Leaf {
        value: Option<bool>,
        free: bool,
    },
    Not(Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Implies(Box<Node>, Box<Node>),
    Iff(Box<Node>, Box<Node>),
    /// A quantifier over its bindings, one node per binding.
    Count(QKind, Vec<Node>),
}

/// What a node can be once the free leaves may take either value.
#[derive(Debug, Clone, Copy)]
struct Can {
    t: bool,
    f: bool,
}

impl Node {
    fn and(a: Node, b: Node) -> Node {
        Node::And(Box::new(a), Box::new(b))
    }

    fn can(&self) -> Can {
        match self {
            Node::Leaf { value: None, .. } => Can { t: false, f: false },
            Node::Leaf {
                value: Some(_),
                free: true,
            } => Can { t: true, f: true },
            Node::Leaf {
                value: Some(v),
                free: false,
            } => Can { t: *v, f: !*v },
            Node::Not(x) => {
                let x = x.can();
                Can { t: x.f, f: x.t }
            }
            Node::And(a, b) => {
                let (a, b) = (a.can(), b.can());
                Can {
                    t: a.t && b.t,
                    f: a.f || b.f,
                }
            }
            Node::Or(a, b) => {
                let (a, b) = (a.can(), b.can());
                Can {
                    t: a.t || b.t,
                    f: a.f && b.f,
                }
            }
            Node::Implies(a, b) => {
                let (a, b) = (a.can(), b.can());
                Can {
                    t: a.f || b.t,
                    f: a.t && b.f,
                }
            }
            Node::Iff(a, b) => {
                let (a, b) = (a.can(), b.can());
                Can {
                    t: (a.t && b.t) || (a.f && b.f),
                    f: (a.t && b.f) || (a.f && b.t),
                }
            }
            Node::Count(kind, items) => {
                let cs: Vec<Can> = items.iter().map(Node::can).collect();
                let any_t = cs.iter().any(|c| c.t);
                let any_f = cs.iter().any(|c| c.f);
                let all_t = cs.iter().all(|c| c.t);
                let all_f = cs.iter().all(|c| c.f);
                match kind {
                    QKind::All => Can { t: all_t, f: any_f },
                    QKind::Any => Can { t: any_t, f: all_f },
                    QKind::None => Can { t: all_f, f: any_t },
                    // One true while every other is false; false with
                    // none true, or with two.
                    QKind::ExactlyOne => Can {
                        t: (0..cs.len())
                            .any(|i| cs[i].t && cs.iter().enumerate().all(|(j, c)| j == i || c.f)),
                        f: all_f || cs.iter().filter(|c| c.t).count() >= 2,
                    },
                }
            }
        }
    }
}

impl<'g> Checker<'g> {
    /// Whether an example reaches a clause that holds on it: some flip of
    /// the clause's output-reading parts on the example's own pair makes
    /// it false. Call it only for a clause that evaluated true there.
    pub fn reaches(&mut self, expr: &Expr, env: &Env<'_>) -> bool {
        self.trace(expr, env, &[]).can().f
    }

    /// The clause's evaluation tree. `outs` are the binders in scope whose
    /// domain reads the output.
    fn trace(&mut self, e: &Expr, env: &Env<'_>, outs: &[String]) -> Node {
        match e {
            Expr::Not(x) => Node::Not(Box::new(self.trace(x, env, outs))),
            Expr::Binary(op @ (BinOp::And | BinOp::Or | BinOp::Implies | BinOp::Iff), a, b) => {
                let a = Box::new(self.trace(a, env, outs));
                let b = Box::new(self.trace(b, env, outs));
                match op {
                    BinOp::And => Node::And(a, b),
                    BinOp::Or => Node::Or(a, b),
                    BinOp::Implies => Node::Implies(a, b),
                    _ => Node::Iff(a, b),
                }
            }
            Expr::Quantifier(q) if q.body.is_some() => self.trace_quantifier(q, env, outs),
            _ => self.leaf(e, env, outs),
        }
    }

    fn leaf(&mut self, e: &Expr, env: &Env<'_>, outs: &[String]) -> Node {
        Node::Leaf {
            value: self.eval_bool(e, env).ok(),
            free: self.reads_output(e) || outs.iter().any(|n| e.mentions(n)),
        }
    }

    fn trace_quantifier(&mut self, q: &Quantifier, env: &Env<'_>, outs: &[String]) -> Node {
        let Ok(b) = self.quantifier_bindings(q, env) else {
            // Only a domain the evaluator never reached can fail here.
            return Node::Leaf {
                value: None,
                free: false,
            };
        };
        // The binder reads the output when its domain does; otherwise it
        // shadows any outer binder of the same name.
        let mut inner_outs: Vec<String> = outs
            .iter()
            .filter(|n| Some(*n) != q.binder.name.as_ref())
            .cloned()
            .collect();
        let domain_reads =
            q.leftover || self.reads_output(&q.domain) || outs.iter().any(|n| q.domain.mentions(n));
        if let (true, Some(n)) = (domain_reads, &q.binder.name) {
            inner_outs.push(n.clone());
        }
        let body = q.body.as_ref().expect("a bodied quantifier");
        let mut filters = Vec::new();
        let mut bodies = Vec::new();
        for (p, v) in &b.matched {
            let inner = bind(q, &b.scope, &b.shared, *p, v);
            if let Some(w) = &q.where_ {
                filters.push(self.trace(w, &inner, &inner_outs));
            }
            bodies.push(self.trace(body, &inner, &inner_outs));
        }
        let Some(_) = &q.where_ else {
            return Node::Count(q.kind, bodies);
        };
        let guarded = |join: fn(Node, Node) -> Node| -> Vec<Node> {
            filters
                .iter()
                .cloned()
                .zip(bodies.iter().cloned())
                .map(|(w, b)| join(w, b))
                .collect()
        };
        let implies = |w: Node, b: Node| Node::Implies(Box::new(w), Box::new(b));
        let filtered = match q.kind {
            QKind::All => Node::Count(QKind::All, guarded(implies)),
            QKind::Any | QKind::None => Node::Count(q.kind, guarded(Node::and)),
            QKind::ExactlyOne => Node::and(
                Node::Count(QKind::ExactlyOne, filters.clone()),
                Node::Count(QKind::All, guarded(implies)),
            ),
        };
        let Some(e) = &q.else_ else {
            return filtered;
        };
        let otherwise = self.trace(e, &b.scope, outs);
        Node::Or(
            Box::new(Node::and(
                Node::Count(QKind::Any, filters.clone()),
                filtered,
            )),
            Box::new(Node::and(Node::Count(QKind::None, filters), otherwise)),
        )
    }
}
