//! Gaps: the underdetermination check's findings as an author reads them.
//! Each underdetermined input is two outputs that pass every clause, and
//! many inputs show the same hole, so they are grouped by where the two
//! outputs differ. A gap is that place, the smallest input that shows it,
//! and the slipped mutants whose output is one of a pair it holds: the same
//! hole, seen from the library.

use crate::mutants::{Mutant, MutantReport, Underdetermined};
use crate::value::Value;

/// One step into a value: a side of a `Result`, a field, an element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Ok,
    Err,
    Field(String),
    Index(usize),
}

/// What differs at the place two outputs part.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// Two different values.
    Values(Value, Value),
    /// The same elements in another order.
    Order,
    /// Elements one list has and the other does not, counted with
    /// multiplicity.
    Elements {
        only_a: Vec<Value>,
        only_b: Vec<Value>,
    },
}

/// Where two outputs first differ, and how.
#[derive(Debug, Clone, PartialEq)]
pub struct Divergence {
    pub path: Vec<Step>,
    pub change: Change,
}

/// Where `a` and `b` part: through a shared side, a record whose one field
/// differs, a list whose one element differs (in place, or alone among
/// the unmatched once order is set aside); a list that differs more is an
/// order or an element difference; anything else is two values. None when
/// they are equal.
pub fn diverge(a: &Value, b: &Value) -> Option<Divergence> {
    if a == b {
        return None;
    }
    let mut path = Vec::new();
    let (mut a, mut b) = (a, b);
    let change = loop {
        match (a, b) {
            (Value::Ok(x), Value::Ok(y)) => {
                path.push(Step::Ok);
                (a, b) = (x, y);
            }
            (Value::Err(x), Value::Err(y)) => {
                path.push(Step::Err);
                (a, b) = (x, y);
            }
            (Value::Record(fa), Value::Record(fb))
                if fa.len() == fb.len() && fa.iter().zip(fb).all(|(x, y)| x.0 == y.0) =>
            {
                let differing: Vec<_> = fa.iter().zip(fb).filter(|(x, y)| x.1 != y.1).collect();
                match differing.as_slice() {
                    [((name, x), (_, y))] => {
                        path.push(Step::Field(name.clone()));
                        (a, b) = (x, y);
                    }
                    _ => break Change::Values(a.clone(), b.clone()),
                }
            }
            (Value::List(xs), Value::List(ys)) => {
                if xs.len() == ys.len() {
                    let differing: Vec<usize> = (0..xs.len()).filter(|&i| xs[i] != ys[i]).collect();
                    if let [i] = differing.as_slice() {
                        path.push(Step::Index(*i));
                        (a, b) = (&xs[*i], &ys[*i]);
                        continue;
                    }
                }
                let (only_a, only_b) = apart(xs, ys);
                // One element each side: the same slot holding another
                // value, wherever the order put it.
                if let ([i], [j]) = (only_a.as_slice(), only_b.as_slice()) {
                    path.push(Step::Index(*i));
                    (a, b) = (&xs[*i], &ys[*j]);
                    continue;
                }
                break if only_a.is_empty() && only_b.is_empty() {
                    Change::Order
                } else {
                    Change::Elements {
                        only_a: only_a.iter().map(|&i| xs[i].clone()).collect(),
                        only_b: only_b.iter().map(|&j| ys[j].clone()).collect(),
                    }
                };
            }
            _ => break Change::Values(a.clone(), b.clone()),
        }
    };
    Some(Divergence { path, change })
}

/// The positions of the elements of `xs` not matched in `ys`, and of `ys`
/// not matched in `xs`, each element matching at most once.
fn apart(xs: &[Value], ys: &[Value]) -> (Vec<usize>, Vec<usize>) {
    let mut left: Vec<usize> = (0..ys.len()).collect();
    let mut only_a = Vec::new();
    for (i, x) in xs.iter().enumerate() {
        match left.iter().position(|&j| ys[j] == *x) {
            Some(k) => {
                left.remove(k);
            }
            None => only_a.push(i),
        }
    }
    (only_a, left)
}

fn listed(vs: &[Value]) -> String {
    vs.iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A value's text cut to fit a one-line message; the full text is the
/// witness's.
fn cut(text: String) -> String {
    const MAX: usize = 60;
    if text.chars().count() <= MAX {
        text
    } else {
        format!("{}…", text.chars().take(MAX).collect::<String>())
    }
}

impl Divergence {
    /// The place in `out`: `out`, `out[0].events`, `on Ok, out.code`; a
    /// side below the top reads `(Ok)`.
    pub fn place(&self) -> String {
        self.place_with(|i| format!("[{i}]"))
    }

    /// The place with every index as `[_]`: what inputs showing the same
    /// hole share.
    fn shape(&self) -> String {
        self.place_with(|_| "[_]".to_string())
    }

    fn place_with(&self, index: impl Fn(usize) -> String) -> String {
        let mut steps = self.path.as_slice();
        let mut s = String::new();
        match steps.first() {
            Some(Step::Ok) => {
                s.push_str("on Ok, ");
                steps = &steps[1..];
            }
            Some(Step::Err) => {
                s.push_str("on Err, ");
                steps = &steps[1..];
            }
            _ => {}
        }
        s.push_str("out");
        for step in steps {
            match step {
                Step::Ok => s.push_str(" (Ok)"),
                Step::Err => s.push_str(" (Err)"),
                Step::Field(f) => s.push_str(&format!(".{f}")),
                Step::Index(i) => s.push_str(&index(*i)),
            }
        }
        s
    }

    /// What each output holds at the place, as text: the two values, the
    /// elements only it has; None for an order difference.
    pub fn parts(&self) -> Option<(String, String)> {
        match &self.change {
            Change::Values(a, b) => Some((a.to_string(), b.to_string())),
            Change::Order => None,
            Change::Elements { only_a, only_b } => Some((listed(only_a), listed(only_b))),
        }
    }

    /// The difference in one line, each value cut to fit.
    pub fn says(&self) -> String {
        let place = self.place();
        match &self.change {
            Change::Values(a, b) => format!(
                "{place} can be {} or {}",
                cut(a.to_string()),
                cut(b.to_string())
            ),
            Change::Order => format!("{place} can hold the same elements in another order"),
            Change::Elements { only_a, only_b } if only_a.is_empty() || only_b.is_empty() => {
                let extra = if only_a.is_empty() { only_b } else { only_a };
                format!("{place} can have {} or not", cut(listed(extra)))
            }
            Change::Elements { only_a, only_b } => format!(
                "{place} can have {} or {} in its place",
                cut(listed(only_a)),
                cut(listed(only_b))
            ),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self.change {
            Change::Values(..) => "values",
            Change::Order => "order",
            Change::Elements { .. } => "elements",
        }
    }
}

/// How big an input is to read: one per value, plus a text's length.
fn size(v: &Value) -> usize {
    match v {
        Value::Text(s) => 1 + s.chars().count(),
        Value::Time { text, .. } => 1 + text.chars().count(),
        Value::List(xs) => 1 + xs.iter().map(size).sum::<usize>(),
        Value::Record(fs) => 1 + fs.iter().map(|(_, v)| size(v)).sum::<usize>(),
        Value::Ok(x) | Value::Err(x) => 1 + size(x),
        _ => 1,
    }
}

/// One hole: where the two passing outputs differ on its smallest input,
/// every input that shows it, and the slipped mutants that return one of
/// its outputs.
#[derive(Debug, Clone)]
pub struct Gap {
    pub divergence: Divergence,
    /// Indices into the report's `search`, the smallest input first.
    pub inputs: Vec<usize>,
    pub mutants: Vec<Mutant>,
}

impl Gap {
    pub fn witness<'r>(&self, r: &'r MutantReport) -> &'r Underdetermined {
        &r.search[self.inputs[0]]
    }

    pub fn says(&self) -> String {
        self.divergence.says()
    }
}

/// The report's open underdetermined inputs as gaps, in the order their
/// first input was found, and each slipped mutant in the first gap that
/// holds its output on the same input, or on any input for a mutant that
/// returns a constant, or that names it as the same hole.
pub fn gaps(r: &MutantReport) -> Vec<Gap> {
    let mut keyed: Vec<((String, &'static str), Gap)> = Vec::new();
    for (i, u) in r.search.iter().enumerate() {
        if u.accepted_by.is_some() {
            continue;
        }
        let Some(d) = diverge(&u.a, &u.b) else {
            continue;
        };
        let key = (d.shape(), d.kind());
        match keyed.iter_mut().find(|(k, _)| *k == key) {
            Some((_, g)) => {
                g.inputs.push(i);
                let witness = &r.search[g.inputs[0]];
                if inputs_size(u) < inputs_size(witness) {
                    g.inputs.rotate_right(1);
                    g.divergence = d;
                }
            }
            None => keyed.push((
                key,
                Gap {
                    divergence: d,
                    inputs: vec![i],
                    mutants: Vec::new(),
                },
            )),
        }
    }
    let mut out: Vec<Gap> = keyed.into_iter().map(|(_, g)| g).collect();
    for m in r.slipped() {
        // A mutant whose output does not depend on the input matches a
        // pair on any input; any other only on the input it was shown on.
        let constant = matches!(m.mutant, Mutant::Empty | Mutant::OkEmpty);
        let holds = |g: &Gap| {
            g.inputs.iter().any(|&i| {
                let u = &r.search[i];
                u.same_as == Some(m.mutant)
                    || m.shown.as_ref().is_some_and(|sh| {
                        (constant || sh.inputs == u.inputs)
                            && (sh.output == u.a || sh.output == u.b)
                    })
            })
        };
        if let Some(g) = out.iter_mut().find(|g| holds(g)) {
            g.mutants.push(m.mutant);
        }
    }
    out
}

fn inputs_size(u: &Underdetermined) -> usize {
    u.inputs.iter().map(size).sum()
}
