//! Values at the boundary: the checker's own value model, the JSON
//! encoding the Python runner speaks, and the boundary validator that
//! decodes JSON against a declared type, refinements included.

use std::cmp::Ordering;
use std::fmt;

use serde_json::{Map, Value as Json};
use surety_syntax::{Expr, Refinement, Type};

#[derive(Debug, Clone)]
pub enum Value {
    /// The boundary caps an `Int` at `i64`; the checker's arithmetic runs
    /// in `i128`, so a product of two boundary values and a sum over any
    /// list the generator or an example produces fits, and the obligation's
    /// own arithmetic never rounds or wraps.
    Int(i128),
    Float(f64),
    Text(String),
    /// Nanoseconds since the epoch, plus the text as written.
    Time {
        instant: i128,
        text: String,
    },
    Bool(bool),
    None,
    List(Vec<Value>),
    /// Fields sorted by name.
    Record(Vec<(String, Value)>),
    Ok(Box<Value>),
    Err(Box<Value>),
}

impl Value {
    pub fn record(mut fields: Vec<(String, Value)>) -> Value {
        fields.sort_by(|a, b| a.0.cmp(&b.0));
        Value::Record(fields)
    }

    pub fn field(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Record(fields) => fields.iter().find(|(n, _)| n == name).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Ordering for `<`, `<=`, `>`, `>=` and `sorted_by`: numbers
    /// numerically, Text by code point, Time as instants.
    pub fn order(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
            (a, b) if a.as_f64().is_some() && b.as_f64().is_some() => {
                a.as_f64()?.partial_cmp(&b.as_f64()?)
            }
            (Value::Text(a), Value::Text(b)) => Some(a.cmp(b)),
            (Value::Time { instant: a, .. }, Value::Time { instant: b, .. }) => Some(a.cmp(b)),
            (Value::Time { instant: a, .. }, Value::Text(b)) => Some(a.cmp(&parse_rfc3339(b)?)),
            (Value::Text(a), Value::Time { instant: b, .. }) => Some(parse_rfc3339(a)?.cmp(b)),
            _ => Option::None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Text(_) => "Text",
            Value::Time { .. } => "Time",
            Value::Bool(_) => "Bool",
            Value::None => "None",
            Value::List(_) => "list",
            Value::Record(_) => "record",
            Value::Ok(_) => "Ok",
            Value::Err(_) => "Err",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (a, b) if a.as_f64().is_some() && b.as_f64().is_some() => a.as_f64() == b.as_f64(),
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Time { instant: a, .. }, Value::Time { instant: b, .. }) => a == b,
            (Value::Time { instant: a, .. }, Value::Text(b))
            | (Value::Text(b), Value::Time { instant: a, .. }) => parse_rfc3339(b) == Some(*a),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::List(a), Value::List(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y)
            }
            (Value::Record(a), Value::Record(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|((n, x), (m, y))| n == m && x == y)
            }
            (Value::Ok(a), Value::Ok(b)) | (Value::Err(a), Value::Err(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(x) => write!(f, "{x:?}"),
            Value::Text(s) => write!(f, "{s:?}"),
            Value::Time { text, .. } => write!(f, "{text:?}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::None => write!(f, "None"),
            Value::List(xs) => {
                write!(f, "[")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{x}")?;
                }
                write!(f, "]")
            }
            Value::Record(fields) => {
                write!(f, "{{")?;
                for (i, (n, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{n}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::Ok(v) => write!(f, "Ok({v})"),
            Value::Err(v) => write!(f, "Err({v})"),
        }
    }
}

/// Days since 1970-01-01 for a proleptic Gregorian date.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
    }
}

/// Parse an RFC 3339 timestamp to nanoseconds since the epoch.
pub fn parse_rfc3339(s: &str) -> Option<i128> {
    let b = s.as_bytes();
    if b.len() < 20 {
        return Option::None;
    }
    let num = |from: usize, to: usize| -> Option<i64> {
        if b[from..to].iter().all(|c| c.is_ascii_digit()) {
            s[from..to].parse().ok()
        } else {
            Option::None
        }
    };
    if b[4] != b'-'
        || b[7] != b'-'
        || !(b[10] == b'T' || b[10] == b't')
        || b[13] != b':'
        || b[16] != b':'
    {
        return Option::None;
    }
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, sec) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&mo) || d < 1 || d > days_in_month(y, mo) || h > 23 || mi > 59 || sec > 60
    {
        return Option::None;
    }
    let mut i = 19;
    let mut nanos: i128 = 0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return Option::None;
        }
        let mut digits = s[start..i].to_string();
        digits.truncate(9);
        while digits.len() < 9 {
            digits.push('0');
        }
        nanos = digits.parse().ok()?;
    }
    if i >= b.len() {
        return Option::None;
    }
    let offset: i64 = match b[i] {
        b'Z' | b'z' => {
            i += 1;
            0
        }
        b'+' | b'-' => {
            let sign = if b[i] == b'+' { 1 } else { -1 };
            if i + 6 > b.len() || b[i + 3] != b':' {
                return Option::None;
            }
            let oh = num(i + 1, i + 3)?;
            let om = num(i + 4, i + 6)?;
            if oh > 23 || om > 59 {
                return Option::None;
            }
            i += 6;
            sign * (oh * 3600 + om * 60)
        }
        _ => return Option::None,
    };
    if i != b.len() {
        return Option::None;
    }
    let secs = days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + sec - offset;
    Some(secs as i128 * 1_000_000_000 + nanos)
}

/// A value that does not fit its declared type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryError {
    pub path: String,
    pub message: String,
    /// The key of the `unique` refinement that failed, when one did.
    pub unique_key: Option<String>,
}

impl fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "at {}: {}", self.path, self.message)
        }
    }
}

fn bad(path: &str, message: impl Into<String>) -> BoundaryError {
    BoundaryError {
        path: path.to_string(),
        message: message.into(),
        unique_key: None,
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

pub fn check_refinements(
    items: &[Value],
    refs: &[Refinement],
    path: &str,
) -> Result<(), BoundaryError> {
    for r in refs {
        match r {
            Refinement::Length { lo, hi } => {
                let n = items.len() as u64;
                if n < *lo || n > *hi {
                    return Err(bad(
                        path,
                        format!("list has {n} element(s), the type allows {lo}..{hi}"),
                    ));
                }
            }
            Refinement::UniqueBy(key) => {
                let mut seen: Vec<&Value> = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    let Some(k) = item.field(key) else {
                        return Err(bad(
                            &format!("{path}[{i}]"),
                            format!("no field `{key}` to be unique by"),
                        ));
                    };
                    if seen.contains(&k) {
                        let mut e = bad(
                            path,
                            format!(
                                "`{key}` {k} appears more than once; the type says `unique {key}`"
                            ),
                        );
                        e.unique_key = Some(key.clone());
                        return Err(e);
                    }
                    seen.push(k);
                }
            }
        }
    }
    Ok(())
}

/// Decode JSON against a declared (expanded) type: the boundary validator.
pub fn from_json(json: &Json, ty: &Type) -> Result<Value, BoundaryError> {
    decode(json, ty, "out")
}

pub fn from_json_at(json: &Json, ty: &Type, path: &str) -> Result<Value, BoundaryError> {
    decode(json, ty, path)
}

fn decode(json: &Json, ty: &Type, path: &str) -> Result<Value, BoundaryError> {
    match ty {
        Type::Int => match json {
            Json::Number(n) if n.is_i64() => Ok(Value::Int(i128::from(n.as_i64().unwrap()))),
            Json::Number(n)
                if n.as_f64()
                    .is_some_and(|f| f.fract() == 0.0 && f.abs() < 9.0e15) =>
            {
                Ok(Value::Int(n.as_f64().unwrap() as i128))
            }
            other => Err(bad(
                path,
                format!("expected an Int, found {}", describe(other)),
            )),
        },
        Type::Float => match json {
            Json::Number(n) => Ok(Value::Float(n.as_f64().unwrap())),
            other => Err(bad(
                path,
                format!("expected a Float, found {}", describe(other)),
            )),
        },
        Type::Text | Type::Id => match json {
            Json::String(s) => Ok(Value::Text(s.clone())),
            other => Err(bad(
                path,
                format!("expected {ty}, found {}", describe(other)),
            )),
        },
        Type::Uuid => match json {
            Json::String(s) if is_uuid(s) => Ok(Value::Text(s.clone())),
            Json::String(s) => Err(bad(path, format!("{s:?} is not a UUID"))),
            other => Err(bad(
                path,
                format!("expected a UUID string, found {}", describe(other)),
            )),
        },
        Type::Time => match json {
            Json::String(s) => match parse_rfc3339(s) {
                Some(instant) => Ok(Value::Time {
                    instant,
                    text: s.clone(),
                }),
                Option::None => Err(bad(path, format!("{s:?} is not an RFC 3339 time"))),
            },
            Json::Object(map) if map.len() == 1 && map.contains_key("$time") => {
                decode(&map["$time"], ty, path)
            }
            other => Err(bad(
                path,
                format!("expected a Time string, found {}", describe(other)),
            )),
        },
        Type::Bool => match json {
            Json::Bool(b) => Ok(Value::Bool(*b)),
            other => Err(bad(
                path,
                format!("expected a Bool, found {}", describe(other)),
            )),
        },
        Type::Option(inner) => match json {
            Json::Null => Ok(Value::None),
            other => decode(other, inner, path),
        },
        Type::Result(ok, err) => match json {
            Json::Object(map) if map.len() == 1 && map.contains_key("ok") => Ok(Value::Ok(
                Box::new(decode(&map["ok"], ok, &format!("{path}.Ok"))?),
            )),
            Json::Object(map) if map.len() == 1 && map.contains_key("err") => Ok(Value::Err(
                Box::new(decode(&map["err"], err, &format!("{path}.Err"))?),
            )),
            other => Err(bad(
                path,
                format!(
                    "expected a Result ({{\"ok\": v}} or {{\"err\": e}}), found {}",
                    describe(other)
                ),
            )),
        },
        Type::List(elem, refs) => match json {
            Json::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for (i, item) in items.iter().enumerate() {
                    out.push(decode(item, elem, &format!("{path}[{i}]"))?);
                }
                check_refinements(&out, refs, path)?;
                Ok(Value::List(out))
            }
            other => Err(bad(
                path,
                format!("expected a list, found {}", describe(other)),
            )),
        },
        Type::Record { fields, .. } => match json {
            Json::Object(map) => {
                let mut out = Vec::with_capacity(fields.len());
                for (name, fty) in fields {
                    let Some(v) = map.get(name) else {
                        return Err(bad(path, format!("missing field `{name}`")));
                    };
                    out.push((name.clone(), decode(v, fty, &format!("{path}.{name}"))?));
                }
                for key in map.keys() {
                    if !fields.iter().any(|(n, _)| n == key) {
                        return Err(bad(
                            path,
                            format!("unexpected field `{key}` (records are strict)"),
                        ));
                    }
                }
                Ok(Value::record(out))
            }
            other => Err(bad(
                path,
                format!("expected a record, found {}", describe(other)),
            )),
        },
        Type::Named(n) => Err(bad(path, format!("unresolved type `{n}`"))),
        Type::Unknown => Ok(from_json_untyped(json)),
    }
}

fn describe(json: &Json) -> String {
    match json {
        Json::Null => "null".into(),
        Json::Bool(b) => format!("{b}"),
        Json::Number(n) => format!("{n}"),
        Json::String(s) => format!("{s:?}"),
        Json::Array(_) => "a list".into(),
        Json::Object(_) => "an object".into(),
    }
}

/// The first `Int` in the value past what the boundary carries (`i64`),
/// which only the obligation's own arithmetic can produce.
pub fn exceeds_boundary(v: &Value) -> Option<i128> {
    match v {
        Value::Int(i) => i64::try_from(*i).is_err().then_some(*i),
        Value::List(xs) => xs.iter().find_map(exceeds_boundary),
        Value::Record(fields) => fields.iter().find_map(|(_, v)| exceeds_boundary(v)),
        Value::Ok(x) | Value::Err(x) => exceeds_boundary(x),
        _ => None,
    }
}

/// Decode JSON with no type to guide it: a graph reference whose
/// obligation is not in the graph.
pub fn from_json_untyped(json: &Json) -> Value {
    match json {
        Json::Null => Value::None,
        Json::Bool(b) => Value::Bool(*b),
        Json::Number(n) => n
            .as_i64()
            .map(|i| Value::Int(i128::from(i)))
            .unwrap_or_else(|| Value::Float(n.as_f64().unwrap_or(f64::NAN))),
        Json::String(s) => Value::Text(s.clone()),
        Json::Array(xs) => Value::List(xs.iter().map(from_json_untyped).collect()),
        Json::Object(map) if map.len() == 1 && map.contains_key("$time") => match &map["$time"] {
            Json::String(s) => match parse_rfc3339(s) {
                Some(instant) => Value::Time {
                    instant,
                    text: s.clone(),
                },
                Option::None => Value::Text(s.clone()),
            },
            other => from_json_untyped(other),
        },
        Json::Object(map) if map.len() == 1 && map.contains_key("ok") => {
            Value::Ok(Box::new(from_json_untyped(&map["ok"])))
        }
        Json::Object(map) if map.len() == 1 && map.contains_key("err") => {
            Value::Err(Box::new(from_json_untyped(&map["err"])))
        }
        Json::Object(map) => Value::record(
            map.iter()
                .map(|(k, v)| (k.clone(), from_json_untyped(v)))
                .collect(),
        ),
    }
}

/// Encode a value as plain JSON, a Time as its text as written: reports,
/// keys and the record. The Python boundary uses `to_boundary_json`.
pub fn to_json(v: &Value) -> Json {
    match v {
        // An Int past i64 arises only from the obligation's arithmetic and
        // never crosses the boundary (`exceeds_boundary` stops it); in a
        // report or a key it is written as the nearest float.
        Value::Int(i) => i64::try_from(*i)
            .map(Json::from)
            .unwrap_or_else(|_| Json::from(*i as f64)),
        Value::Float(f) => Json::from(*f),
        Value::Text(s) => Json::String(s.clone()),
        Value::Time { text, .. } => Json::String(text.clone()),
        Value::Bool(b) => Json::Bool(*b),
        Value::None => Json::Null,
        Value::List(xs) => Json::Array(xs.iter().map(to_json).collect()),
        Value::Record(fields) => {
            let mut map = Map::new();
            for (n, v) in fields {
                map.insert(n.clone(), to_json(v));
            }
            Json::Object(map)
        }
        Value::Ok(x) => {
            let mut map = Map::new();
            map.insert("ok".into(), to_json(x));
            Json::Object(map)
        }
        Value::Err(x) => {
            let mut map = Map::new();
            map.insert("err".into(), to_json(x));
            Json::Object(map)
        }
    }
}

/// Encode a value for the Python boundary: `to_json`, except that a Time
/// crosses tagged, `{"$time": "<RFC 3339>"}`, in its instant's canonical UTC
/// form, so the driver can deliver it as an aware UTC datetime.
pub fn to_boundary_json(v: &Value) -> Json {
    match v {
        Value::Time { instant, .. } => {
            let mut map = Map::new();
            map.insert("$time".into(), Json::String(boundary_text(*instant)));
            Json::Object(map)
        }
        Value::List(xs) => Json::Array(xs.iter().map(to_boundary_json).collect()),
        Value::Record(fields) => {
            let mut map = Map::new();
            for (n, v) in fields {
                map.insert(n.clone(), to_boundary_json(v));
            }
            Json::Object(map)
        }
        Value::Ok(x) => {
            let mut map = Map::new();
            map.insert("ok".into(), to_boundary_json(x));
            Json::Object(map)
        }
        Value::Err(x) => {
            let mut map = Map::new();
            map.insert("err".into(), to_boundary_json(x));
            Json::Object(map)
        }
        other => to_json(other),
    }
}

/// An instant as RFC 3339 UTC text, with its fraction to the microsecond
/// (what a Python datetime holds) when there is one.
fn boundary_text(instant: i128) -> String {
    let mut text = crate::generate::format_rfc3339(instant);
    let micros = instant.rem_euclid(1_000_000_000) / 1_000;
    if micros != 0 {
        let frac = format!("{micros:06}");
        let frac = frac.trim_end_matches('0');
        text.insert_str(text.len() - 1, &format!(".{frac}"));
    }
    text
}

/// How `...` in an output record literal is filled: given the stated
/// fields and the fields still needed, return the needed fields' values.
pub type RestFiller<'a> =
    &'a dyn Fn(&[(String, Value)], &[(String, Type)]) -> Result<Vec<(String, Value)>, String>;

/// Build a value from a literal expression against a declared type.
pub fn from_literal(
    e: &Expr,
    ty: &Type,
    lookup: &dyn Fn(&str) -> Option<Value>,
    rest: Option<RestFiller<'_>>,
) -> Result<Value, String> {
    let misfit = |what: &str| Err(format!("{what} does not fit the type {ty}"));
    match (e, ty) {
        (Expr::Name(n, _), _) => {
            lookup(n).ok_or_else(|| format!("named value `{n}` is not defined"))
        }
        (Expr::None, Type::Option(_)) => Ok(Value::None),
        (_, Type::Option(inner)) => from_literal(e, inner, lookup, rest),
        (Expr::Int(i), Type::Int) => Ok(Value::Int(i128::from(*i))),
        (Expr::Int(i), Type::Float) => Ok(Value::Float(*i as f64)),
        (Expr::Float(f), Type::Float) => Ok(Value::Float(*f)),
        (Expr::Text(s), Type::Text | Type::Id) => Ok(Value::Text(s.clone())),
        (Expr::Text(s), Type::Uuid) => {
            if is_uuid(s) {
                Ok(Value::Text(s.clone()))
            } else {
                Err(format!("{s:?} is not a UUID"))
            }
        }
        (Expr::Text(s), Type::Time) => parse_rfc3339(s)
            .map(|instant| Value::Time {
                instant,
                text: s.clone(),
            })
            .ok_or_else(|| format!("{s:?} is not an RFC 3339 time")),
        (Expr::Ok(x), Type::Result(t, _)) => {
            Ok(Value::Ok(Box::new(from_literal(x, t, lookup, rest)?)))
        }
        (Expr::Err(x), Type::Result(_, t)) => {
            Ok(Value::Err(Box::new(from_literal(x, t, lookup, rest)?)))
        }
        (Expr::List(xs), Type::List(t, refs)) => {
            let mut out = Vec::new();
            for x in xs {
                out.push(from_literal(x, t, lookup, rest)?);
            }
            check_refinements(&out, refs, "").map_err(|e| e.to_string())?;
            Ok(Value::List(out))
        }
        (Expr::Record(fields, has_rest), Type::Record { fields: tf, .. }) => {
            let mut stated = Vec::new();
            for (n, x) in fields {
                let Some((_, t)) = tf.iter().find(|(m, _)| m == n) else {
                    return Err(format!("no field `{n}` on {ty}"));
                };
                stated.push((n.clone(), from_literal(x, t, lookup, rest)?));
            }
            let missing: Vec<(String, Type)> = tf
                .iter()
                .filter(|(n, _)| !fields.iter().any(|(m, _)| m == n))
                .cloned()
                .collect();
            if *has_rest {
                let Some(fill) = rest else {
                    return Err("`...` is only allowed in an example's `out`".into());
                };
                stated.extend(fill(&stated, &missing)?);
            } else if let Some((n, _)) = missing.first() {
                return Err(format!("record literal is missing field `{n}` of {ty}"));
            }
            Ok(Value::record(stated))
        }
        (_, Type::Unknown) => Err("cannot build a value for an unknown type".into()),
        (Expr::Tuple(_), _) => misfit("a tuple"),
        (Expr::Int(_), _) => misfit("an Int literal"),
        (Expr::Float(_), _) => misfit("a Float literal"),
        (Expr::Text(_), _) => misfit("a Text literal"),
        (Expr::None, _) => misfit("`None`"),
        (Expr::Ok(_), _) => misfit("`Ok(...)`"),
        (Expr::Err(_), _) => misfit("`Err(...)`"),
        (Expr::List(_), _) => misfit("a list literal"),
        (Expr::Record(..), _) => misfit("a record literal"),
        _ => Err("not a literal".into()),
    }
}
