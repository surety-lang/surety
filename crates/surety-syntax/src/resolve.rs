//! The graph of loaded files: name resolution across files, type expansion,
//! type inference over expressions, and the static checks that need types.

use crate::ast::*;
use crate::error::{Error, ErrorKind, Fix};
use crate::parser::BUILTINS;

#[derive(Debug, Clone)]
pub struct GraphFile {
    /// The fixture name (file stem), used to find its performance.
    pub name: String,
    pub file: File,
}

/// Every `.sur` in a corpus directory, loaded as one graph.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub files: Vec<GraphFile>,
}

impl Graph {
    pub fn new(files: Vec<(String, File)>) -> Graph {
        Graph {
            files: files
                .into_iter()
                .map(|(name, file)| GraphFile { name, file })
                .collect(),
        }
    }

    pub fn single(name: &str, file: File) -> Graph {
        Graph::new(vec![(name.to_string(), file)])
    }

    pub fn file(&self, idx: usize) -> &File {
        &self.files[idx].file
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.files.iter().position(|f| f.name == name)
    }

    pub fn functions_in(&self, idx: usize) -> Vec<&Function> {
        self.file(idx)
            .decls
            .iter()
            .filter_map(|d| match d {
                Decl::Function(f) => Some(f),
                _ => None,
            })
            .collect()
    }

    /// The obligation defining a function, anywhere in the graph.
    pub fn find_function(&self, name: &str) -> Option<(usize, &Function)> {
        for (i, gf) in self.files.iter().enumerate() {
            for d in &gf.file.decls {
                if let Decl::Function(f) = d {
                    if f.name == name {
                        return Some((i, f));
                    }
                }
            }
        }
        None
    }

    pub fn named_value(&self, idx: usize, name: &str) -> Option<&NamedValue> {
        self.file(idx).decls.iter().find_map(|d| match d {
            Decl::NamedValue(v) if v.name == name => Some(v),
            _ => None,
        })
    }

    pub fn top_abbrev(&self, idx: usize, name: &str) -> Option<&Abbrev> {
        self.file(idx).decls.iter().find_map(|d| match d {
            Decl::Abbrev(a) if a.name == name => Some(a),
            _ => None,
        })
    }

    fn type_decl_in(&self, idx: usize, name: &str) -> Option<&TypeDecl> {
        self.file(idx).decls.iter().find_map(|d| match d {
            Decl::Type(t) if t.name == name => Some(t),
            _ => None,
        })
    }

    /// Own file first; otherwise the unique definition elsewhere in the graph.
    pub fn find_type(
        &self,
        from: usize,
        name: &str,
        line: usize,
    ) -> Result<(usize, &TypeDecl), Error> {
        if let Some(t) = self.type_decl_in(from, name) {
            return Ok((from, t));
        }
        let mut found = Vec::new();
        for i in 0..self.files.len() {
            if i == from {
                continue;
            }
            if let Some(t) = self.type_decl_in(i, name) {
                found.push((i, t));
            }
        }
        match found.len() {
            0 => Err(Error::new(
                ErrorKind::UnknownType,
                line,
                format!("unknown type `{name}`"),
            )),
            1 => Ok(found.pop().unwrap()),
            _ => Err(Error::new(
                ErrorKind::AmbiguousType,
                line,
                format!(
                    "type `{name}` is defined in several files: {}",
                    found
                        .iter()
                        .map(|(i, _)| self.files[*i].name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )),
        }
    }

    /// Fully structural type: names resolved, spreads flattened. Refinements
    /// are kept.
    pub fn expand(&self, from: usize, ty: &Type, line: usize) -> Result<Type, Error> {
        self.expand_depth(from, ty, line, 0)
    }

    fn expand_depth(
        &self,
        from: usize,
        ty: &Type,
        line: usize,
        depth: usize,
    ) -> Result<Type, Error> {
        if depth > 32 {
            return Err(Error::new(
                ErrorKind::TypeError,
                line,
                "type definition is circular",
            ));
        }
        Ok(match ty {
            Type::Named(n) => {
                let (idx, decl) = self.find_type(from, n, line)?;
                self.expand_depth(idx, &decl.ty, decl.line, depth + 1)?
            }
            Type::Record { spreads, fields } => {
                let mut all: Vec<(String, Type)> = Vec::new();
                for s in spreads {
                    let (idx, decl) = self.find_type(from, s, line)?;
                    match self.expand_depth(idx, &decl.ty, decl.line, depth + 1)? {
                        Type::Record { fields, .. } => all.extend(fields),
                        other => {
                            return Err(Error::new(
                                ErrorKind::TypeError,
                                line,
                                format!("`...{s}` spreads a non-record type {other}"),
                            ));
                        }
                    }
                }
                for (n, t) in fields {
                    if all.iter().any(|(m, _)| m == n) {
                        return Err(Error::new(
                            ErrorKind::DuplicateDefinition,
                            line,
                            format!("field `{n}` appears twice"),
                        ));
                    }
                    all.push((n.clone(), self.expand_depth(from, t, line, depth + 1)?));
                }
                Type::Record {
                    spreads: Vec::new(),
                    fields: all,
                }
            }
            Type::List(t, refs) => Type::List(
                Box::new(self.expand_depth(from, t, line, depth + 1)?),
                refs.clone(),
            ),
            Type::Option(t) => {
                Type::Option(Box::new(self.expand_depth(from, t, line, depth + 1)?))
            }
            Type::Result(t, e) => Type::Result(
                Box::new(self.expand_depth(from, t, line, depth + 1)?),
                Box::new(self.expand_depth(from, e, line, depth + 1)?),
            ),
            other => other.clone(),
        })
    }

    /// The type of a named value: declared, or inferred from its literal.
    pub fn named_value_type(&self, idx: usize, nv: &NamedValue) -> Result<Type, Error> {
        match &nv.ty {
            Some(t) => self.expand(idx, t, nv.line),
            None => Scope::top_level(self, idx).infer_literal(&nv.value, nv.line),
        }
    }

    /// Run every static check on one file. Returns the first error.
    pub fn check_file(&self, idx: usize) -> Result<(), Error> {
        let file = self.file(idx);
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for d in &file.decls {
            let (kind, name, line) = match d {
                Decl::Type(t) => ("type", t.name.as_str(), t.line),
                Decl::NamedValue(v) => ("name", v.name.as_str(), v.line),
                Decl::Abbrev(a) => ("name", a.name.as_str(), a.line),
                Decl::Function(f) => ("name", f.name.as_str(), f.line),
            };
            if seen.contains(&(kind, name)) {
                return Err(Error::new(
                    ErrorKind::DuplicateDefinition,
                    line,
                    format!("`{name}` is defined twice"),
                ));
            }
            seen.push((kind, name));
        }
        for d in &file.decls {
            match d {
                Decl::Type(t) => {
                    self.expand(idx, &t.ty, t.line)?;
                }
                Decl::NamedValue(v) => {
                    let scope = Scope::top_level(self, idx);
                    match &v.ty {
                        Some(t) => {
                            let ty = self.expand(idx, t, v.line)?;
                            scope.check_literal_type(&v.value, &ty, v.line)?;
                        }
                        None => {
                            scope.infer_literal(&v.value, v.line)?;
                        }
                    }
                }
                Decl::Abbrev(a) => {
                    let scope = Scope::top_level(self, idx);
                    scope.infer_abbrev_body(a)?;
                }
                Decl::Function(f) => self.check_function(idx, f)?,
            }
        }
        Ok(())
    }

    fn check_function(&self, idx: usize, f: &Function) -> Result<(), Error> {
        let scope = Scope::for_function(self, idx, f)?;
        for a in &f.abbrevs {
            scope.infer_abbrev_body(a)?;
        }
        for p in &f.properties {
            let ty = scope.infer(&p.expr, p.line)?;
            if !matches!(ty, Type::Bool | Type::Unknown) {
                return Err(Error::new(
                    ErrorKind::TypeError,
                    p.line,
                    format!("a property must be a Bool claim, this one is {ty}"),
                ));
            }
        }
        for a in &f.accepts {
            if a.expr.mentions("out") {
                return Err(Error::new(
                    ErrorKind::TypeError,
                    a.line,
                    "`accept` ranges over the inputs; `out` is not allowed in it",
                ));
            }
            if !f.inputs.iter().any(|(n, _)| a.expr.mentions(n)) {
                return Err(Error::new(
                    ErrorKind::TypeError,
                    a.line,
                    "`accept` must name an input: a predicate that is always true is refused",
                ));
            }
            let ty = scope.infer(&a.expr, a.line)?;
            if !matches!(ty, Type::Bool | Type::Unknown) {
                return Err(Error::new(
                    ErrorKind::TypeError,
                    a.line,
                    format!("`accept` must be a Bool over the inputs, this one is {ty}"),
                ));
            }
        }
        for e in &f.examples {
            scope.check_example_in(&e.input, e.line)?;
            scope.check_literal_type(&e.output, scope.output.as_ref().unwrap(), e.line)?;
        }
        Ok(())
    }
}

/// What names mean at one point in a file.
#[derive(Debug, Clone)]
pub struct Scope<'g> {
    pub graph: &'g Graph,
    pub file: usize,
    pub function: Option<&'g Function>,
    pub inputs: Vec<(String, Type)>,
    pub output: Option<Type>,
    pub binders: Vec<(String, Type)>,
    pub params: Vec<(String, Type)>,
    depth: usize,
}

fn textish(t: &Type) -> bool {
    matches!(t, Type::Text | Type::Id | Type::Time | Type::Uuid)
}

fn numeric(t: &Type) -> bool {
    matches!(t, Type::Int | Type::Float)
}

pub fn ordered(t: &Type) -> bool {
    matches!(
        t,
        Type::Int | Type::Float | Type::Text | Type::Time | Type::Unknown
    )
}

fn strip_option(t: &Type) -> &Type {
    match t {
        Type::Option(x) => x,
        other => other,
    }
}

/// Structural sameness, refinements ignored, `Unknown` matching anything,
/// and the text-like scalars treated as one family (a Text literal may name
/// an Id or a Time).
pub fn same_type(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Unknown, _) | (_, Type::Unknown) => true,
        (Type::List(x, _), Type::List(y, _)) => same_type(x, y),
        (Type::Option(x), Type::Option(y)) => same_type(x, y),
        (Type::Result(x1, x2), Type::Result(y1, y2)) => same_type(x1, y1) && same_type(x2, y2),
        (Type::Record { fields: fa, .. }, Type::Record { fields: fb, .. }) => {
            if fa.len() != fb.len() {
                return false;
            }
            let mut sa: Vec<&(String, Type)> = fa.iter().collect();
            let mut sb: Vec<&(String, Type)> = fb.iter().collect();
            sa.sort_by(|x, y| x.0.cmp(&y.0));
            sb.sort_by(|x, y| x.0.cmp(&y.0));
            sa.iter()
                .zip(sb.iter())
                .all(|(x, y)| x.0 == y.0 && same_type(&x.1, &y.1))
        }
        _ if textish(a) && textish(b) => true,
        _ => a == b,
    }
}

/// Whether `==` between the two makes sense: a bare value equals a present
/// Option holding it; a list compared with a scalar broadcasts.
pub fn compatible(a: &Type, b: &Type) -> bool {
    let (a, b) = (strip_option(a), strip_option(b));
    match (a, b) {
        (Type::Unknown, _) | (_, Type::Unknown) => true,
        (Type::List(x, _), Type::List(y, _)) => compatible(x, y),
        (Type::List(x, _), y) | (y, Type::List(x, _)) => compatible(x, y),
        _ if numeric(a) && numeric(b) => true,
        _ => same_type(a, b),
    }
}

impl<'g> Scope<'g> {
    pub fn top_level(graph: &'g Graph, file: usize) -> Scope<'g> {
        Scope {
            graph,
            file,
            function: None,
            inputs: Vec::new(),
            output: None,
            binders: Vec::new(),
            params: Vec::new(),
            depth: 0,
        }
    }

    pub fn for_function(
        graph: &'g Graph,
        file: usize,
        f: &'g Function,
    ) -> Result<Scope<'g>, Error> {
        let mut inputs = Vec::new();
        for (n, t) in &f.inputs {
            inputs.push((n.clone(), graph.expand(file, t, f.line)?));
        }
        let output = graph.expand(file, &f.output, f.line)?;
        Ok(Scope {
            graph,
            file,
            function: Some(f),
            inputs,
            output: Some(output),
            binders: Vec::new(),
            params: Vec::new(),
            depth: 0,
        })
    }

    pub fn bind(&self, name: &str, ty: Type) -> Scope<'g> {
        let mut s = self.clone();
        s.binders.push((name.to_string(), ty));
        s
    }

    fn with_params(&self, params: Vec<(String, Type)>) -> Scope<'g> {
        Scope {
            graph: self.graph,
            file: self.file,
            function: self.function,
            inputs: self.inputs.clone(),
            output: self.output.clone(),
            binders: Vec::new(),
            params,
            depth: self.depth + 1,
        }
    }

    fn local_abbrev(&self, name: &str) -> Option<&'g Abbrev> {
        self.function
            .and_then(|f| f.abbrevs.iter().find(|a| a.name == name))
    }

    /// Any abbreviation visible here: local first, then top-level.
    pub fn abbrev(&self, name: &str) -> Option<&'g Abbrev> {
        self.local_abbrev(name)
            .or_else(|| self.graph.top_abbrev(self.file, name))
    }

    pub fn is_input(&self, name: &str) -> bool {
        self.inputs.iter().any(|(n, _)| n == name)
    }

    pub fn infer_abbrev_body(&self, a: &Abbrev) -> Result<Type, Error> {
        if self.depth > 16 {
            return Err(Error::new(
                ErrorKind::TypeError,
                a.line,
                format!("abbreviation `{}` refers to itself", a.name),
            ));
        }
        let mut params = Vec::new();
        for (n, t) in &a.params {
            params.push((n.clone(), self.graph.expand(self.file, t, a.line)?));
        }
        self.with_params(params).infer(&a.body, a.line)
    }

    /// The type of a bare name, or an unbound-name error.
    pub fn lookup(&self, name: &str, line: usize) -> Result<Type, Error> {
        if let Some((_, t)) = self.params.iter().rev().find(|(n, _)| n == name) {
            return Ok(t.clone());
        }
        if let Some((_, t)) = self.binders.iter().rev().find(|(n, _)| n == name) {
            return Ok(t.clone());
        }
        if let Some((_, t)) = self.inputs.iter().find(|(n, _)| n == name) {
            return Ok(t.clone());
        }
        if name == "out" {
            if let Some(t) = &self.output {
                return Ok(t.clone());
            }
        }
        if let Some(a) = self.abbrev(name) {
            if !a.params.is_empty() {
                return Err(Error::new(
                    ErrorKind::TypeError,
                    line,
                    format!("abbreviation `{name}` takes parameters"),
                ));
            }
            return self.infer_abbrev_body(a);
        }
        if let Some(v) = self.graph.named_value(self.file, name) {
            return self.graph.named_value_type(self.file, v);
        }
        Err(Error::new(
            ErrorKind::UnboundName,
            line,
            format!(
                "`{name}` is not an input, `out`, a binder here, a named value, or an abbreviation"
            ),
        ))
    }

    fn err(&self, line: usize, msg: impl Into<String>) -> Error {
        Error::new(ErrorKind::TypeError, line, msg)
    }

    fn expect_bool(&self, e: &Expr, line: usize) -> Result<(), Error> {
        let t = self.infer(e, line)?;
        if matches!(t, Type::Bool | Type::Unknown) {
            Ok(())
        } else {
            Err(self.err(line, format!("expected a Bool, found {t}")))
        }
    }

    /// An operand of `+ - *` or `abs` is an `Int`. The refusals name what
    /// the other types wait on, since each is a reach-for with a known
    /// answer.
    fn expect_arith(&self, what: &str, t: &Type, line: usize) -> Result<(), Error> {
        let msg = match t {
            Type::Int | Type::Unknown => return Ok(()),
            Type::Float => format!(
                "`{what}` on Float waits for the tolerance decision; write amounts in cents and rates in basis points, which are exact Ints"
            ),
            Type::Time => format!(
                "`{what}` on Time: the language has no time arithmetic; Time values compare, they do not add"
            ),
            t if textish(t) => {
                format!("`{what}` on {t}: no arithmetic on text; `len` gives its length")
            }
            Type::List(..) => {
                format!("`{what}` on a list: `sum` and `len` turn a list into one Int")
            }
            t => format!("`{what}` needs Int operands, found {t}"),
        };
        Err(self.err(line, msg))
    }

    fn expect_list(&self, e: &Expr, line: usize, what: &str) -> Result<Type, Error> {
        match self.infer(e, line)? {
            Type::List(t, _) => Ok(*t),
            Type::Unknown => Ok(Type::Unknown),
            other => Err(self.err(line, format!("{what} needs a list, found {other}"))),
        }
    }

    /// The type of an expression, checking it as we go.
    pub fn infer(&self, e: &Expr, line: usize) -> Result<Type, Error> {
        match e {
            Expr::Int(_) => Ok(Type::Int),
            Expr::Float(_) => Ok(Type::Float),
            Expr::Text(_) => Ok(Type::Text),
            Expr::None => Ok(Type::Option(Box::new(Type::Unknown))),
            Expr::Name(n, l) => self.lookup(n, *l),
            Expr::Field(base, f, l) => {
                let t = self.infer(base, *l)?;
                let field_of = |rec: &Type| -> Result<Type, Error> {
                    match rec {
                        Type::Record { fields, .. } => fields
                            .iter()
                            .find(|(n, _)| n == f)
                            .map(|(_, t)| t.clone())
                            .ok_or_else(|| {
                                Error::new(
                                    ErrorKind::UnknownField,
                                    *l,
                                    format!("no field `{f}` on {rec}"),
                                )
                            }),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(Error::new(
                            ErrorKind::NotARecord,
                            *l,
                            format!(
                                "`{}` is {other}, not a record, so it has no field `{f}`",
                                crate::render::expr(base)
                            ),
                        )),
                    }
                };
                match &t {
                    Type::List(inner, _) => Ok(Type::list(field_of(inner)?)),
                    other => field_of(other),
                }
            }
            Expr::Call(name, args, l) => self.infer_call(name, args, *l),
            Expr::Ok(x) => Ok(Type::Result(
                Box::new(self.infer(x, line)?),
                Box::new(Type::Unknown),
            )),
            Expr::Err(x) => Ok(Type::Result(
                Box::new(Type::Unknown),
                Box::new(self.infer(x, line)?),
            )),
            Expr::List(xs) => {
                let mut elem = Type::Unknown;
                for x in xs {
                    let t = self.infer(x, line)?;
                    if elem.is_unknown() {
                        elem = t;
                    } else if !same_type(&elem, &t) {
                        return Err(self.err(line, format!("list mixes {elem} and {t}")));
                    }
                }
                Ok(Type::list(elem))
            }
            Expr::Record(fields, _) => {
                let mut out = Vec::new();
                for (n, x) in fields {
                    out.push((n.clone(), self.infer(x, line)?));
                }
                Ok(Type::Record {
                    spreads: Vec::new(),
                    fields: out,
                })
            }
            Expr::Tuple(_) => Err(self.err(
                line,
                "a tuple is only the `in:` of a several-input function",
            )),
            Expr::Not(x) => {
                self.expect_bool(x, line)?;
                Ok(Type::Bool)
            }
            Expr::Binary(op, a, b) => {
                match op {
                    BinOp::And | BinOp::Or | BinOp::Implies | BinOp::Iff => {
                        self.expect_bool(a, line)?;
                        self.expect_bool(b, line)?;
                    }
                    BinOp::Eq | BinOp::Ne => {
                        let ta = self.infer(a, line)?;
                        let tb = self.infer(b, line)?;
                        if !compatible(&ta, &tb) {
                            return Err(self.err(line, format!("cannot compare {ta} with {tb}")));
                        }
                    }
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        let ta = self.infer(a, line)?;
                        let tb = self.infer(b, line)?;
                        if !ordered(&ta) || !ordered(&tb) {
                            return Err(self.err(
                                line,
                                format!(
                                    "`{}` needs ordered values; found {ta} and {tb}",
                                    op.symbol()
                                ),
                            ));
                        }
                        if !compatible(&ta, &tb) {
                            return Err(self.err(line, format!("cannot compare {ta} with {tb}")));
                        }
                    }
                    BinOp::In | BinOp::NotIn => {
                        let ta = self.infer(a, line)?;
                        let tb = self.infer(b, line)?;
                        let ok = match &tb {
                            Type::List(elem, _) => compatible(&ta, elem),
                            Type::Unknown => true,
                            t if textish(t) => textish(strip_option(&ta)) || ta.is_unknown(),
                            _ => false,
                        };
                        if !ok {
                            return Err(self.err(
                                line,
                                format!(
                                    "`in` needs a list or Text on the right; found {ta} in {tb}"
                                ),
                            ));
                        }
                    }
                }
                Ok(Type::Bool)
            }
            Expr::Arith(op, a, b) => {
                let ta = self.infer(a, line)?;
                let tb = self.infer(b, line)?;
                for t in [&ta, &tb] {
                    self.expect_arith(op.symbol(), t, line)?;
                }
                Ok(Type::Int)
            }
            Expr::Slice { list, from, to } => {
                let elem = self.expect_list(list, line, "a slice")?;
                for pos in [from, to] {
                    let t = self.infer(pos, line)?;
                    if !matches!(t, Type::Int | Type::Unknown) {
                        return Err(
                            self.err(line, format!("a slice position is an Int; found {t}"))
                        );
                    }
                }
                Ok(Type::list(elem))
            }
            Expr::Before { a, b, list } => {
                let elem = self.expect_list(list, line, "`before`")?;
                let ta = self.infer(a, line)?;
                let tb = self.infer(b, line)?;
                if !compatible(&ta, &elem) || !compatible(&tb, &elem) {
                    return Err(self.err(line, "`before` compares elements of the list"));
                }
                Ok(Type::Bool)
            }
            Expr::Relation(rel, a, b) => {
                let ea = self.expect_list(a, line, rel.symbol())?;
                let eb = self.expect_list(b, line, rel.symbol())?;
                if !same_type(&ea, &eb) {
                    let (sa, sb) = (crate::render::expr(a), crate::render::expr(b));
                    // One side a list of lists of the other's elements, the
                    // usual way to meet this: a field broadcast over a list
                    // of records that each hold a list. `flatten` joins them.
                    let nested = |outer: &Type, inner: &Type| matches!(outer, Type::List(t, _) if same_type(t, inner));
                    let join = |side: &str| Fix {
                        title: format!("Join the lists: flatten({side})"),
                        find: side.to_string(),
                        replace: format!("flatten({side})"),
                    };
                    let (message, fix) = if nested(&ea, &eb) {
                        (
                            format!(
                                "`{}` relates lists of different element types: each element of `{sa}` is itself a list of {eb}, while `{sb}` holds {eb}; `flatten({sa})` joins those lists into one",
                                rel.symbol()
                            ),
                            Some(join(&sa)),
                        )
                    } else if nested(&eb, &ea) {
                        (
                            format!(
                                "`{}` relates lists of different element types: `{sa}` holds {ea}, while each element of `{sb}` is itself a list of {ea}; `flatten({sb})` joins those lists into one",
                                rel.symbol()
                            ),
                            Some(join(&sb)),
                        )
                    } else {
                        (
                            format!(
                                "`{}` relates lists of different element types: `{sa}` holds {ea}, and `{sb}` holds {eb}",
                                rel.symbol()
                            ),
                            None,
                        )
                    };
                    let e = Error::new(ErrorKind::ElementTypeMismatch, line, message);
                    return Err(match fix {
                        Some(f) => e.fixed_by(f),
                        None => e,
                    });
                }
                Ok(Type::Bool)
            }
            Expr::SortedBy(x, keys) => {
                let elem = self.expect_list(x, line, "`sorted_by`")?;
                if let Type::Record { fields, .. } = &elem {
                    for (k, _) in keys {
                        let Some((_, kt)) = fields.iter().find(|(n, _)| n == k) else {
                            return Err(Error::new(
                                ErrorKind::UnknownField,
                                line,
                                format!("no sort key `{k}` on {elem}"),
                            ));
                        };
                        if !ordered(kt) {
                            return Err(
                                self.err(line, format!("sort key `{k}` is not ordered ({kt})"))
                            );
                        }
                    }
                } else if !elem.is_unknown() {
                    return Err(self.err(line, "`sorted_by` needs a list of records"));
                }
                Ok(Type::Bool)
            }
            Expr::Unordered(x) => {
                self.expect_list(x, line, "`unordered`")?;
                Ok(Type::Bool)
            }
            Expr::Unique(x) => {
                self.expect_list(x, line, "`unique`")?;
                Ok(Type::Bool)
            }
            Expr::Quantifier(q) => self.infer_quantifier(q),
        }
    }

    fn infer_call(&self, name: &str, args: &[Expr], line: usize) -> Result<Type, Error> {
        if BUILTINS.contains(&name) {
            let tys: Vec<Type> = args
                .iter()
                .map(|a| self.infer(a, line))
                .collect::<Result<_, _>>()?;
            return match name {
                "sum" => match tys.as_slice() {
                    [Type::List(t, _)] if numeric(t) || t.is_unknown() => Ok((**t).clone()),
                    [Type::Unknown] => Ok(Type::Unknown),
                    _ => Err(self.err(line, "`sum` takes one list of numbers")),
                },
                "len" => match tys.as_slice() {
                    [Type::List(..)] | [Type::Unknown] => Ok(Type::Int),
                    [t] if textish(t) => Ok(Type::Int),
                    _ => Err(self.err(line, "`len` takes one list or Text")),
                },
                "count" => match tys.as_slice() {
                    [Type::List(..)] | [Type::Unknown] => Ok(Type::Int),
                    _ => Err(self.err(line, "`count` takes one list")),
                },
                "min" | "max" => {
                    if tys.len() < 2 {
                        return Err(self.err(line, format!("`{name}` takes two or more scalars")));
                    }
                    let mut result = Type::Unknown;
                    for t in &tys {
                        if !ordered(t) || matches!(t, Type::List(..)) {
                            return Err(self
                                .err(line, format!("`{name}` takes ordered scalars, found {t}")));
                        }
                        if result.is_unknown() {
                            result = t.clone();
                        } else if !compatible(&result, t) {
                            return Err(self.err(line, format!("`{name}` mixes {result} and {t}")));
                        }
                    }
                    Ok(result)
                }
                "abs" => match tys.as_slice() {
                    [Type::Int] | [Type::Unknown] => Ok(Type::Int),
                    [t] => {
                        self.expect_arith("abs", t, line)?;
                        Ok(Type::Int)
                    }
                    _ => Err(self.err(line, "`abs` takes one Int")),
                },
                "flatten" => match tys.as_slice() {
                    [Type::List(inner, _)] => match &**inner {
                        Type::List(t, _) => Ok(Type::list((**t).clone())),
                        Type::Unknown => Ok(Type::Unknown),
                        _ => Err(self.err(line, "`flatten` takes a list of lists")),
                    },
                    [Type::Unknown] => Ok(Type::Unknown),
                    _ => Err(self.err(line, "`flatten` takes a list of lists")),
                },
                _ => unreachable!(),
            };
        }
        if let Some(a) = self.abbrev(name) {
            if a.params.len() != args.len() {
                return Err(self.err(
                    line,
                    format!(
                        "`{name}` takes {} parameter(s), called with {}",
                        a.params.len(),
                        args.len()
                    ),
                ));
            }
            let mut params = Vec::new();
            for ((pn, pt), arg) in a.params.iter().zip(args) {
                let pt = self.graph.expand(self.file, pt, a.line)?;
                let at = self.infer(arg, line)?;
                if !compatible(&at, &pt) {
                    return Err(self.err(
                        line,
                        format!("`{name}` parameter `{pn}` is {pt}, argument is {at}"),
                    ));
                }
                params.push((pn.clone(), pt));
            }
            if self.depth > 16 {
                return Err(self.err(line, format!("abbreviation `{name}` refers to itself")));
            }
            return self.with_params(params).infer(&a.body, a.line);
        }
        if self.graph.named_value(self.file, name).is_some() {
            return Err(self.err(line, format!("`{name}` is a value, not a function")));
        }
        // A graph reference: another obligation's function, or one we only
        // meet at check time.
        if let Some((idx, f)) = self.graph.find_function(name) {
            if f.inputs.len() != args.len() {
                return Err(self.err(
                    line,
                    format!(
                        "`{name}` takes {} input(s), called with {}",
                        f.inputs.len(),
                        args.len()
                    ),
                ));
            }
            for ((pn, pt), arg) in f.inputs.iter().zip(args) {
                let pt = self.graph.expand(idx, pt, f.line)?;
                let at = self.infer(arg, line)?;
                if !compatible(&at, &pt) {
                    return Err(self.err(
                        line,
                        format!("`{name}` input `{pn}` is {pt}, argument is {at}"),
                    ));
                }
            }
            return self.graph.expand(idx, &f.output, f.line);
        }
        for a in args {
            self.infer(a, line)?;
        }
        Ok(Type::Unknown)
    }

    /// The element type a quantifier binds, given its domain and pattern.
    pub fn domain_elem(&self, q: &Quantifier) -> Result<Type, Error> {
        let dom = self.infer(&q.domain, q.line)?;
        let payload = |t: &Type| -> Result<Type, Error> {
            match (t, q.binder.side) {
                (Type::Result(ok, _), Some(Side::Ok)) => Ok((**ok).clone()),
                (Type::Result(_, err), Some(Side::Err)) => Ok((**err).clone()),
                (Type::Unknown, _) => Ok(Type::Unknown),
                (_, Some(_)) => Err(self.err(
                    q.line,
                    format!("an `Ok`/`Err` pattern needs a Result domain, found {t}"),
                )),
                (other, None) => Ok(other.clone()),
            }
        };
        match &dom {
            Type::List(t, _) => payload(t),
            Type::Result(..) => payload(&dom),
            Type::Option(t) => {
                if q.binder.side.is_some() {
                    return Err(self.err(
                        q.line,
                        "an `Ok`/`Err` pattern needs a Result domain, found an Option",
                    ));
                }
                Ok((**t).clone())
            }
            Type::Unknown => Ok(Type::Unknown),
            other => Err(self.err(
                q.line,
                format!("a quantifier ranges over a list, Result, or Option; found {other}"),
            )),
        }
    }

    fn infer_quantifier(&self, q: &Quantifier) -> Result<Type, Error> {
        let elem = self.domain_elem(q)?;
        if q.leftover {
            let out = self
                .output
                .as_ref()
                .ok_or_else(|| self.err(q.line, "`leftover` needs an `out`"))?;
            match out {
                Type::List(u, _) if same_type(u, &elem) => {}
                Type::Unknown => {}
                other => {
                    return Err(self.err(
                        q.line,
                        format!("`leftover` subtracts `out` from the domain; `out` is {other}, elements are {elem}"),
                    ));
                }
            }
        }
        let inner = match &q.binder.name {
            Some(n) => self.bind(n, elem),
            None => self.clone(),
        };
        if let Some(w) = &q.where_ {
            inner.expect_bool(w, q.line)?;
        }
        if let Some(b) = &q.body {
            inner.expect_bool(b, q.line)?;
        }
        if let Some(e) = &q.else_ {
            inner.expect_bool(e, q.line)?;
        }
        Ok(Type::Bool)
    }

    /// The type of a literal used as a named value without a declared type.
    pub fn infer_literal(&self, e: &Expr, line: usize) -> Result<Type, Error> {
        match e {
            Expr::List(xs) if xs.is_empty() => Err(self.err(
                line,
                "an untyped named value needs at least one element to infer from",
            )),
            _ => self.infer(e, line),
        }
    }

    /// Check an example's `in:` against the inputs.
    pub fn check_example_in(&self, e: &Expr, line: usize) -> Result<(), Error> {
        match (self.inputs.len(), e) {
            (1, Expr::Tuple(_)) => Err(Error::new(
                ErrorKind::ExampleInNotLiteral,
                line,
                "the function has one input; `in:` should be its literal, not a tuple",
            )),
            (1, _) => self.check_literal_type(e, &self.inputs[0].1, line),
            (n, Expr::Tuple(xs)) if xs.len() == n => {
                for (x, (_, t)) in xs.iter().zip(&self.inputs) {
                    self.check_literal_type(x, t, line)?;
                }
                Ok(())
            }
            (n, _) => Err(Error::new(
                ErrorKind::ExampleInNotLiteral,
                line,
                format!("the function has {n} inputs; `in:` should be a tuple of {n}"),
            )),
        }
    }

    /// Check a literal against a declared (expanded) type. A record with
    /// `...` states a subset of its fields; the parser already forbids `...`
    /// where the frame rule cannot fill it.
    pub fn check_literal_type(&self, e: &Expr, ty: &Type, line: usize) -> Result<(), Error> {
        let bad = |what: &str| Err(self.err(line, format!("{what} does not fit the type {ty}")));
        match (e, ty) {
            (_, Type::Unknown) => Ok(()),
            (Expr::Name(n, l), _) => {
                let t = self.lookup(n, *l)?;
                if same_type(&t, ty) || compatible(&t, ty) && !matches!(ty, Type::List(..)) {
                    Ok(())
                } else {
                    Err(self.err(
                        line,
                        format!("`{n}` is {t}, which does not fit the type {ty}"),
                    ))
                }
            }
            (Expr::None, Type::Option(_)) => Ok(()),
            (_, Type::Option(inner)) => self.check_literal_type(e, inner, line),
            (Expr::Int(_), Type::Int | Type::Float) => Ok(()),
            (Expr::Float(_), Type::Float) => Ok(()),
            (Expr::Text(_), t) if textish(t) => Ok(()),
            (Expr::Ok(x), Type::Result(t, _)) => self.check_literal_type(x, t, line),
            (Expr::Err(x), Type::Result(_, t)) => self.check_literal_type(x, t, line),
            (Expr::List(xs), Type::List(t, _)) => {
                for x in xs {
                    self.check_literal_type(x, t, line)?;
                }
                Ok(())
            }
            (Expr::Record(fields, rest), Type::Record { fields: tf, .. }) => {
                for (n, x) in fields {
                    let Some((_, t)) = tf.iter().find(|(m, _)| m == n) else {
                        return Err(Error::new(
                            ErrorKind::UnknownField,
                            line,
                            format!("no field `{n}` on {ty}"),
                        ));
                    };
                    self.check_literal_type(x, t, line)?;
                }
                if !*rest {
                    for (n, _) in tf {
                        if !fields.iter().any(|(m, _)| m == n) {
                            return Err(self.err(
                                line,
                                format!("record literal is missing field `{n}` of {ty}"),
                            ));
                        }
                    }
                }
                Ok(())
            }
            (Expr::Tuple(_), _) => bad("a tuple"),
            (Expr::Int(_), _) => bad("an Int literal"),
            (Expr::Float(_), _) => bad("a Float literal"),
            (Expr::Text(_), _) => bad("a Text literal"),
            (Expr::None, _) => bad("`None`"),
            (Expr::Ok(_), _) => bad("`Ok(...)`"),
            (Expr::Err(_), _) => bad("`Err(...)`"),
            (Expr::List(_), _) => bad("a list literal"),
            (Expr::Record(..), _) => bad("a record literal"),
            _ => bad("this expression"),
        }
    }
}
