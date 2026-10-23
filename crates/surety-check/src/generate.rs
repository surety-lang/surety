//! Generated inputs (slice 2, acceptance test 1): example-sized values built
//! from the obligation's types and seeded with the literals it uses.
//!
//! Nothing here is random. Domains are collected in source order, the
//! sampler is a counter-based hash with a fixed seed, and small spaces are
//! enumerated outright, so the same obligation always yields the same
//! inputs in the same order.

use std::collections::HashSet;

use surety_syntax::{Expr, Refinement, Type};

use crate::value::{Value, parse_rfc3339, to_json};

/// Lists are generated at every length from 0 to this, or to the
/// refinement's bound if lower. Four because P4's own tie case needs a
/// fourth element.
pub const MAX_LEN: usize = 4;

/// How much the generated tier and the mutant search may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budget {
    /// Generated inputs per obligation, after the example inputs.
    pub generated: usize,
    /// Generated inputs the mutant search covers, besides every example.
    pub search_inputs: usize,
    /// Output candidates the search tries per input.
    pub search_candidates: usize,
    /// Whether `check` runs the underdetermination check too and carries
    /// its verdict in the report.
    pub mutants: bool,
}

impl Default for Budget {
    fn default() -> Budget {
        Budget {
            generated: 500,
            search_inputs: 8,
            search_candidates: 600,
            mutants: true,
        }
    }
}

impl Budget {
    /// Slice 1 behaviour: examples only, no search.
    pub fn none() -> Budget {
        Budget {
            generated: 0,
            search_inputs: 0,
            search_candidates: 0,
            mutants: false,
        }
    }
}

/// The scalar values generation draws from, per primitive type.
#[derive(Debug, Clone, Default)]
pub struct Domain {
    pub ints: Vec<i128>,
    pub floats: Vec<f64>,
    pub texts: Vec<String>,
    pub ids: Vec<String>,
    pub times: Vec<(i128, String)>,
    pub uuids: Vec<String>,
}

const BASE_TIMES: [&str; 3] = [
    "2026-01-01T00:00:00Z",
    "2026-01-02T00:00:00Z",
    "2026-01-03T00:00:00Z",
];
const BASE_UUIDS: [&str; 2] = [
    "00000000-0000-0000-0000-000000000000",
    "123e4567-e89b-12d3-a456-426614174000",
];
const DAY: i128 = 86_400 * 1_000_000_000;

fn push_unique<T: PartialEq>(v: &mut Vec<T>, x: T) {
    if !v.contains(&x) {
        v.push(x);
    }
}

fn push_float(v: &mut Vec<f64>, x: f64) {
    if !v.iter().any(|y| y.to_bits() == x.to_bits()) {
        v.push(x);
    }
}

fn push_time(v: &mut Vec<(i128, String)>, instant: i128, text: &str) {
    if !v.iter().any(|(i, _)| *i == instant) {
        v.push((instant, text.to_string()));
    }
}

fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}

impl Domain {
    /// What every obligation gets before its own literals.
    pub fn base() -> Domain {
        Domain {
            // The lengths the generator gives lists, so a page and a size
            // range over the item counts and their products straddle them: a
            // computed threshold is crossed without an example.
            ints: vec![0, 1, 2, 3, 4],
            floats: vec![0.0, 1.0],
            texts: vec!["".into(), "a".into(), "b".into()],
            ids: (0..MAX_LEN).map(|i| format!("k{i}")).collect(),
            times: Vec::new(),
            uuids: Vec::new(),
        }
    }

    /// The base plus the scalars of some typed values: for the mutant
    /// search, an input's own scalars and the obligation's named values.
    pub fn seeded(values: &[(&Value, &Type)]) -> Domain {
        let mut d = Domain::base();
        for (v, t) in values {
            d.add_typed(v, t);
        }
        d.finish();
        d
    }

    fn push_text(&mut self, s: &str) {
        if let Some(instant) = parse_rfc3339(s) {
            push_time(&mut self.times, instant, s);
        } else if is_uuid(s) {
            push_unique(&mut self.uuids, s.to_string());
        } else {
            push_unique(&mut self.texts, s.to_string());
            push_unique(&mut self.ids, s.to_string());
        }
    }

    /// Every scalar in a value, filed under the type that declares it.
    pub fn add_typed(&mut self, v: &Value, t: &Type) {
        match (v, t) {
            (Value::None, _) | (Value::Bool(_), _) => {}
            (v, Type::Option(inner)) => self.add_typed(v, inner),
            (Value::Int(i), Type::Float) => push_float(&mut self.floats, *i as f64),
            (Value::Int(i), _) => push_unique(&mut self.ints, *i),
            (Value::Float(f), _) => push_float(&mut self.floats, *f),
            (Value::Text(s), Type::Id) => push_unique(&mut self.ids, s.clone()),
            (Value::Text(s), Type::Uuid) => push_unique(&mut self.uuids, s.clone()),
            (Value::Text(s), Type::Time) => {
                if let Some(i) = parse_rfc3339(s) {
                    push_time(&mut self.times, i, s);
                }
            }
            (Value::Text(s), Type::Text) => push_unique(&mut self.texts, s.clone()),
            (Value::Text(s), _) => self.push_text(s),
            (Value::Time { instant, text }, _) => push_time(&mut self.times, *instant, text),
            (Value::List(xs), Type::List(elem, _)) => {
                for x in xs {
                    self.add_typed(x, elem);
                }
            }
            (Value::List(xs), _) => {
                for x in xs {
                    self.add_typed(x, &Type::Unknown);
                }
            }
            (Value::Record(fields), Type::Record { fields: tf, .. }) => {
                for (n, x) in fields {
                    let t = tf
                        .iter()
                        .find(|(m, _)| m == n)
                        .map(|(_, t)| t)
                        .unwrap_or(&Type::Unknown);
                    self.add_typed(x, t);
                }
            }
            (Value::Record(fields), _) => {
                for (_, x) in fields {
                    self.add_typed(x, &Type::Unknown);
                }
            }
            (Value::Ok(x), Type::Result(ok, _)) => self.add_typed(x, ok),
            (Value::Err(x), Type::Result(_, err)) => self.add_typed(x, err),
            (Value::Ok(x) | Value::Err(x), _) => self.add_typed(x, &Type::Unknown),
        }
    }

    /// Every literal in a property or abbreviation body, untyped. An
    /// integer literal brings its two neighbours, so a threshold the
    /// obligation names is crossed from both sides even when no example
    /// crosses it: a draft has no examples, and `amount <= 100` with inputs
    /// drawn from 0, 1, 2 and 100 never sees 101.
    pub fn add_expr(&mut self, e: &Expr) {
        e.walk(&mut |sub| match sub {
            Expr::Int(i) => {
                let i = i128::from(*i);
                push_unique(&mut self.ints, i);
                push_unique(&mut self.ints, i - 1);
                push_unique(&mut self.ints, i + 1);
            }
            Expr::Float(f) => push_float(&mut self.floats, *f),
            Expr::Text(s) => self.push_text(s),
            _ => {}
        });
    }

    /// Close the domain: Time gets one earlier and one later instant, or
    /// three fixed ones when the obligation names none; UUID gets fixed
    /// values when it names none.
    pub fn finish(&mut self) {
        self.close();
        // Sorted: the domain is a set, and the order inputs are generated in
        // must not follow the order the obligation's literals appear in,
        // or two presentations of one obligation would run different inputs.
        self.ints.sort_unstable();
        self.floats.sort_by(|a, b| a.total_cmp(b));
        self.texts.sort();
        self.ids.sort();
        self.times.sort();
        self.uuids.sort();
    }

    fn close(&mut self) {
        if self.times.is_empty() {
            for t in BASE_TIMES {
                self.times.push((parse_rfc3339(t).unwrap(), t.to_string()));
            }
        } else {
            let lo = self.times.iter().map(|(i, _)| *i).min().unwrap();
            let hi = self.times.iter().map(|(i, _)| *i).max().unwrap();
            let (earlier, later) = (lo - DAY, hi + DAY);
            push_time(&mut self.times, earlier, &format_rfc3339(earlier));
            push_time(&mut self.times, later, &format_rfc3339(later));
        }
        if self.uuids.is_empty() {
            for u in BASE_UUIDS {
                self.uuids.push(u.to_string());
            }
        }
    }

    /// How many distinct values a scalar type has here.
    pub fn scalar_space(&self, t: &Type) -> usize {
        match t {
            Type::Int => self.ints.len(),
            Type::Float => self.floats.len(),
            Type::Text => self.texts.len(),
            Type::Id => self.ids.len(),
            Type::Time => self.times.len(),
            Type::Uuid => self.uuids.len(),
            Type::Bool => 2,
            Type::Option(inner) => 1 + self.scalar_space(inner),
            _ => 1,
        }
    }
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// An instant as an RFC 3339 text in UTC, whole seconds.
pub fn format_rfc3339(nanos: i128) -> String {
    let secs = nanos.div_euclid(1_000_000_000) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// A deterministic stream of 64-bit values (splitmix64 over a counter).
#[derive(Debug, Clone)]
pub struct Stream(u64);

impl Stream {
    pub fn new(seed: u64) -> Stream {
        Stream(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    pub fn draw(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.draw() % n as u64) as usize
        }
    }
}

fn length_range(refs: &[Refinement], max_len: usize) -> (usize, usize) {
    let (mut lo, mut hi) = (0usize, max_len);
    for r in refs {
        if let Refinement::Length { lo: l, hi: h } = r {
            lo = lo.max(*l as usize);
            hi = hi.min(*h as usize);
        }
    }
    (lo, hi.max(lo))
}

fn unique_key(refs: &[Refinement]) -> Option<&str> {
    refs.iter().find_map(|r| match r {
        Refinement::UniqueBy(k) => Some(k.as_str()),
        _ => None,
    })
}

fn field_type<'t>(elem: &'t Type, key: &str) -> Option<&'t Type> {
    elem.record_fields()?
        .iter()
        .find(|(n, _)| n == key)
        .map(|(_, t)| t)
}

/// One value of a type, drawn from the stream. A list's length is
/// `fixed_len` when given, else drawn from its range; `max_len` bounds every
/// list inside.
pub fn sample(
    ty: &Type,
    d: &Domain,
    s: &mut Stream,
    fixed_len: Option<usize>,
    max_len: usize,
) -> Value {
    match ty {
        Type::Int => Value::Int(d.ints[s.below(d.ints.len())]),
        Type::Float => Value::Float(d.floats[s.below(d.floats.len())]),
        Type::Text => Value::Text(d.texts[s.below(d.texts.len())].clone()),
        Type::Id => Value::Text(d.ids[s.below(d.ids.len())].clone()),
        Type::Uuid => Value::Text(d.uuids[s.below(d.uuids.len())].clone()),
        Type::Time => {
            let (instant, text) = &d.times[s.below(d.times.len())];
            Value::Time {
                instant: *instant,
                text: text.clone(),
            }
        }
        Type::Bool => Value::Bool(s.below(2) == 1),
        Type::Option(inner) => {
            if s.below(3) == 0 {
                Value::None
            } else {
                sample(inner, d, s, None, max_len)
            }
        }
        Type::Result(ok, err) => {
            if s.below(2) == 0 {
                Value::Ok(Box::new(sample(ok, d, s, None, max_len)))
            } else {
                Value::Err(Box::new(sample(err, d, s, None, max_len)))
            }
        }
        Type::List(elem, refs) => {
            let (lo, hi) = length_range(refs, max_len);
            let len = fixed_len.unwrap_or_else(|| lo + s.below(hi - lo + 1));
            let key = unique_key(refs);
            let mut items = Vec::with_capacity(len);
            let mut keys: Vec<Value> = Vec::new();
            'items: for _ in 0..len {
                for _try in 0..32 {
                    let v = sample(elem, d, s, None, max_len);
                    if let Some(k) = key {
                        if let Some(kv) = v.field(k) {
                            if keys.contains(kv) {
                                continue;
                            }
                            keys.push(kv.clone());
                        }
                    }
                    items.push(v);
                    continue 'items;
                }
                break;
            }
            Value::List(items)
        }
        Type::Record { fields, .. } => Value::record(
            fields
                .iter()
                .map(|(n, t)| (n.clone(), sample(t, d, s, None, max_len)))
                .collect(),
        ),
        Type::Named(_) | Type::Unknown => Value::None,
    }
}

fn pow_sat(base: u128, exp: usize) -> u128 {
    let mut out: u128 = 1;
    for _ in 0..exp {
        out = out.saturating_mul(base);
    }
    out
}

/// An upper bound on the number of values of a type (uniqueness is not
/// subtracted).
pub fn count(ty: &Type, d: &Domain, fixed_len: Option<usize>, max_len: usize) -> u128 {
    match ty {
        Type::Int | Type::Float | Type::Text | Type::Id | Type::Time | Type::Uuid | Type::Bool => {
            d.scalar_space(ty) as u128
        }
        Type::Option(inner) => 1 + count(inner, d, None, max_len),
        Type::Result(ok, err) => {
            count(ok, d, None, max_len).saturating_add(count(err, d, None, max_len))
        }
        Type::Record { fields, .. } => fields.iter().fold(1u128, |acc, (_, t)| {
            acc.saturating_mul(count(t, d, None, max_len))
        }),
        Type::List(elem, refs) => {
            let e = count(elem, d, None, max_len);
            match fixed_len {
                Some(l) => pow_sat(e, l),
                None => {
                    let (lo, hi) = length_range(refs, max_len);
                    (lo..=hi).fold(0u128, |acc, l| acc.saturating_add(pow_sat(e, l)))
                }
            }
        }
        Type::Named(_) | Type::Unknown => 1,
    }
}

fn cartesian(parts: &[Vec<Value>]) -> Vec<Vec<Value>> {
    let mut out: Vec<Vec<Value>> = vec![Vec::new()];
    for part in parts {
        let mut next = Vec::with_capacity(out.len() * part.len());
        for prefix in &out {
            for v in part {
                let mut row = prefix.clone();
                row.push(v.clone());
                next.push(row);
            }
        }
        out = next;
    }
    out
}

fn tuples(elems: &[Value], len: usize, key: Option<&str>) -> Vec<Value> {
    let parts: Vec<Vec<Value>> = (0..len).map(|_| elems.to_vec()).collect();
    cartesian(&parts)
        .into_iter()
        .filter(|row| {
            let Some(k) = key else { return true };
            let keys: Vec<Option<&Value>> = row.iter().map(|v| v.field(k)).collect();
            keys.iter()
                .enumerate()
                .all(|(i, kv)| !keys[..i].contains(kv))
        })
        .map(Value::List)
        .collect()
}

fn all_values(ty: &Type, d: &Domain, fixed_len: Option<usize>, max_len: usize) -> Vec<Value> {
    match ty {
        Type::Int => d.ints.iter().map(|i| Value::Int(*i)).collect(),
        Type::Float => d.floats.iter().map(|f| Value::Float(*f)).collect(),
        Type::Text => d.texts.iter().map(|s| Value::Text(s.clone())).collect(),
        Type::Id => d.ids.iter().map(|s| Value::Text(s.clone())).collect(),
        Type::Uuid => d.uuids.iter().map(|s| Value::Text(s.clone())).collect(),
        Type::Time => d
            .times
            .iter()
            .map(|(instant, text)| Value::Time {
                instant: *instant,
                text: text.clone(),
            })
            .collect(),
        Type::Bool => vec![Value::Bool(false), Value::Bool(true)],
        Type::Option(inner) => {
            let mut out = vec![Value::None];
            out.extend(all_values(inner, d, None, max_len));
            out
        }
        Type::Result(ok, err) => {
            let mut out: Vec<Value> = all_values(ok, d, None, max_len)
                .into_iter()
                .map(|v| Value::Ok(Box::new(v)))
                .collect();
            out.extend(
                all_values(err, d, None, max_len)
                    .into_iter()
                    .map(|v| Value::Err(Box::new(v))),
            );
            out
        }
        Type::Record { fields, .. } => {
            let parts: Vec<Vec<Value>> = fields
                .iter()
                .map(|(_, t)| all_values(t, d, None, max_len))
                .collect();
            cartesian(&parts)
                .into_iter()
                .map(|row| Value::record(fields.iter().map(|(n, _)| n.clone()).zip(row).collect()))
                .collect()
        }
        Type::List(elem, refs) => {
            let (lo, hi) = length_range(refs, max_len);
            let lens: Vec<usize> = match fixed_len {
                Some(l) => vec![l],
                None => (lo..=hi).collect(),
            };
            let elems = all_values(elem, d, None, max_len);
            let key = unique_key(refs);
            lens.into_iter()
                .flat_map(|l| tuples(&elems, l, key))
                .collect()
        }
        Type::Named(_) | Type::Unknown => vec![Value::None],
    }
}

/// Every value of a type, when there are at most `limit` of them.
pub fn all(
    ty: &Type,
    d: &Domain,
    fixed_len: Option<usize>,
    max_len: usize,
    limit: usize,
) -> Option<Vec<Value>> {
    if count(ty, d, fixed_len, max_len) > limit as u128 {
        return None;
    }
    Some(all_values(ty, d, fixed_len, max_len))
}

/// The number of scalar leaves in a value: the size the search orders by.
pub fn size(v: &Value) -> usize {
    match v {
        Value::List(xs) => 1 + xs.iter().map(size).sum::<usize>(),
        Value::Record(fields) => fields.iter().map(|(_, x)| size(x)).sum(),
        Value::Ok(x) | Value::Err(x) => size(x),
        _ => 1,
    }
}

/// The JSON text of an input tuple: the identity dedupe works on.
pub fn key_of(inputs: &[Value]) -> String {
    serde_json::Value::Array(inputs.iter().map(to_json).collect()).to_string()
}

/// One length bucket of the top-level generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bucket {
    /// The length of each top-level list input, in input order.
    pub lengths: Vec<usize>,
    pub taken: usize,
    /// How many input tuples exist with those lengths (an upper bound).
    pub space: u128,
    /// Whether every one of them was taken.
    pub exhaustive: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Generated {
    pub inputs: Vec<Vec<Value>>,
    pub buckets: Vec<Bucket>,
}

impl Generated {
    pub fn cut(&self) -> u128 {
        self.buckets
            .iter()
            .map(|b| b.space.saturating_sub(b.taken as u128))
            .fold(0u128, |a, b| a.saturating_add(b))
    }
}

/// Generated input tuples for a signature: `budget` of them at most, split
/// evenly over the lengths of the top-level list inputs so every length is
/// represented, exhaustive where a bucket is small enough, sampled
/// otherwise. Tuples in `exclude` (the examples) are not repeated.
pub fn generate(
    inputs: &[(String, Type)],
    d: &Domain,
    budget: usize,
    exclude: &[Vec<Value>],
) -> Generated {
    let mut out = Generated::default();
    if budget == 0 || inputs.is_empty() {
        return out;
    }
    let mut ranges: Vec<(usize, Vec<usize>)> = Vec::new();
    for (i, (_, t)) in inputs.iter().enumerate() {
        if let Type::List(elem, refs) = t {
            let (lo, mut hi) = length_range(refs, MAX_LEN);
            if let Some(k) = unique_key(refs) {
                if let Some(kt) = field_type(elem, k) {
                    hi = hi.min(d.scalar_space(kt));
                }
            }
            let lens: Vec<usize> = if hi < lo {
                vec![lo]
            } else {
                (lo..=hi).collect()
            };
            ranges.push((i, lens));
        }
    }
    let length_sets: Vec<Vec<usize>> = {
        let parts: Vec<Vec<Value>> = ranges
            .iter()
            .map(|(_, lens)| lens.iter().map(|l| Value::Int(*l as i128)).collect())
            .collect();
        cartesian(&parts)
            .into_iter()
            .map(|row| {
                row.iter()
                    .map(|v| match v {
                        Value::Int(i) => *i as usize,
                        _ => 0,
                    })
                    .collect()
            })
            .collect()
    };
    let n = length_sets.len().max(1);
    let mut seen: HashSet<String> = exclude.iter().map(|e| key_of(e)).collect();
    let mut carry = 0usize;
    for (b, lengths) in length_sets.iter().enumerate() {
        let quota = budget / n + usize::from(b < budget % n) + carry;
        let mut fixed: Vec<Option<usize>> = vec![None; inputs.len()];
        for ((i, _), l) in ranges.iter().zip(lengths) {
            fixed[*i] = Some(*l);
        }
        let space = inputs.iter().enumerate().fold(1u128, |acc, (i, (_, t))| {
            acc.saturating_mul(count(t, d, fixed[i], MAX_LEN))
        });
        let mut taken = 0usize;
        let exhaustive = space <= quota as u128;
        if exhaustive {
            let parts: Vec<Vec<Value>> = inputs
                .iter()
                .enumerate()
                .map(|(i, (_, t))| all_values(t, d, fixed[i], MAX_LEN))
                .collect();
            for combo in cartesian(&parts) {
                if seen.insert(key_of(&combo)) {
                    out.inputs.push(combo);
                    taken += 1;
                }
            }
        } else {
            let mut s = Stream::new(b as u64 + 1);
            let mut attempts = 0usize;
            while taken < quota && attempts < quota * 20 {
                attempts += 1;
                let combo: Vec<Value> = inputs
                    .iter()
                    .enumerate()
                    .map(|(i, (_, t))| sample(t, d, &mut s, fixed[i], MAX_LEN))
                    .collect();
                let lengths_ok = combo.iter().zip(&fixed).all(|(v, f)| match (v, f) {
                    (Value::List(xs), Some(l)) => xs.len() == *l,
                    _ => true,
                });
                if lengths_ok && seen.insert(key_of(&combo)) {
                    out.inputs.push(combo);
                    taken += 1;
                }
            }
        }
        carry = quota.saturating_sub(taken);
        out.buckets.push(Bucket {
            lengths: lengths.clone(),
            taken,
            space,
            exhaustive,
        });
    }
    out
}

/// Output candidates for the mutant search: the `seed` values first (what
/// the caller drew from the input), then every value with lists of at most
/// one element, smallest first, then sampled values with lists up to
/// `MAX_LEN`, up to `cap` in all.
pub fn candidates(ty: &Type, d: &Domain, cap: usize, seed: Vec<Value>) -> Vec<Value> {
    struct Pool {
        out: Vec<Value>,
        seen: HashSet<String>,
        cap: usize,
    }
    impl Pool {
        fn push(&mut self, v: Value) {
            if self.out.len() < self.cap && self.seen.insert(to_json(&v).to_string()) {
                self.out.push(v);
            }
        }
    }
    let mut pool = Pool {
        out: Vec::new(),
        seen: HashSet::new(),
        cap,
    };
    for v in seed {
        pool.push(v);
    }
    match all(ty, d, None, 1, cap) {
        Some(mut small) => {
            small.sort_by_key(size);
            for v in small {
                pool.push(v);
            }
        }
        None => {
            let mut s = Stream::new(7);
            let mut attempts = 0;
            while pool.out.len() < cap / 2 && attempts < cap * 10 {
                attempts += 1;
                pool.push(sample(ty, d, &mut s, None, 1));
            }
        }
    }
    let mut s = Stream::new(11);
    let mut attempts = 0;
    while pool.out.len() < cap && attempts < cap * 20 {
        attempts += 1;
        pool.push(sample(ty, d, &mut s, None, MAX_LEN));
    }
    pool.out
}
