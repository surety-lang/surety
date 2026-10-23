//! The underdetermination check (slice 2, acceptance tests 3 to 7).
//!
//! A fixed library of wrong performances is built from the obligation's
//! types and the input value, with no performance anywhere, and run against
//! the obligation over the same inputs `check` uses. A mutant that no
//! property or frame property catches on any input has slipped through,
//! and is reported with the clauses it slipped through; whether an example
//! caught it is stated beside that, because an example-only catch means
//! the property layer has slack on every input that is not that example.
//! A bounded search then looks, per input, for two different outputs that
//! both pass every clause: an input on which the obligation is
//! underdetermined.

use surety_syntax::parser::BUILTINS;
use surety_syntax::resolve::Scope;
use surety_syntax::{Expr, Type};

use crate::against::Against;
use crate::checker::{Checker, Outcome};
use crate::examples::{Step, compare};
use crate::frame::{Correspondence, frame_pairs};
use crate::generate::{Bucket, Domain, candidates};
use crate::value::{Value, from_json, to_json};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutant {
    Empty,
    OkEmpty,
    Identity,
    DropOne,
    DuplicateOne,
    AppendFabricated,
    SwapAdjacent,
    /// Built from the frame pairs: one output element per element of the
    /// paired input list, matched and preserved fields copied, every other
    /// field empty. The wrong performance for a transform.
    Skeleton,
    /// The skeleton with one element per distinct value of the matched
    /// fields: the wrong group-by.
    SkeletonPerKey,
    SkeletonDropOne,
    SkeletonDuplicateOne,
    SkeletonAppendFabricated,
    SkeletonSwapAdjacent,
    /// The scalar family: one `Int` or `Bool` of the output, not inside a
    /// list, replaced by a formula over the inputs, the rest the skeleton
    /// or the empty value. Built per signature by `scalar_family`; a mutant
    /// is a name plus parameters, so the family's identity is its path and
    /// formula and a later magnitude knob is a parameter, not a redesign.
    Scalar(Scalar),
}

/// An `Int` or `Bool` of the output, by path from the root through `Ok`
/// and record fields, and the formula that replaces it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar {
    pub path: Vec<Step>,
    pub formula: Formula,
}

/// What a scalar mutant returns in the scalar's place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    /// A constant, 1 or -1: 0 is the empty value, which `empty` and the
    /// skeleton already give.
    Const(i128),
    /// The base itself: the field is an input, or the sum of one (the tax
    /// forgotten). For a bare output that is an input, `identity` is this
    /// and the family leaves it out.
    Base(Base),
    PlusOne(Base),
    MinusOne(Base),
    Negated(Base),
    /// The base times 100: the unit-swap mutant, dollars where cents go.
    Scaled100(Base),
    /// The `Bool` flipped from the empty value's `false`.
    True,
}

/// An `Int` the inputs afford: an `Int` input, an `Int` field of a record
/// input, or the sum of an input list's `Int` elements or `Int` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Base {
    Input(usize, String),
    Field(usize, String, String),
    Sum(usize, String, Option<String>),
}

impl Base {
    fn text(&self) -> String {
        match self {
            Base::Input(_, n) => n.clone(),
            Base::Field(_, n, f) => format!("{n}.{f}"),
            Base::Sum(_, n, None) => format!("sum({n})"),
            Base::Sum(_, n, Some(f)) => format!("sum({n}.{f})"),
        }
    }

    fn value(&self, inputs: &[Value]) -> Option<i128> {
        fn int(v: &Value) -> Option<i128> {
            match v {
                Value::Int(i) => Some(*i),
                _ => None,
            }
        }
        match self {
            Base::Input(i, _) => int(inputs.get(*i)?),
            Base::Field(i, _, f) => int(inputs.get(*i)?.field(f)?),
            Base::Sum(i, _, f) => {
                let Value::List(xs) = inputs.get(*i)? else {
                    return None;
                };
                let mut total: i128 = 0;
                for x in xs {
                    let x = match f {
                        Some(f) => x.field(f)?,
                        None => x,
                    };
                    total = total.checked_add(int(x)?)?;
                }
                Some(total)
            }
        }
    }
}

impl Formula {
    /// The formula in the obligation's own spelling, as the mutant's name.
    fn text(&self) -> String {
        match self {
            Formula::Const(c) => c.to_string(),
            Formula::Base(b) => b.text(),
            Formula::PlusOne(b) => format!("{} + 1", b.text()),
            Formula::MinusOne(b) => format!("{} - 1", b.text()),
            Formula::Negated(b) => format!("0 - {}", b.text()),
            Formula::Scaled100(b) => format!("{} * 100", b.text()),
            Formula::True => "true".into(),
        }
    }

    /// The value on given inputs; none where the base overflows.
    fn value(&self, inputs: &[Value]) -> Option<Value> {
        Some(match self {
            Formula::Const(c) => Value::Int(*c),
            Formula::True => Value::Bool(true),
            Formula::Base(b) => Value::Int(b.value(inputs)?),
            Formula::PlusOne(b) => Value::Int(b.value(inputs)?.checked_add(1)?),
            Formula::MinusOne(b) => Value::Int(b.value(inputs)?.checked_sub(1)?),
            Formula::Negated(b) => Value::Int(b.value(inputs)?.checked_neg()?),
            Formula::Scaled100(b) => Value::Int(b.value(inputs)?.checked_mul(100)?),
        })
    }
}

impl Scalar {
    /// The scalar's place in the output: `out` for a bare scalar, else the
    /// field path, `Ok` left unsaid.
    pub fn path_text(&self) -> String {
        let fields: Vec<&str> = self
            .path
            .iter()
            .filter_map(|s| match s {
                Step::Field(f) => Some(f.as_str()),
                _ => None,
            })
            .collect();
        if fields.is_empty() {
            "out".into()
        } else {
            fields.join(".")
        }
    }
}

/// The structural library: the same thirteen for every signature. The
/// scalar family joins them per signature in `library`.
pub const LIBRARY: [Mutant; 13] = [
    Mutant::Empty,
    Mutant::OkEmpty,
    Mutant::Identity,
    Mutant::DropOne,
    Mutant::DuplicateOne,
    Mutant::AppendFabricated,
    Mutant::SwapAdjacent,
    Mutant::Skeleton,
    Mutant::SkeletonPerKey,
    Mutant::SkeletonDropOne,
    Mutant::SkeletonDuplicateOne,
    Mutant::SkeletonAppendFabricated,
    Mutant::SkeletonSwapAdjacent,
];

impl Mutant {
    pub fn name(&self) -> String {
        if let Mutant::Scalar(s) = self {
            return format!("{} = {}", s.path_text(), s.formula.text());
        }
        self.structural_name().to_string()
    }

    fn structural_name(&self) -> &'static str {
        match self {
            Mutant::Empty => "empty",
            Mutant::OkEmpty => "ok_empty",
            Mutant::Identity => "identity",
            Mutant::DropOne => "drop_one",
            Mutant::DuplicateOne => "duplicate_one",
            Mutant::AppendFabricated => "append_fabricated",
            Mutant::SwapAdjacent => "swap_adjacent",
            Mutant::Skeleton => "skeleton",
            Mutant::SkeletonPerKey => "skeleton_per_key",
            Mutant::SkeletonDropOne => "skeleton_drop_one",
            Mutant::SkeletonDuplicateOne => "skeleton_duplicate_one",
            Mutant::SkeletonAppendFabricated => "skeleton_append_fabricated",
            Mutant::SkeletonSwapAdjacent => "skeleton_swap_adjacent",
            Mutant::Scalar(_) => "scalar",
        }
    }

    /// The skeleton family: built from the frame pairs, not from an input
    /// that has the output's shape.
    pub fn is_skeleton(&self) -> bool {
        matches!(
            self,
            Mutant::Skeleton
                | Mutant::SkeletonPerKey
                | Mutant::SkeletonDropOne
                | Mutant::SkeletonDuplicateOne
                | Mutant::SkeletonAppendFabricated
                | Mutant::SkeletonSwapAdjacent
        )
    }

    pub fn describe(&self) -> String {
        if let Mutant::Scalar(s) = self {
            return if s.path.iter().all(|step| matches!(step, Step::Ok)) {
                format!("returns {}", s.formula.text())
            } else {
                format!(
                    "returns the skeleton, or the empty value, with {} = {}",
                    s.path_text(),
                    s.formula.text()
                )
            };
        }
        self.structural_description().to_string()
    }

    fn structural_description(&self) -> &'static str {
        match self {
            Mutant::Scalar(_) => "replaces one scalar of the output",
            Mutant::Empty => "returns the empty value of the output type",
            Mutant::OkEmpty => "returns Ok of the empty value",
            Mutant::Identity => "returns the input unchanged",
            Mutant::DropOne => "returns the input without its first element",
            Mutant::AppendFabricated => "returns the input with a fabricated element appended",
            Mutant::DuplicateOne => "returns the input with its first element repeated",
            Mutant::SwapAdjacent => "returns the input with its first two elements swapped",
            Mutant::Skeleton => {
                "builds the output from the frame pairs: one element per paired input element, matched and preserved fields copied, the rest empty"
            }
            Mutant::SkeletonPerKey => {
                "builds the skeleton with one element per distinct value of the matched fields"
            }
            Mutant::SkeletonDropOne => "builds the skeleton without its first element",
            Mutant::SkeletonDuplicateOne => "builds the skeleton with its first element repeated",
            Mutant::SkeletonAppendFabricated => {
                "builds the skeleton with a fabricated element appended"
            }
            Mutant::SkeletonSwapAdjacent => {
                "builds the skeleton with its first two elements swapped"
            }
        }
    }

    /// A structural mutant by name; the scalar family's names are per
    /// signature and come from `library`.
    pub fn from_name(name: &str) -> Option<Mutant> {
        LIBRARY.into_iter().find(|m| m.structural_name() == name)
    }
}

/// Every mutant for a signature: the structural thirteen and the scalar
/// family the output's scalars and the inputs afford.
pub fn library(scope: &Scope<'_>) -> Vec<Mutant> {
    LIBRARY
        .iter()
        .cloned()
        .chain(scalar_family(scope))
        .collect()
}

/// The `Int` and `Bool` places of the output outside any list, through
/// `Ok` and record fields.
fn scalar_paths(ty: &Type, path: &mut Vec<Step>, out: &mut Vec<(Vec<Step>, Type)>) {
    match ty {
        Type::Int | Type::Bool => out.push((path.clone(), ty.clone())),
        Type::Record { fields, .. } => {
            for (n, t) in fields {
                path.push(Step::Field(n.clone()));
                scalar_paths(t, path, out);
                path.pop();
            }
        }
        Type::Result(ok, _) => {
            path.push(Step::Ok);
            scalar_paths(ok, path, out);
            path.pop();
        }
        _ => {}
    }
}

/// The `Int`s the inputs afford as a base.
fn bases(scope: &Scope<'_>) -> Vec<Base> {
    let mut out = Vec::new();
    for (i, (name, ty)) in scope.inputs.iter().enumerate() {
        match ty {
            Type::Int => out.push(Base::Input(i, name.clone())),
            Type::Record { fields, .. } => {
                for (f, t) in fields {
                    if matches!(t, Type::Int) {
                        out.push(Base::Field(i, name.clone(), f.clone()));
                    }
                }
            }
            Type::List(elem, _) => match elem.as_ref() {
                Type::Int => out.push(Base::Sum(i, name.clone(), None)),
                Type::Record { fields, .. } => {
                    for (f, t) in fields {
                        if matches!(t, Type::Int) {
                            out.push(Base::Sum(i, name.clone(), Some(f.clone())));
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    out
}

/// The scalar family for a signature: for each `Int` of the output, the
/// constants 1 and -1 and, for each base, the base, one more, one less,
/// negated and times 100; for each `Bool`, `true`. A bare output that is
/// an input is `identity`'s and is left out.
pub fn scalar_family(scope: &Scope<'_>) -> Vec<Mutant> {
    let Some(out) = scope.output.as_ref() else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    scalar_paths(out, &mut Vec::new(), &mut paths);
    let bases = bases(scope);
    let identity = identity_source(scope);
    let mut family = Vec::new();
    for (path, ty) in paths {
        let bare = path.iter().all(|s| matches!(s, Step::Ok));
        let mut push = |formula: Formula| {
            family.push(Mutant::Scalar(Scalar {
                path: path.clone(),
                formula,
            }))
        };
        match ty {
            Type::Int => {
                push(Formula::Const(1));
                push(Formula::Const(-1));
                for b in &bases {
                    let is_identity = bare
                        && matches!(b, Base::Input(i, _) if identity.is_some_and(|(j, _)| *i == j));
                    if !is_identity {
                        push(Formula::Base(b.clone()));
                    }
                    push(Formula::PlusOne(b.clone()));
                    push(Formula::MinusOne(b.clone()));
                    push(Formula::Negated(b.clone()));
                    push(Formula::Scaled100(b.clone()));
                }
            }
            Type::Bool => push(Formula::True),
            _ => {}
        }
    }
    family
}

/// Replace the value at a path, through record fields and the `Ok` side
/// of a `Result`, which is taken even where the base value is an `Err`.
fn set_at(v: &mut Value, ty: &Type, path: &[Step], new: Value) -> bool {
    match path.split_first() {
        None => {
            *v = new;
            true
        }
        Some((Step::Field(f), rest)) => {
            let (Value::Record(fields), Type::Record { fields: tys, .. }) = (v, ty) else {
                return false;
            };
            let Some((_, inner)) = fields.iter_mut().find(|(n, _)| n == f) else {
                return false;
            };
            let Some((_, inner_ty)) = tys.iter().find(|(n, _)| n == f) else {
                return false;
            };
            set_at(inner, inner_ty, rest, new)
        }
        Some((Step::Ok, rest)) => {
            let Type::Result(ok, _) = ty else {
                return false;
            };
            if !matches!(v, Value::Ok(_)) {
                *v = Value::Ok(Box::new(empty_value(ok)));
            }
            let Value::Ok(inner) = v else {
                return false;
            };
            set_at(inner, ok, rest, new)
        }
        _ => false,
    }
}

/// A scalar mutant's output: the skeleton where there is a plan, else the
/// empty value, with the scalar replaced.
fn scalar_output(
    s: &Scalar,
    inputs: &[Value],
    scope: &Scope<'_>,
    plan: &[Correspondence],
) -> Option<Value> {
    let out = scope.output.as_ref()?;
    let value = s.formula.value(inputs)?;
    let mut v = if plan.is_empty() {
        empty_value(out)
    } else {
        skeleton_value(out, &mut Vec::new(), inputs, plan, false)
    };
    set_at(&mut v, out, &s.path, value).then_some(v)
}

const EPOCH: &str = "1970-01-01T00:00:00Z";
const ZERO_UUID: &str = "00000000-0000-0000-0000-000000000000";

fn scalar(ty: &Type, text: &str) -> Value {
    match ty {
        Type::Int => Value::Int(0),
        Type::Float => Value::Float(0.0),
        Type::Text | Type::Id => Value::Text(text.to_string()),
        Type::Uuid => Value::Text(ZERO_UUID.to_string()),
        Type::Time => Value::Time {
            instant: 0,
            text: EPOCH.to_string(),
        },
        Type::Bool => Value::Bool(false),
        Type::Option(_) => Value::None,
        Type::List(..) => Value::List(Vec::new()),
        Type::Record { fields, .. } => Value::record(
            fields
                .iter()
                .map(|(n, t)| (n.clone(), scalar(t, text)))
                .collect(),
        ),
        Type::Result(_, err) => Value::Err(Box::new(scalar(err, text))),
        Type::Named(_) | Type::Unknown => Value::None,
    }
}

/// The empty value of a type: `[]`, `""`, 0, the epoch, `None`, a record of
/// empty fields, `Err` of the empty error.
pub fn empty_value(ty: &Type) -> Value {
    scalar(ty, "")
}

/// An element invented from nothing: like the empty value, but Text and Id
/// are `"fab"`, so it is visibly not from the input.
pub fn fabricated(ty: &Type) -> Value {
    scalar(ty, "fab")
}

/// Types equal up to refinements.
pub fn same_shape(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::List(x, _), Type::List(y, _)) => same_shape(x, y),
        (Type::Option(x), Type::Option(y)) => same_shape(x, y),
        (Type::Result(a1, b1), Type::Result(a2, b2)) => same_shape(a1, a2) && same_shape(b1, b2),
        (Type::Record { fields: f, .. }, Type::Record { fields: g, .. }) => {
            f.len() == g.len()
                && f.iter()
                    .all(|(n, t)| g.iter().any(|(m, u)| m == n && same_shape(t, u)))
        }
        (a, b) => a == b,
    }
}

/// The input the identity family copies, as `(index, wrap in Ok)`: the one
/// input with the output's shape, or, when the output is a Result, the one
/// input with its Ok shape.
pub fn identity_source(scope: &Scope<'_>) -> Option<(usize, bool)> {
    let out = scope.output.as_ref()?;
    let matching = |target: &Type| -> Vec<usize> {
        scope
            .inputs
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| same_shape(t, target))
            .map(|(i, _)| i)
            .collect()
    };
    let direct = matching(out);
    if direct.len() == 1 {
        return Some((direct[0], false));
    }
    if direct.is_empty() {
        if let Type::Result(ok, _) = out {
            let wrapped = matching(ok);
            if wrapped.len() == 1 {
                return Some((wrapped[0], true));
            }
        }
    }
    None
}

/// The list type the identity family edits, if the identity is a list.
fn identity_list(scope: &Scope<'_>) -> Option<Type> {
    let (_, wrap) = identity_source(scope)?;
    let out = scope.output.as_ref()?;
    let ty = if wrap {
        match out {
            Type::Result(ok, _) => ok.as_ref(),
            _ => return None,
        }
    } else {
        out
    };
    match ty {
        Type::List(..) => Some(ty.clone()),
        _ => None,
    }
}

/// The value at a path inside a value, through record fields and Result
/// sides.
fn value_at<'v>(v: &'v Value, path: &[Step]) -> Option<&'v Value> {
    let mut cur = v;
    for step in path {
        cur = match (step, cur) {
            (Step::Field(f), Value::Record(_)) => cur.field(f)?,
            (Step::Ok, Value::Ok(x)) | (Step::Err, Value::Err(x)) => x,
            _ => return None,
        };
    }
    Some(cur)
}

/// The type at a path inside a type.
fn type_at<'t>(t: &'t Type, path: &[Step]) -> Option<&'t Type> {
    let mut cur = t;
    for step in path {
        cur = match (step, cur) {
            (Step::Field(f), Type::Record { fields, .. }) => {
                &fields.iter().find(|(n, _)| n == f)?.1
            }
            (Step::Ok, Type::Result(ok, _)) => ok,
            (Step::Err, Type::Result(_, err)) => err,
            (Step::Each, Type::List(elem, _)) => elem,
            _ => return None,
        };
    }
    Some(cur)
}

/// The skeleton plan: one correspondence per output path, the first the
/// frame pairs state for it.
pub fn skeleton_plan(pairs: &[crate::frame::FramePair]) -> Vec<Correspondence> {
    let mut plan: Vec<Correspondence> = Vec::new();
    for p in pairs {
        if !plan.iter().any(|c| c.out_path == p.correspondence.out_path) {
            plan.push(p.correspondence.clone());
        }
    }
    plan
}

fn skeleton_record(fields: &[(String, Type)], c: &Correspondence, src: Option<&Value>) -> Value {
    Value::record(
        fields
            .iter()
            .map(|(n, t)| {
                let from = c
                    .matched
                    .iter()
                    .find(|(of, _)| of == n)
                    .map(|(_, inf)| inf.as_str())
                    .or_else(|| c.preserved.contains(n).then_some(n.as_str()));
                let v = from
                    .and_then(|f| src.and_then(|s| s.field(f)).cloned())
                    .unwrap_or_else(|| empty_value(t));
                (n.clone(), v)
            })
            .collect(),
    )
}

fn skeleton_list(elem: &Type, c: &Correspondence, inputs: &[Value], per_key: bool) -> Value {
    let src = match inputs.get(c.input).and_then(|v| value_at(v, &c.input_path)) {
        Some(Value::List(xs)) => xs,
        _ => return Value::List(Vec::new()),
    };
    let Some(fields) = elem.record_fields() else {
        return Value::List(Vec::new());
    };
    let mut out = Vec::new();
    let mut seen: Vec<Vec<Option<Value>>> = Vec::new();
    for s in src {
        if per_key {
            let key: Vec<Option<Value>> = c
                .matched
                .iter()
                .map(|(_, inf)| s.field(inf).cloned())
                .collect();
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
        }
        out.push(skeleton_record(fields, c, Some(s)));
    }
    Value::List(out)
}

fn skeleton_value(
    ty: &Type,
    path: &mut Vec<Step>,
    inputs: &[Value],
    plan: &[Correspondence],
    per_key: bool,
) -> Value {
    match ty {
        Type::List(elem, _) => match plan.iter().find(|c| c.out_path == *path) {
            Some(c) => skeleton_list(elem, c, inputs, per_key),
            None => Value::List(Vec::new()),
        },
        Type::Record { fields, .. } => match plan.iter().find(|c| c.out_path == *path) {
            Some(c) => {
                let src = inputs.get(c.input).and_then(|v| value_at(v, &c.input_path));
                skeleton_record(fields, c, src)
            }
            None => Value::record(
                fields
                    .iter()
                    .map(|(n, t)| {
                        path.push(Step::Field(n.clone()));
                        let v = skeleton_value(t, path, inputs, plan, per_key);
                        path.pop();
                        (n.clone(), v)
                    })
                    .collect(),
            ),
        },
        Type::Result(ok, err) => {
            path.push(Step::Ok);
            let inside = plan.iter().any(|c| c.out_path.starts_with(path));
            let v = if inside {
                Value::Ok(Box::new(skeleton_value(ok, path, inputs, plan, per_key)))
            } else {
                Value::Err(Box::new(empty_value(err)))
            };
            path.pop();
            v
        }
        other => empty_value(other),
    }
}

/// The output path of the first list the skeleton builds: what the
/// skeleton tweaks edit.
fn skeleton_list_path(out: &Type, plan: &[Correspondence]) -> Option<Vec<Step>> {
    plan.iter()
        .find(|c| matches!(type_at(out, &c.out_path), Some(Type::List(..))))
        .map(|c| c.out_path.clone())
}

fn edit_at(v: &mut Value, path: &[Step], edit: &dyn Fn(&mut Vec<Value>) -> bool) -> bool {
    if path.is_empty() {
        return match v {
            Value::List(xs) => edit(xs),
            _ => false,
        };
    }
    match (&path[0], v) {
        (Step::Field(f), Value::Record(fields)) => fields
            .iter_mut()
            .find(|(n, _)| n == f)
            .is_some_and(|(_, x)| edit_at(x, &path[1..], edit)),
        (Step::Ok, Value::Ok(x)) | (Step::Err, Value::Err(x)) => edit_at(x, &path[1..], edit),
        _ => false,
    }
}

/// Whether a mutant can be built for this signature at all.
pub fn applies(m: &Mutant, scope: &Scope<'_>, plan: &[Correspondence]) -> bool {
    let Some(out) = scope.output.as_ref() else {
        return false;
    };
    match m {
        Mutant::Scalar(_) => true,
        Mutant::Empty => true,
        Mutant::OkEmpty => matches!(out, Type::Result(..)),
        Mutant::Identity => identity_source(scope).is_some(),
        Mutant::DropOne
        | Mutant::DuplicateOne
        | Mutant::AppendFabricated
        | Mutant::SwapAdjacent => identity_list(scope).is_some(),
        Mutant::Skeleton => !plan.is_empty(),
        Mutant::SkeletonPerKey => plan.iter().any(|c| !c.matched.is_empty()),
        _ => skeleton_list_path(out, plan).is_some(),
    }
}

/// The mutant's output for given inputs, or `None` where it has none on
/// those inputs (nothing to drop from an empty list).
pub fn apply(
    m: &Mutant,
    inputs: &[Value],
    scope: &Scope<'_>,
    plan: &[Correspondence],
) -> Option<Value> {
    let out = scope.output.as_ref()?;
    match m {
        Mutant::Scalar(s) => scalar_output(s, inputs, scope, plan),
        Mutant::Empty => Some(empty_value(out)),
        Mutant::OkEmpty => match out {
            Type::Result(ok, _) => Some(Value::Ok(Box::new(empty_value(ok)))),
            _ => None,
        },
        m if m.is_skeleton() => {
            if !applies(m, scope, plan) {
                return None;
            }
            let per_key = *m == Mutant::SkeletonPerKey;
            let mut v = skeleton_value(out, &mut Vec::new(), inputs, plan, per_key);
            if matches!(m, Mutant::Skeleton | Mutant::SkeletonPerKey) {
                return Some(v);
            }
            let path = skeleton_list_path(out, plan)?;
            let elem = match type_at(out, &path) {
                Some(Type::List(elem, _)) => elem.as_ref().clone(),
                _ => return None,
            };
            let ok = match m {
                Mutant::SkeletonDropOne => edit_at(&mut v, &path, &|xs| {
                    if xs.is_empty() {
                        return false;
                    }
                    xs.remove(0);
                    true
                }),
                Mutant::SkeletonDuplicateOne => edit_at(&mut v, &path, &|xs| {
                    if xs.is_empty() {
                        return false;
                    }
                    let first = xs[0].clone();
                    xs.insert(0, first);
                    true
                }),
                Mutant::SkeletonAppendFabricated => edit_at(&mut v, &path, &|xs| {
                    xs.push(fabricated(&elem));
                    true
                }),
                Mutant::SkeletonSwapAdjacent => edit_at(&mut v, &path, &|xs| {
                    if xs.len() < 2 {
                        return false;
                    }
                    xs.swap(0, 1);
                    true
                }),
                _ => false,
            };
            ok.then_some(v)
        }
        _ => {
            let (idx, wrap) = identity_source(scope)?;
            let base = inputs.get(idx)?.clone();
            let wrapped = |v: Value| if wrap { Value::Ok(Box::new(v)) } else { v };
            if *m == Mutant::Identity {
                return Some(wrapped(base));
            }
            let list_ty = identity_list(scope)?;
            let Type::List(elem, _) = &list_ty else {
                return None;
            };
            let Value::List(xs) = base else {
                return None;
            };
            let edited = match m {
                Mutant::DropOne => {
                    if xs.is_empty() {
                        return None;
                    }
                    xs[1..].to_vec()
                }
                Mutant::DuplicateOne => {
                    if xs.is_empty() {
                        return None;
                    }
                    let mut v = xs.clone();
                    v.insert(0, xs[0].clone());
                    v
                }
                Mutant::AppendFabricated => {
                    let mut v = xs.clone();
                    v.push(fabricated(elem));
                    v
                }
                Mutant::SwapAdjacent => {
                    if xs.len() < 2 {
                        return None;
                    }
                    let mut v = xs.clone();
                    v.swap(0, 1);
                    v
                }
                _ => return None,
            };
            Some(wrapped(Value::List(edited)))
        }
    }
}

/// A clause of the obligation, as the report names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clause {
    Property {
        line: usize,
        text: String,
    },
    Frame {
        line: usize,
        description: String,
    },
    /// The output type itself.
    Boundary,
}

impl Clause {
    pub fn line(&self) -> Option<usize> {
        match self {
            Clause::Property { line, .. } | Clause::Frame { line, .. } => Some(*line),
            Clause::Boundary => None,
        }
    }

    pub fn text(&self) -> String {
        match self {
            Clause::Property { text, .. } => text.clone(),
            Clause::Frame { description, .. } => format!("frame {description}"),
            Clause::Boundary => "the output type".to_string(),
        }
    }
}

/// A clause that caught a mutant: on how many inputs, and the first.
#[derive(Debug, Clone)]
pub struct Kill {
    pub clause: Clause,
    /// The boundary message, or an evaluation error, when it was not a plain `false`.
    pub message: Option<String>,
    pub inputs: Vec<Value>,
    pub output: Value,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutantVerdict {
    /// No clause was false on any input and every output passed the boundary.
    Slipped,
    Caught,
    NotApplicable,
}

/// The input a report shows for a mutant: an example that caught it when
/// one did, else the first input it ran on.
#[derive(Debug, Clone)]
pub struct Shown {
    pub inputs: Vec<Value>,
    pub output: Value,
    pub expected: Option<Value>,
    /// 1-based example number, when the input is an example's.
    pub example: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MutantRun {
    pub mutant: Mutant,
    pub verdict: MutantVerdict,
    pub inputs_tried: usize,
    /// Inputs on which at least one clause caught it.
    pub caught_inputs: usize,
    pub kills: Vec<Kill>,
    /// Clauses never false for it.
    pub slipped: Vec<Clause>,
    /// 1-based examples whose expected output differs from the mutant's.
    pub example_catches: Vec<usize>,
    pub shown: Option<Shown>,
}

/// An input on which two different outputs pass every clause.
#[derive(Debug, Clone)]
pub struct Underdetermined {
    pub inputs: Vec<Value>,
    pub example: Option<usize>,
    pub a: Value,
    pub b: Value,
    /// `a` is the example's expected output.
    pub a_is_expected: bool,
    pub candidates_tried: usize,
    /// The index of the `accept` line this input satisfies, when one does:
    /// a deliberate non-decision, reported as accepted.
    pub accepted_by: Option<usize>,
    /// A slipped mutant whose output on this input is one of the two: the
    /// same hole, seen from the search.
    pub same_as: Option<Mutant>,
}

#[derive(Debug, Clone, Default)]
pub struct MutantReport {
    pub obligation: String,
    pub obligation_hash: String,
    pub example_inputs: usize,
    pub generated_inputs: usize,
    pub buckets: Vec<Bucket>,
    pub mutants: Vec<MutantRun>,
    pub search: Vec<Underdetermined>,
    /// Inputs the search covered: every example plus the budget's share of generated ones.
    pub search_inputs: usize,
    pub search_cap: usize,
    /// Clauses the search could not evaluate on any candidate, with the first error.
    pub unevaluable: Vec<(Clause, String)>,
    /// Clauses that caught no applicable mutant.
    pub uncaught: Vec<Clause>,
    /// Clauses that cannot read the output: claims about the inputs alone,
    /// credited with no catch, since on an input where one is false it is
    /// false whatever the output.
    pub input_only: Vec<Clause>,
    /// For each of `input_only`, the inputs tried on which it is false.
    pub input_only_false: Vec<usize>,
    /// Inputs tried on which some clause about the inputs alone is false:
    /// no output passes there, so the obligation is unsatisfiable on them.
    pub excluded_inputs: usize,
    /// Every clause over each example's own pair, as `check` has it.
    pub against: Against,
    /// What ran; `mutants` never runs the performance, so `runs` is zero.
    pub ran: crate::checker::Ran,
}

impl MutantReport {
    pub fn applicable(&self) -> usize {
        self.mutants
            .iter()
            .filter(|m| m.verdict != MutantVerdict::NotApplicable)
            .count()
    }

    pub fn slipped(&self) -> Vec<&MutantRun> {
        self.mutants
            .iter()
            .filter(|m| m.verdict == MutantVerdict::Slipped)
            .collect()
    }

    pub fn caught(&self) -> usize {
        self.mutants
            .iter()
            .filter(|m| m.verdict == MutantVerdict::Caught)
            .count()
    }

    /// Underdetermined inputs no `accept` covers.
    pub fn open(&self) -> Vec<&Underdetermined> {
        self.search
            .iter()
            .filter(|u| u.accepted_by.is_none())
            .collect()
    }

    /// Underdetermined inputs an `accept` covers: decided, not forgotten.
    pub fn accepted(&self) -> Vec<&Underdetermined> {
        self.search
            .iter()
            .filter(|u| u.accepted_by.is_some())
            .collect()
    }

    /// The underdetermination rate's numerator for this obligation: a
    /// slipping mutant or an input with two passing outputs that no
    /// `accept` covers.
    pub fn underdetermined(&self) -> bool {
        !self.slipped().is_empty() || !self.open().is_empty()
    }

    pub fn run(&self, m: Mutant) -> Option<&MutantRun> {
        self.mutants.iter().find(|r| r.mutant == m)
    }
}

struct Case {
    inputs: Vec<Value>,
    expected: Option<Value>,
    example: Option<usize>,
}

/// How much pairs of distinct elements agree, over every list in the
/// inputs (an input, or a list field of a record input): the most fields
/// any one pair shares, then the total over pairs. Identical elements do
/// not count: nothing distinguishes them to begin with.
pub fn agreement(inputs: &[Value]) -> (usize, usize) {
    fn pairs(xs: &[Value], best: &mut usize, total: &mut usize) {
        for (i, a) in xs.iter().enumerate() {
            for b in &xs[..i] {
                if a == b {
                    continue;
                }
                if let (Value::Record(fa), Value::Record(fb)) = (a, b) {
                    let shared = fa
                        .iter()
                        .filter(|(n, v)| fb.iter().any(|(m, w)| m == n && w == v))
                        .count();
                    *best = (*best).max(shared);
                    *total += shared;
                }
            }
        }
    }
    let (mut best, mut total) = (0, 0);
    for v in inputs {
        match v {
            Value::List(xs) => pairs(xs, &mut best, &mut total),
            Value::Record(fields) => {
                for (_, x) in fields {
                    if let Value::List(xs) = x {
                        pairs(xs, &mut best, &mut total);
                    }
                }
            }
            _ => {}
        }
    }
    (best, total)
}

fn permutations(xs: &[Value]) -> Vec<Vec<Value>> {
    if xs.len() <= 1 {
        return vec![xs.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..xs.len() {
        let mut rest = xs.to_vec();
        let head = rest.remove(i);
        for mut tail in permutations(&rest) {
            tail.insert(0, head.clone());
            out.push(tail);
        }
    }
    out
}

/// Output candidates drawn from the input: when the output is a list, or a
/// `Result` with a list on either side, every sub-multiset of the matching
/// input lists' elements and of the named values of the element type,
/// smallest first, in every order while the pool is small enough for that
/// to be cheap. A filter, a selection, a reordering or an error list has
/// its answer here.
pub fn drawn_candidates(
    out_ty: &Type,
    inputs: &[Value],
    scope: &Scope<'_>,
    named: &[(Value, Type)],
) -> Vec<Value> {
    match out_ty {
        Type::List(..) => drawn_list(out_ty, inputs, scope, named, None),
        Type::Result(ok, err) => {
            let mut out = Vec::new();
            if matches!(**ok, Type::List(..)) {
                out.extend(drawn_list(ok, inputs, scope, named, Some(true)));
            }
            if matches!(**err, Type::List(..)) {
                out.extend(drawn_list(err, inputs, scope, named, Some(false)));
            }
            out
        }
        _ => Vec::new(),
    }
}

fn drawn_list(
    list_ty: &Type,
    inputs: &[Value],
    scope: &Scope<'_>,
    named: &[(Value, Type)],
    wrap: Option<bool>,
) -> Vec<Value> {
    let Type::List(elem, _) = list_ty else {
        return Vec::new();
    };
    let mut pool: Vec<Value> = Vec::new();
    for (v, (_, t)) in inputs.iter().zip(&scope.inputs) {
        if let (Value::List(xs), Type::List(e, _)) = (v, t) {
            if same_shape(e, elem) {
                pool.extend(xs.iter().cloned());
            }
        }
    }
    for (v, t) in named {
        if same_shape(t, elem) && !pool.contains(v) {
            pool.push(v.clone());
        }
    }
    if pool.is_empty() || pool.len() > 8 {
        return Vec::new();
    }
    let mut subsets: Vec<Vec<Value>> = (0..(1u32 << pool.len()))
        .map(|mask| {
            pool.iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, v)| v.clone())
                .collect()
        })
        .collect();
    subsets.sort_by_key(|s| s.len());
    let mut out = Vec::new();
    for subset in subsets {
        let orderings = if pool.len() <= 6 {
            permutations(&subset)
        } else {
            let mut rev = subset.clone();
            rev.reverse();
            vec![subset, rev]
        };
        for o in orderings {
            let v = Value::List(o);
            out.push(match wrap {
                Some(true) => Value::Ok(Box::new(v)),
                Some(false) => Value::Err(Box::new(v)),
                None => v,
            });
        }
    }
    out
}

fn kill(run: &mut MutantRun, clause: Clause, message: Option<String>, case: &Case, out: &Value) {
    if let Some(k) = run.kills.iter_mut().find(|k| k.clause == clause) {
        k.count += 1;
    } else {
        run.kills.push(Kill {
            clause,
            message,
            inputs: case.inputs.clone(),
            output: out.clone(),
            count: 1,
        });
    }
}

impl<'g> Checker<'g> {
    /// Whether evaluating an expression calls a performance (a graph
    /// reference, or the obligation itself), directly or through an
    /// abbreviation.
    pub fn has_graph_call(&self, e: &Expr) -> bool {
        fn walk(scope: &Scope<'_>, e: &Expr, depth: usize) -> bool {
            let mut found = false;
            e.walk(&mut |sub| {
                if found {
                    return;
                }
                if let Expr::Call(name, _, _) = sub {
                    if BUILTINS.contains(&name.as_str()) {
                        return;
                    }
                    match scope.abbrev(name) {
                        Some(a) if depth < 8 => found = walk(scope, &a.body, depth + 1),
                        Some(_) => {}
                        None => found = true,
                    }
                }
                if let Expr::Name(name, _) = sub {
                    if let Some(a) = scope.abbrev(name) {
                        if depth < 8 && walk(scope, &a.body, depth + 1) {
                            found = true;
                        }
                    }
                }
            });
            found
        }
        walk(&self.scope, e, 0)
    }

    /// Whether a clause's truth can depend on the output: it names `out`,
    /// ranges over leftovers, or calls the obligation itself, directly or
    /// through an abbreviation.
    pub fn reads_output(&self, e: &Expr) -> bool {
        fn walk(scope: &Scope<'_>, own: &str, e: &Expr, depth: usize) -> bool {
            let mut found = false;
            e.walk(&mut |sub| {
                if found {
                    return;
                }
                match sub {
                    Expr::Name(n, _) if n == "out" => found = true,
                    Expr::Call(n, _, _) if n == own => found = true,
                    // `leftover` ranges over the domain less `out`.
                    Expr::Quantifier(q) if q.leftover => found = true,
                    Expr::Name(n, _) | Expr::Call(n, _, _) => {
                        if let Some(a) = scope.abbrev(n) {
                            if depth < 8 && walk(scope, own, &a.body, depth + 1) {
                                found = true;
                            }
                        }
                    }
                    _ => {}
                }
            });
            found
        }
        walk(&self.scope, &self.fun.name, e, 0)
    }

    /// Every clause a candidate output is judged by: the properties, then
    /// the frame rule's implicit ones.
    pub fn clauses(&self) -> Vec<(Clause, Expr)> {
        let fun = self.fun;
        let mut clauses: Vec<(Clause, Expr)> = fun
            .properties
            .iter()
            .map(|p| {
                (
                    Clause::Property {
                        line: p.line,
                        text: p.text.clone(),
                    },
                    p.expr.clone(),
                )
            })
            .collect();
        for pair in frame_pairs(&self.scope, fun) {
            clauses.push((
                Clause::Frame {
                    line: pair.line,
                    description: pair.description,
                },
                pair.implicit,
            ));
        }
        clauses
    }

    /// An output that passes every clause on these inputs, if the
    /// candidate search finds one, and how many candidates it tried. This
    /// is how a generated-input failure is attributed: a passing output
    /// exists and the performance did not produce it, or none does and the
    /// obligation is unsatisfiable there. The outputs already in hand on
    /// the input come first, before the pool: the example's, the
    /// performance's, and the library's, the skeletons among them, since a
    /// mutant that passes every clause on an input is a passing output
    /// there, whatever it does elsewhere.
    pub fn find_passing(&mut self, inputs: &[Value]) -> (Option<Value>, usize) {
        let cap = self.budget.search_candidates;
        if cap == 0 {
            return (None, 0);
        }
        let out_ty = self.output_type().clone();
        let clauses = self.clauses();
        let named = self.named_values_typed();
        let mut typed: Vec<(&Value, &Type)> = inputs
            .iter()
            .zip(self.scope.inputs.iter().map(|(_, t)| t))
            .collect();
        typed.extend(named.iter().map(|(v, t)| (v, t)));
        let domain = Domain::seeded(&typed);
        let drawn = drawn_candidates(&out_ty, inputs, &self.scope, &named);
        // The outputs in hand, in order: an example's on its own input (a
        // contradicted one fails the clauses below like any candidate), the
        // performance's where it ran on the input, then the mutants'.
        let mut seeds: Vec<Value> = Vec::new();
        for ex in &self.fun.examples {
            if self.example_inputs(ex).is_ok_and(|i| i == inputs) {
                if let Ok(e) = self.expected_output(ex, inputs) {
                    seeds.push(e);
                }
            }
        }
        seeds.extend(
            self.runs
                .iter()
                .filter(|r| r.inputs == inputs)
                .map(|r| r.output.clone()),
        );
        seeds.extend(self.generated.iter().filter_map(|g| match &g.outcome {
            Outcome::Output(v) if g.inputs == inputs => Some(v.clone()),
            _ => None,
        }));
        seeds.extend(
            library(&self.scope)
                .iter()
                .filter(|m| applies(m, &self.scope, &self.plan))
                .filter_map(|m| apply(m, inputs, &self.scope, &self.plan)),
        );
        let mut tried = 0;
        for cand in seeds
            .into_iter()
            .chain(candidates(&out_ty, &domain, cap, drawn))
        {
            tried += 1;
            if from_json(&to_json(&cand), &out_ty).is_err() {
                continue;
            }
            // A clause calling a performance that cannot start is unchecked,
            // as it is for the mutants' catches: it neither passes nor fails
            // a candidate, or one graph reference with no performance yet
            // would leave every input without a passing output.
            let all = clauses
                .iter()
                .all(|(_, e)| match self.holds(e, inputs, &cand) {
                    Ok(b) => b,
                    Err(m) => self.unstartable(&m),
                });
            if all {
                return (Some(cand), tried);
            }
        }
        (None, tried)
    }

    /// The underdetermination check: the library against the obligation
    /// over the examples and the generated inputs, then the search.
    pub fn mutants(&mut self) -> MutantReport {
        let against = self.against();
        let mut cases: Vec<Case> = Vec::new();
        for (i, ex) in self.fun.examples.iter().enumerate() {
            if let Ok(inputs) = self.example_inputs(ex) {
                let expected = self.expected_output(ex, &inputs).ok();
                cases.push(Case {
                    inputs,
                    expected,
                    example: Some(i + 1),
                });
            }
        }
        let example_inputs = cases.len();
        let generated = self.generated_inputs();
        for inputs in &generated.inputs {
            cases.push(Case {
                inputs: inputs.clone(),
                expected: None,
                example: None,
            });
        }
        let out_ty = self.output_type().clone();
        let clauses = self.clauses();
        // A clause about the inputs alone is false whatever the output on an
        // input where it is false: no catch of its is the mutant's doing.
        // The frame rule always pairs an output record with an input one.
        let reads: Vec<bool> = clauses
            .iter()
            .map(|(c, e)| matches!(c, Clause::Frame { .. }) || self.reads_output(e))
            .collect();
        // Where a clause about the inputs alone is false, it is false whatever
        // the output: count those inputs, per clause and in all.
        let only: Vec<Expr> = clauses
            .iter()
            .zip(&reads)
            .filter(|(_, r)| !**r)
            .map(|((_, e), _)| e.clone())
            .collect();
        let mut input_only_false = vec![0; only.len()];
        let mut excluded_inputs = 0;
        for case in &cases {
            let mut any = false;
            for (k, e) in only.iter().enumerate() {
                if matches!(self.holds(e, &case.inputs, &Value::None), Ok(false)) {
                    input_only_false[k] += 1;
                    any = true;
                }
            }
            if any {
                excluded_inputs += 1;
            }
        }
        let unordered: Vec<Vec<Step>> = self.unordered().to_vec();
        let mut unchecked: Vec<(Clause, String)> = Vec::new();
        let mut mutants = Vec::new();
        for m in library(&self.scope) {
            let mut run = MutantRun {
                mutant: m.clone(),
                verdict: MutantVerdict::NotApplicable,
                inputs_tried: 0,
                caught_inputs: 0,
                kills: Vec::new(),
                slipped: Vec::new(),
                example_catches: Vec::new(),
                shown: None,
            };
            if !applies(&m, &self.scope, &self.plan) {
                mutants.push(run);
                continue;
            }
            self.mutant = Some(m.clone());
            for case in &cases {
                let Some(out) = apply(&m, &case.inputs, &self.scope, &self.plan) else {
                    continue;
                };
                run.inputs_tried += 1;
                let before = run.kills.iter().map(|k| k.count).sum::<usize>();
                if let Err(b) = from_json(&to_json(&out), &out_ty) {
                    kill(&mut run, Clause::Boundary, Some(b.to_string()), case, &out);
                } else {
                    for ((clause, expr), _) in clauses.iter().zip(&reads).filter(|(_, r)| **r) {
                        match self.holds(expr, &case.inputs, &out) {
                            Ok(true) => {}
                            Ok(false) => kill(&mut run, clause.clone(), None, case, &out),
                            // A module that could not start fails every
                            // output alike: the clause is unchecked, and no
                            // mutant is caught by it.
                            Err(e) if self.unstartable(&e) => {
                                if !unchecked.iter().any(|(c, _)| c == clause) {
                                    unchecked.push((clause.clone(), e));
                                }
                            }
                            Err(e) => kill(&mut run, clause.clone(), Some(e), case, &out),
                        }
                    }
                }
                if run.kills.iter().map(|k| k.count).sum::<usize>() > before {
                    run.caught_inputs += 1;
                }
                let example_caught = case
                    .expected
                    .as_ref()
                    .is_some_and(|exp| compare(exp, &out, &unordered).is_err());
                if example_caught {
                    run.example_catches.push(case.example.unwrap_or(0));
                }
                let replace = match &run.shown {
                    None => true,
                    Some(s) => example_caught && s.example.is_none(),
                };
                if replace {
                    run.shown = Some(Shown {
                        inputs: case.inputs.clone(),
                        output: out.clone(),
                        expected: case.expected.clone(),
                        example: case.example,
                    });
                }
            }
            self.mutant = None;
            run.slipped = clauses
                .iter()
                .map(|(c, _)| c.clone())
                .filter(|c| !run.kills.iter().any(|k| k.clause == *c))
                .collect();
            run.verdict = if run.inputs_tried == 0 {
                MutantVerdict::NotApplicable
            } else if run.kills.is_empty() {
                MutantVerdict::Slipped
            } else {
                MutantVerdict::Caught
            };
            mutants.push(run);
        }
        let uncaught: Vec<Clause> = clauses
            .iter()
            .map(|(c, _)| c.clone())
            .filter(|c| {
                !mutants
                    .iter()
                    .any(|r| r.kills.iter().any(|k| k.clause == *c))
            })
            .collect();
        let (mut search, search_inputs, mut unevaluable) =
            self.search(&cases, &clauses, &unordered);
        for (c, e) in unchecked {
            if !unevaluable.iter().any(|(u, _)| *u == c) {
                unevaluable.push((c, e));
            }
        }
        // An underdetermined input whose second output is what a slipped
        // mutant returns there is the same hole, seen twice.
        let slipped: Vec<Mutant> = mutants
            .iter()
            .filter(|m| m.verdict == MutantVerdict::Slipped)
            .map(|m| m.mutant.clone())
            .collect();
        for u in &mut search {
            for m in &slipped {
                if let Some(v) = apply(m, &u.inputs, &self.scope, &self.plan) {
                    if compare(&v, &u.a, &unordered).is_ok()
                        || compare(&v, &u.b, &unordered).is_ok()
                    {
                        u.same_as = Some(m.clone());
                        break;
                    }
                }
            }
        }

        let ran = crate::checker::Ran {
            examples: against.evaluated(),
            runs: 0,
            mutants: mutants.iter().map(|m| m.inputs_tried).sum(),
            searched: search_inputs,
            performance: self.performance.is_file(),
        };
        MutantReport {
            obligation: self.fun.name.clone(),
            obligation_hash: surety_syntax::normalize::canonical_file(self.graph, self.idx)
                .map(|c| c.hash)
                .unwrap_or_default(),
            example_inputs,
            generated_inputs: generated.inputs.len(),
            buckets: generated.buckets,
            mutants,
            search,
            search_inputs,
            search_cap: self.budget.search_candidates,
            unevaluable,
            uncaught,
            input_only: clauses
                .iter()
                .zip(&reads)
                .filter(|(_, r)| !**r)
                .map(|((c, _), _)| c.clone())
                .collect(),
            input_only_false,
            excluded_inputs,
            ran,
            against,
        }
    }

    /// For each example input and the budget's share of generated ones:
    /// candidate outputs of the output type, built from the input's own
    /// scalars and the named values, smallest first; the first two that
    /// pass every clause and differ under the order claims are a finding.
    /// An example's expected output counts as the first.
    fn search(
        &mut self,
        cases: &[Case],
        clauses: &[(Clause, Expr)],
        unordered: &[Vec<Step>],
    ) -> (Vec<Underdetermined>, usize, Vec<(Clause, String)>) {
        let cap = self.budget.search_candidates;
        if cap == 0 {
            return (Vec::new(), 0, Vec::new());
        }
        // Every example, then the generated inputs whose list elements agree
        // on the most fields while differing somewhere: ties and near
        // duplicates are where an obligation's matches become ambiguous.
        let mut chosen: Vec<&Case> = cases.iter().filter(|c| c.example.is_some()).collect();
        let mut generated: Vec<((usize, usize), &Case)> = cases
            .iter()
            .filter(|c| c.example.is_none())
            .map(|c| (agreement(&c.inputs), c))
            .collect();
        generated.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        chosen.extend(
            generated
                .iter()
                .take(self.budget.search_inputs)
                .map(|(_, c)| *c),
        );
        let out_ty = self.output_type().clone();
        // Clauses that call no performance are evaluated first, so most
        // candidates die before a Python round trip.
        let expensive: Vec<bool> = clauses
            .iter()
            .map(|(_, e)| self.has_graph_call(e))
            .collect();
        let mut order: Vec<usize> = (0..clauses.len()).collect();
        order.sort_by_key(|i| expensive[*i]);
        let named = self.named_values_typed();
        let mut evaluated = vec![0usize; clauses.len()];
        let mut errors: Vec<(usize, Option<String>)> =
            (0..clauses.len()).map(|_| (0, None)).collect();
        let mut findings = Vec::new();
        let mut covered = 0;
        for case in chosen {
            covered += 1;
            let mut typed: Vec<(&Value, &Type)> = case
                .inputs
                .iter()
                .zip(self.scope.inputs.iter().map(|(_, t)| t))
                .collect();
            typed.extend(named.iter().map(|(v, t)| (v, t)));
            let domain = Domain::seeded(&typed);
            let drawn = drawn_candidates(&out_ty, &case.inputs, &self.scope, &named);
            // An example's output counts as the first passing output only
            // once it passes every clause: a contradicted example is the
            // obligation's finding, never one side of a gap.
            let mut passing: Vec<Value> = Vec::new();
            if let Some(e) = &case.expected {
                if clauses
                    .iter()
                    .all(|(_, x)| matches!(self.holds(x, &case.inputs, e), Ok(true)))
                {
                    passing.push(e.clone());
                }
            }
            let seeded = !passing.is_empty();
            for (tried, cand) in candidates(&out_ty, &domain, cap, drawn)
                .into_iter()
                .enumerate()
            {
                let tried = tried + 1;
                if passing.iter().any(|p| compare(p, &cand, unordered).is_ok()) {
                    continue;
                }
                if from_json(&to_json(&cand), &out_ty).is_err() {
                    continue;
                }
                let mut ok = true;
                for &i in &order {
                    evaluated[i] += 1;
                    match self.holds(&clauses[i].1, &case.inputs, &cand) {
                        Ok(true) => {}
                        Ok(false) => {
                            ok = false;
                            break;
                        }
                        Err(e) => {
                            errors[i].0 += 1;
                            if errors[i].1.is_none() {
                                errors[i].1 = Some(e);
                            }
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    continue;
                }
                passing.push(cand);
                if passing.len() >= 2 {
                    let fun = self.fun;
                    let accepted_by = fun
                        .accepts
                        .iter()
                        .position(|a| self.accepts_input(&a.expr, &case.inputs));
                    findings.push(Underdetermined {
                        inputs: case.inputs.clone(),
                        example: case.example,
                        a: passing[0].clone(),
                        b: passing[1].clone(),
                        a_is_expected: seeded,
                        candidates_tried: tried,
                        accepted_by,
                        same_as: None,
                    });
                    break;
                }
            }
        }
        let unevaluable = clauses
            .iter()
            .enumerate()
            .filter(|(i, _)| evaluated[*i] > 0 && errors[*i].0 == evaluated[*i])
            .map(|(i, (c, _))| (c.clone(), errors[i].1.clone().unwrap_or_default()))
            .collect();
        (findings, covered, unevaluable)
    }
}
