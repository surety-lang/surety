//! The abstract syntax of an obligation file.
//!
//! Everything here is what the corpus uses. Layout is not recorded: an
//! inline quantifier body and a vertical one produce the same tree.

use std::fmt;

/// A parsed `.sur` file.
#[derive(Debug, Clone, Default)]
pub struct File {
    /// Column-0 lines before the first declaration, kept for the record.
    pub header: Vec<(usize, String)>,
    pub decls: Vec<Decl>,
    /// Sentences, comments and notes, in source order. Pinned, never hashed.
    pub pins: Vec<Pin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub line: usize,
    pub kind: PinKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinKind {
    Sentence,
    Comment,
    Note,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Type(TypeDecl),
    NamedValue(NamedValue),
    Abbrev(Abbrev),
    Function(Function),
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub ty: Type,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct NamedValue {
    pub name: String,
    /// Absent when the literal is a list of already-typed named values.
    pub ty: Option<Type>,
    pub value: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Abbrev {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub body: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<(String, Type)>,
    pub output: Type,
    pub line: usize,
    pub note: Option<String>,
    pub abbrevs: Vec<Abbrev>,
    pub properties: Vec<Property>,
    /// `accept <boolean over the inputs>` lines: the inputs on which the
    /// obligation deliberately admits more than one output. Hashed with the
    /// properties; never a claim about `out`.
    pub accepts: Vec<Property>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub expr: Expr,
    pub line: usize,
    /// The `#` sentence directly above, if any.
    pub sentence: Option<String>,
    /// The source text of the property, vertical body included, one line.
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Example {
    pub line: usize,
    pub note: Option<String>,
    pub input: Expr,
    pub output: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Text,
    Bool,
    Id,
    Time,
    Uuid,
    Named(String),
    Record {
        spreads: Vec<String>,
        fields: Vec<(String, Type)>,
    },
    List(Box<Type>, Vec<Refinement>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    /// Only produced by inference, never by the parser.
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Refinement {
    /// Inclusive length bound.
    Length {
        lo: u64,
        hi: u64,
    },
    UniqueBy(String),
}

impl Type {
    pub fn list(elem: Type) -> Type {
        Type::List(Box::new(elem), Vec::new())
    }
    pub fn record_fields(&self) -> Option<&[(String, Type)]> {
        match self {
            Type::Record { fields, .. } => Some(fields),
            _ => None,
        }
    }
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Text => write!(f, "Text"),
            Type::Bool => write!(f, "Bool"),
            Type::Id => write!(f, "Id"),
            Type::Time => write!(f, "Time"),
            Type::Uuid => write!(f, "UUID"),
            Type::Named(n) => write!(f, "{n}"),
            Type::Record { spreads, fields } => {
                write!(f, "{{ ")?;
                let mut first = true;
                for s in spreads {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "...{s}")?;
                }
                for (n, t) in fields {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{n}: {t}")?;
                }
                write!(f, " }}")
            }
            Type::List(t, refs) => {
                write!(f, "{t}[")?;
                let mut first = true;
                for r in refs {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    match r {
                        Refinement::Length { lo, hi } => write!(f, "{lo}..{hi}")?,
                        Refinement::UniqueBy(k) => write!(f, "unique {k}")?,
                    }
                }
                write!(f, "]")
            }
            Type::Option(t) => write!(f, "Option<{t}>"),
            Type::Result(t, e) => write!(f, "Result<{t}, {e}>"),
            Type::Unknown => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Iff,
    Implies,
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
}

/// Integer arithmetic: `*` binds tighter than `+` and `-`, all three
/// left-associative and above every comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
}

impl ArithOp {
    pub fn symbol(self) -> &'static str {
        match self {
            ArithOp::Add => "+",
            ArithOp::Sub => "-",
            ArithOp::Mul => "*",
        }
    }
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Iff => "<->",
            BinOp::Implies => "->",
            BinOp::Or => "or",
            BinOp::And => "and",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::In => "in",
            BinOp::NotIn => "not in",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOp {
    SameOrderAs,
    SameElementsAs,
    DrawnFrom,
}

impl RelOp {
    pub fn symbol(self) -> &'static str {
        match self {
            RelOp::SameOrderAs => "same_order_as",
            RelOp::SameElementsAs => "same_elements_as",
            RelOp::DrawnFrom => "drawn_from",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QKind {
    All,
    Any,
    None,
    ExactlyOne,
}

impl QKind {
    pub fn word(self) -> &'static str {
        match self {
            QKind::All => "all",
            QKind::Any => "any",
            QKind::None => "none",
            QKind::ExactlyOne => "exactly_one",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Ok,
    Err,
}

/// The binder slot: a name, `_`, or `Ok(x)` / `Err(x)` / `Ok(_)` / `Err(_)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binder {
    pub side: Option<Side>,
    /// `None` is the wildcard `_`.
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Quantifier {
    pub kind: QKind,
    pub binder: Binder,
    pub leftover: bool,
    pub domain: Expr,
    pub where_: Option<Expr>,
    pub body: Option<Expr>,
    pub else_: Option<Expr>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Text(String),
    None,
    /// `true` or `false`: a `Bool` value, in an example or a line.
    Bool(bool),
    Name(String, usize),
    Field(Box<Expr>, String, usize),
    Call(String, Vec<Expr>, usize),
    Ok(Box<Expr>),
    Err(Box<Expr>),
    List(Vec<Expr>),
    /// Fields and whether `...` appears.
    Record(Vec<(String, Expr)>, bool),
    Tuple(Vec<Expr>),
    Not(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// `a + b`, `a - b`, `a * b` over `Int`.
    Arith(ArithOp, Box<Expr>, Box<Expr>),
    /// `list[from .. to]`: the elements from position `from` up to but not
    /// including `to`, positions from 0, both clamped to the list, so a
    /// negative `from` starts at 0 and never counts from the end.
    Slice {
        list: Box<Expr>,
        from: Box<Expr>,
        to: Box<Expr>,
    },
    Before {
        a: Box<Expr>,
        b: Box<Expr>,
        list: Box<Expr>,
    },
    Relation(RelOp, Box<Expr>, Box<Expr>),
    SortedBy(Box<Expr>, Vec<(String, Dir)>),
    Unordered(Box<Expr>),
    Unique(Box<Expr>),
    Quantifier(Box<Quantifier>),
}

impl Expr {
    /// Visit every sub-expression, including quantifier parts.
    pub fn walk<'a>(&'a self, f: &mut dyn FnMut(&'a Expr)) {
        f(self);
        match self {
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Text(_)
            | Expr::None
            | Expr::Bool(_)
            | Expr::Name(..) => {}
            Expr::Field(e, _, _) | Expr::Ok(e) | Expr::Err(e) | Expr::Not(e) => e.walk(f),
            Expr::Unordered(e) | Expr::Unique(e) | Expr::SortedBy(e, _) => e.walk(f),
            Expr::Call(_, args, _) | Expr::List(args) | Expr::Tuple(args) => {
                for a in args {
                    a.walk(f);
                }
            }
            Expr::Record(fields, _) => {
                for (_, e) in fields {
                    e.walk(f);
                }
            }
            Expr::Binary(_, a, b) | Expr::Relation(_, a, b) | Expr::Arith(_, a, b) => {
                a.walk(f);
                b.walk(f);
            }
            Expr::Before { a, b, list } => {
                a.walk(f);
                b.walk(f);
                list.walk(f);
            }
            Expr::Slice { list, from, to } => {
                list.walk(f);
                from.walk(f);
                to.walk(f);
            }
            Expr::Quantifier(q) => {
                q.domain.walk(f);
                if let Some(w) = &q.where_ {
                    w.walk(f);
                }
                if let Some(b) = &q.body {
                    b.walk(f);
                }
                if let Some(e) = &q.else_ {
                    e.walk(f);
                }
            }
        }
    }

    /// Whether the name is mentioned anywhere (as a bare name or a field root).
    pub fn mentions(&self, name: &str) -> bool {
        let mut found = false;
        self.walk(&mut |e| {
            if let Expr::Name(n, _) = e {
                if n == name {
                    found = true;
                }
            }
        });
        found
    }
}
