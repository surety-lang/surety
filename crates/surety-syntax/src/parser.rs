//! Parser: logical lines to the AST, with the syntactic checks the brief
//! lists (quantifier mid-line, two bodies, unused binder, `=` for `==`,
//! `else` without `where`, `where` without a body, a pattern as a value).

use crate::ast::*;
use crate::error::{Error, ErrorKind};
use crate::lexer::{Line, Tok, Token, lex};

pub const BUILTINS: &[&str] = &["sum", "len", "min", "max", "count", "flatten"];

/// The type names `parse_type` knows without a declaration.
pub const PRIMITIVES: &[&str] = &[
    "Int", "Float", "Text", "Bool", "Id", "Time", "UUID", "Option", "Result",
];

pub const RESERVED: &[&str] = &[
    "in",
    "where",
    "else",
    "and",
    "or",
    "not",
    "leftover",
    "before",
    "unordered",
    "unique",
    "sorted_by",
    "same_order_as",
    "same_elements_as",
    "drawn_from",
    "asc",
    "desc",
    "type",
    "all",
    "any",
    "none",
    "exactly_one",
    "None",
    "Ok",
    "Err",
    "accept",
];

pub fn is_reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}

#[derive(Debug, Clone)]
struct Item {
    line: Line,
    children: Vec<Item>,
}

fn is_deeper(child: &str, parent: &str) -> bool {
    child.len() > parent.len() && child.starts_with(parent)
}

fn build_block(lines: &[Line], pos: &mut usize, parent: Option<&str>) -> Result<Vec<Item>, Error> {
    let mut items: Vec<Item> = Vec::new();
    if *pos >= lines.len() {
        return Ok(items);
    }
    let base = lines[*pos].indent.clone();
    match parent {
        None if !base.is_empty() => {
            return Err(Error::new(
                ErrorKind::UnexpectedIndent,
                lines[*pos].line,
                "unexpected indentation at top level",
            ));
        }
        Some(p) if !is_deeper(&base, p) => {
            return Err(Error::new(
                ErrorKind::UnexpectedIndent,
                lines[*pos].line,
                "inconsistent indentation",
            ));
        }
        _ => {}
    }
    while *pos < lines.len() {
        let ind = lines[*pos].indent.clone();
        if ind == base {
            items.push(Item {
                line: lines[*pos].clone(),
                children: Vec::new(),
            });
            *pos += 1;
        } else if is_deeper(&ind, &base) {
            let last = items.last_mut().expect("block starts at base indent");
            if !last.children.is_empty() {
                return Err(Error::new(
                    ErrorKind::UnexpectedIndent,
                    lines[*pos].line,
                    "inconsistent indentation (mixed tabs and spaces?)",
                ));
            }
            last.children = build_block(lines, pos, Some(&base))?;
        } else if base.starts_with(ind.as_str()) {
            break;
        } else {
            return Err(Error::new(
                ErrorKind::UnexpectedIndent,
                lines[*pos].line,
                "inconsistent indentation (mixed tabs and spaces?)",
            ));
        }
    }
    Ok(items)
}

/// The source of an item and its vertical body, lines joined by spaces.
fn source_text(item: &Item) -> String {
    let mut parts = vec![item.line.text.clone()];
    fn walk(items: &[Item], parts: &mut Vec<String>) {
        for it in items {
            parts.push(it.line.text.clone());
            walk(&it.children, parts);
        }
    }
    walk(&item.children, &mut parts);
    parts.join(" ")
}

fn flatten_tokens(items: &[Item], out: &mut Vec<Token>) {
    for it in items {
        out.extend(it.line.tokens.iter().cloned());
        flatten_tokens(&it.children, out);
    }
}

/// The markers of a choice left open, as the editor's quick fix writes two
/// passing outputs for the author to choose between, and as a merge does.
const CHOICE_MARKERS: &[&str] = &["<<<<<<<", "=======", ">>>>>>>"];

/// Parse a whole `.sur` file.
pub fn parse(src: &str) -> Result<File, Error> {
    if let Some((i, _)) = src
        .lines()
        .enumerate()
        .find(|(_, l)| CHOICE_MARKERS.iter().any(|m| l.starts_with(m)))
    {
        return Err(Error::new(
            ErrorKind::UnresolvedChoice,
            i + 1,
            "a choice left open: keep one side, and delete the side you do not mean and the three marker lines",
        )
        .spanning(Some((0, 7))));
    }
    let lexed = lex(src)?;
    let mut pos = 0;
    let items = build_block(&lexed.lines, &mut pos, None)?;
    let mut file = File {
        header: lexed.header,
        decls: Vec::new(),
        pins: lexed.pins,
    };
    for item in &items {
        let decl = parse_decl(item, &mut file.pins)?;
        file.decls.push(decl);
    }
    file.pins.sort_by_key(|p| p.line);
    Ok(file)
}

fn syntax(line: usize, msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Syntax, line, msg)
}

fn matching_paren(toks: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, t) in toks.iter().enumerate().skip(open) {
        if t.is_sym("(") {
            depth += 1;
        } else if t.is_sym(")") {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn parse_decl(item: &Item, pins: &mut Vec<Pin>) -> Result<Decl, Error> {
    let toks = &item.line.tokens;
    let line = item.line.line;
    let first = toks.first().ok_or_else(|| syntax(line, "empty line"))?;
    if first.is_ident("type") {
        if !item.children.is_empty() {
            return Err(Error::new(
                ErrorKind::UnexpectedIndent,
                item.children[0].line.line,
                "unexpected indented block under a type",
            ));
        }
        let mut p = Cursor::new(toks, 1, line);
        let name = p.expect_name("type name")?;
        p.expect_sym("=")?;
        let ty = p.parse_type()?;
        p.expect_end()?;
        return Ok(Decl::Type(TypeDecl { name, ty, line }));
    }
    let name = match first.ident() {
        Some(n) if !is_reserved(n) => n.to_string(),
        _ => {
            return Err(syntax(
                line,
                format!("expected a declaration, found {}", first.describe()),
            ));
        }
    };
    let second = toks
        .get(1)
        .ok_or_else(|| syntax(line, "expected a declaration"))?;
    if second.is_sym(":") {
        if !item.children.is_empty() {
            return Err(Error::new(
                ErrorKind::UnexpectedIndent,
                item.children[0].line.line,
                "unexpected indented block under a named value",
            ));
        }
        let mut p = Cursor::new(toks, 2, line);
        let ty = p.parse_type()?;
        p.expect_sym("=")?;
        let value = parse_expr(&toks[p.pos..], &[], Ctx::Literal, line)?;
        check_literal(&value, false, ErrorKind::Syntax, line, "a named value")?;
        return Ok(Decl::NamedValue(NamedValue {
            name,
            ty: Some(ty),
            value,
            line,
        }));
    }
    if second.is_sym("=") {
        let value = parse_expr(&toks[2..], &item.children, Ctx::Property, line)?;
        if is_literal_shaped(&value) {
            check_literal(&value, false, ErrorKind::Syntax, line, "a named value")?;
            return Ok(Decl::NamedValue(NamedValue {
                name,
                ty: None,
                value,
                line,
            }));
        }
        return Ok(Decl::Abbrev(Abbrev {
            name,
            params: Vec::new(),
            body: value,
            line,
        }));
    }
    if second.is_sym("(") {
        let close = matching_paren(toks, 1).ok_or_else(|| syntax(line, "unclosed `(`"))?;
        let mut p = Cursor::new(&toks[..close], 2, line);
        let params = p.parse_params()?;
        let after = toks
            .get(close + 1)
            .ok_or_else(|| syntax(line, "expected `=` or `->` after the parameter list"))?;
        if after.is_sym("=") {
            let body = parse_expr(&toks[close + 2..], &item.children, Ctx::Property, line)?;
            return Ok(Decl::Abbrev(Abbrev {
                name,
                params,
                body,
                line,
            }));
        }
        if after.is_sym("->") {
            let mut p = Cursor::new(toks, close + 2, line);
            let output = p.parse_type()?;
            p.expect_end()?;
            let mut fun = Function {
                name,
                inputs: params,
                output,
                line,
                note: None,
                abbrevs: Vec::new(),
                properties: Vec::new(),
                accepts: Vec::new(),
                examples: Vec::new(),
            };
            parse_body(&mut fun, &item.children, pins)?;
            return Ok(Decl::Function(fun));
        }
        return Err(syntax(
            line,
            format!(
                "expected `=` or `->` after the parameter list, found {}",
                after.describe()
            ),
        ));
    }
    Err(syntax(
        line,
        format!("expected a declaration, found {}", second.describe()),
    ))
}

fn is_literal_shaped(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Int(_)
            | Expr::Float(_)
            | Expr::Text(_)
            | Expr::None
            | Expr::List(_)
            | Expr::Record(..)
            | Expr::Ok(_)
            | Expr::Err(_)
    )
}

fn is_clause_start(toks: &[Token]) -> bool {
    match (toks.first(), toks.get(1)) {
        (Some(a), Some(b)) if a.is_ident("note") => matches!(b.tok, Tok::Str(_)),
        (Some(a), Some(b)) if a.is_ident("in") || a.is_ident("out") => b.is_sym(":"),
        _ => false,
    }
}

fn abbrev_shape(toks: &[Token]) -> Option<(String, Option<usize>)> {
    let name = toks.first()?.ident()?;
    if is_reserved(name) && name != "None" {
        return None;
    }
    let second = toks.get(1)?;
    if second.is_sym("=") {
        return Some((name.to_string(), None));
    }
    if second.is_sym("(") {
        let close = matching_paren(toks, 1)?;
        if toks.get(close + 1)?.is_sym("=") {
            return Some((name.to_string(), Some(close)));
        }
    }
    None
}

fn parse_body(fun: &mut Function, items: &[Item], pins: &mut Vec<Pin>) -> Result<(), Error> {
    let mut i = 0;
    while i < items.len() {
        let it = &items[i];
        let toks = &it.line.tokens;
        let line = it.line.line;
        let first = toks.first().ok_or_else(|| syntax(line, "empty line"))?;
        if first.is_ident("note") && matches!(toks.get(1).map(|t| &t.tok), Some(Tok::Str(_))) {
            if fun.note.is_some() {
                return Err(syntax(line, "a second `note` on the obligation"));
            }
            if !fun.examples.is_empty() {
                return Err(syntax(
                    line,
                    "`note` after the examples belongs to no example",
                ));
            }
            if toks.len() > 2 {
                return Err(syntax(line, "unexpected tokens after `note`"));
            }
            if !it.children.is_empty() {
                return Err(Error::new(
                    ErrorKind::UnexpectedIndent,
                    it.children[0].line.line,
                    "unexpected indented block under `note`",
                ));
            }
            let text = match &toks[1].tok {
                Tok::Str(s) => s.clone(),
                _ => unreachable!(),
            };
            pins.push(Pin {
                line,
                kind: PinKind::Note,
                text: text.clone(),
            });
            fun.note = Some(text);
            i += 1;
            continue;
        }
        if first.is_ident("e") && toks.get(1).is_some_and(|t| t.is_sym(":")) {
            let mut all = toks[2..].to_vec();
            flatten_tokens(&it.children, &mut all);
            i += 1;
            while i < items.len() && is_clause_start(&items[i].line.tokens) {
                all.extend(items[i].line.tokens.iter().cloned());
                flatten_tokens(&items[i].children, &mut all);
                i += 1;
            }
            let ex = parse_example(&all, line)?;
            if let Some(n) = &ex.note {
                pins.push(Pin {
                    line,
                    kind: PinKind::Note,
                    text: n.clone(),
                });
            }
            fun.examples.push(ex);
            continue;
        }
        if let Some((name, close)) = abbrev_shape(toks) {
            if name == "out"
                || fun.inputs.iter().any(|(n, _)| *n == name)
                || BUILTINS.contains(&name.as_str())
            {
                return Err(Error::new(
                    ErrorKind::AssignmentInProperty,
                    line,
                    format!("`{name} =` in a property; use `==` (`=` only defines abbreviations)"),
                ));
            }
            let (params, rest) = match close {
                None => (Vec::new(), 2),
                Some(close) => {
                    let mut p = Cursor::new(&toks[..close], 2, line);
                    (p.parse_params()?, close + 2)
                }
            };
            let body = parse_expr(&toks[rest..], &it.children, Ctx::Property, line)?;
            fun.abbrevs.push(Abbrev {
                name,
                params,
                body,
                line,
            });
            i += 1;
            continue;
        }
        if first.is_ident("accept") {
            if toks.len() == 1 && it.children.is_empty() {
                return Err(syntax(line, "`accept` needs a Bool over the inputs"));
            }
            let expr = parse_expr(&toks[1..], &it.children, Ctx::Property, line)?;
            fun.accepts.push(Property {
                expr,
                line,
                sentence: it.line.sentence.clone(),
                text: source_text(it),
            });
            i += 1;
            continue;
        }
        let expr = parse_expr(toks, &it.children, Ctx::Property, line)?;
        fun.properties.push(Property {
            expr,
            line,
            sentence: it.line.sentence.clone(),
            text: source_text(it),
        });
        i += 1;
    }
    Ok(())
}

fn parse_example(toks: &[Token], line: usize) -> Result<Example, Error> {
    let mut p = ExprParser {
        toks,
        pos: 0,
        ctx: Ctx::Literal,
        block: &[],
        block_used: false,
        line,
        q_done: false,
    };
    let mut note = None;
    let mut input = None;
    let mut output = None;
    while let Some(t) = p.peek() {
        let tline = t.line;
        if t.is_ident("note") {
            p.bump();
            match p.bump().map(|t| &t.tok) {
                Some(Tok::Str(s)) => {
                    if note.replace(s.clone()).is_some() {
                        return Err(syntax(tline, "a second `note` on the example"));
                    }
                }
                _ => return Err(syntax(tline, "expected a string after `note`")),
            }
        } else if t.is_ident("in") && p.peek_at(1).is_some_and(|t| t.is_sym(":")) {
            p.bump();
            p.bump();
            let e = p.parse_iff()?;
            check_literal(
                &e,
                false,
                ErrorKind::ExampleInNotLiteral,
                tline,
                "an example's `in`",
            )?;
            if input.replace(e).is_some() {
                return Err(syntax(tline, "a second `in:` on the example"));
            }
        } else if t.is_ident("out") && p.peek_at(1).is_some_and(|t| t.is_sym(":")) {
            p.bump();
            p.bump();
            let e = p.parse_iff()?;
            check_literal(
                &e,
                true,
                ErrorKind::ExampleOutNotLiteral,
                tline,
                "an example's `out`",
            )?;
            if output.replace(e).is_some() {
                return Err(syntax(tline, "a second `out:` on the example"));
            }
        } else {
            return Err(syntax(
                tline,
                format!(
                    "expected `note`, `in:` or `out:` in an example, found {}",
                    t.describe()
                ),
            ));
        }
    }
    let input = input.ok_or_else(|| syntax(line, "example has no `in:`"))?;
    let output = output.ok_or_else(|| syntax(line, "example has no `out:`"))?;
    Ok(Example {
        line,
        note,
        input,
        output,
    })
}

/// A literal is a value with no operators: scalars, `None`, named values,
/// `Ok`/`Err`, lists, records (with `...` only where allowed) and, at the
/// top of an `in:`, a tuple.
pub fn check_literal(
    e: &Expr,
    allow_rest: bool,
    kind: ErrorKind,
    line: usize,
    what: &str,
) -> Result<(), Error> {
    fn inner(e: &Expr, allow_rest: bool, top: bool) -> bool {
        match e {
            Expr::Int(_) | Expr::Float(_) | Expr::Text(_) | Expr::None | Expr::Name(..) => true,
            Expr::Ok(x) | Expr::Err(x) => inner(x, allow_rest, false),
            Expr::List(xs) => xs.iter().all(|x| inner(x, allow_rest, false)),
            Expr::Record(fields, rest) => {
                (!*rest || allow_rest) && fields.iter().all(|(_, x)| inner(x, allow_rest, false))
            }
            Expr::Tuple(xs) => top && !allow_rest && xs.iter().all(|x| inner(x, allow_rest, false)),
            _ => false,
        }
    }
    if inner(e, allow_rest, true) {
        Ok(())
    } else {
        Err(Error::new(
            kind,
            line,
            format!("{what} must be a full literal (no operators, no `...` in `in:`)"),
        ))
    }
}

/// Token cursor for types and parameter lists.
struct Cursor<'a> {
    toks: &'a [Token],
    pos: usize,
    line: usize,
}

impl<'a> Cursor<'a> {
    fn new(toks: &'a [Token], pos: usize, line: usize) -> Self {
        Cursor { toks, pos, line }
    }
    fn peek(&self) -> Option<&'a Token> {
        self.toks.get(self.pos)
    }
    fn bump(&mut self) -> Option<&'a Token> {
        let t = self.toks.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn err(&self, msg: impl Into<String>) -> Error {
        let line = self.peek().map(|t| t.line).unwrap_or(self.line);
        syntax(line, msg).spanning(self.peek().map(|t| (t.col, t.end)))
    }
    fn expect_sym(&mut self, s: &str) -> Result<(), Error> {
        match self.peek() {
            Some(t) if t.is_sym(s) => {
                self.pos += 1;
                Ok(())
            }
            Some(t) => Err(self.err(format!("expected `{s}`, found {}", t.describe()))),
            None => Err(self.err(format!("expected `{s}`"))),
        }
    }
    fn expect_name(&mut self, what: &str) -> Result<String, Error> {
        match self.peek() {
            Some(t) => match t.ident() {
                Some(n) if !is_reserved(n) => {
                    self.pos += 1;
                    Ok(n.to_string())
                }
                _ => Err(self.err(format!("expected {what}, found {}", t.describe()))),
            },
            None => Err(self.err(format!("expected {what}"))),
        }
    }
    fn expect_end(&self) -> Result<(), Error> {
        match self.peek() {
            None => Ok(()),
            Some(t) => Err(self.err(format!("unexpected {}", t.describe()))),
        }
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Type)>, Error> {
        let mut params = Vec::new();
        while self.peek().is_some() {
            let name = self.expect_name("a parameter name")?;
            self.expect_sym(":")?;
            let ty = self.parse_type()?;
            params.push((name, ty));
            match self.peek() {
                Some(t) if t.is_sym(",") => {
                    self.pos += 1;
                }
                Some(t) => return Err(self.err(format!("expected `,`, found {}", t.describe()))),
                None => {}
            }
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, Error> {
        let t = self.bump().ok_or_else(|| self.err("expected a type"))?;
        let mut base = match &t.tok {
            Tok::Ident(n) => match n.as_str() {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Text" => Type::Text,
                "Bool" => Type::Bool,
                "Id" => Type::Id,
                "Time" => Type::Time,
                "UUID" => Type::Uuid,
                "Option" => {
                    self.expect_sym("<")?;
                    let inner = self.parse_type()?;
                    self.expect_sym(">")?;
                    Type::Option(Box::new(inner))
                }
                "Result" => {
                    self.expect_sym("<")?;
                    let ok = self.parse_type()?;
                    self.expect_sym(",")?;
                    let err = self.parse_type()?;
                    self.expect_sym(">")?;
                    Type::Result(Box::new(ok), Box::new(err))
                }
                n if is_reserved(n) => {
                    return Err(syntax(t.line, format!("expected a type, found `{n}`")));
                }
                n => Type::Named(n.to_string()),
            },
            Tok::Sym("{") => {
                let mut spreads = Vec::new();
                let mut fields = Vec::new();
                loop {
                    match self.peek() {
                        Some(t) if t.is_sym("}") => {
                            self.pos += 1;
                            break;
                        }
                        Some(t) if t.is_sym("...") => {
                            self.pos += 1;
                            spreads.push(self.expect_name("a type name after `...`")?);
                        }
                        Some(_) => {
                            let name = self.expect_name("a field name")?;
                            self.expect_sym(":")?;
                            let ty = self.parse_type()?;
                            fields.push((name, ty));
                        }
                        None => return Err(self.err("unclosed `{` in a type")),
                    }
                    match self.peek() {
                        Some(t) if t.is_sym(",") => {
                            self.pos += 1;
                        }
                        Some(t) if t.is_sym("}") => {}
                        Some(t) => {
                            return Err(
                                self.err(format!("expected `,` or `}}`, found {}", t.describe()))
                            );
                        }
                        None => return Err(self.err("unclosed `{` in a type")),
                    }
                }
                Type::Record { spreads, fields }
            }
            _ => {
                return Err(syntax(
                    t.line,
                    format!("expected a type, found {}", t.describe()),
                ));
            }
        };
        while self.peek().is_some_and(|t| t.is_sym("[")) {
            self.pos += 1;
            let mut refs = Vec::new();
            loop {
                match self.peek() {
                    Some(t) if t.is_sym("]") => {
                        self.pos += 1;
                        break;
                    }
                    Some(t) if matches!(t.tok, Tok::Int(_)) => {
                        let lo = match self.bump().map(|t| &t.tok) {
                            Some(Tok::Int(i)) => *i,
                            _ => unreachable!(),
                        };
                        self.expect_sym("..")?;
                        let hi = match self.bump().map(|t| &t.tok) {
                            Some(Tok::Int(i)) => *i,
                            _ => return Err(self.err("expected the upper bound of a length range")),
                        };
                        if lo < 0 || hi < lo {
                            return Err(self.err("bad length range"));
                        }
                        refs.push(Refinement::Length {
                            lo: lo as u64,
                            hi: hi as u64,
                        });
                    }
                    Some(t) if t.is_ident("unique") => {
                        self.pos += 1;
                        let key = self.expect_name("a field name after `unique`")?;
                        refs.push(Refinement::UniqueBy(key));
                    }
                    Some(t) => {
                        return Err(self.err(format!(
                            "expected a list refinement (`0..3` or `unique f`), found {}",
                            t.describe()
                        )));
                    }
                    None => return Err(self.err("unclosed `[` in a type")),
                }
                match self.peek() {
                    Some(t) if t.is_sym(",") => {
                        self.pos += 1;
                    }
                    Some(t) if t.is_sym("]") => {}
                    Some(t) => {
                        return Err(
                            self.err(format!("expected `,` or `]`, found {}", t.describe()))
                        );
                    }
                    None => return Err(self.err("unclosed `[` in a type")),
                }
            }
            base = Type::List(Box::new(base), refs);
        }
        Ok(base)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Property,
    Literal,
}

struct ExprParser<'a> {
    toks: &'a [Token],
    pos: usize,
    ctx: Ctx,
    block: &'a [Item],
    block_used: bool,
    line: usize,
    q_done: bool,
}

fn parse_expr(toks: &[Token], block: &[Item], ctx: Ctx, line: usize) -> Result<Expr, Error> {
    let mut p = ExprParser {
        toks,
        pos: 0,
        ctx,
        block,
        block_used: false,
        line,
        q_done: false,
    };
    let e = p.parse_iff()?;
    if let Some(t) = p.peek() {
        if p.q_done {
            return Err(Error::new(
                ErrorKind::QuantifierMidLine,
                t.line,
                "a quantifier must be the last operand on its line",
            ));
        }
        return Err(syntax(t.line, format!("unexpected {}", t.describe())));
    }
    if !block.is_empty() && !p.block_used {
        return Err(Error::new(
            ErrorKind::UnexpectedIndent,
            block[0].line.line,
            "indented block under a line that is not a quantifier head",
        ));
    }
    Ok(e)
}

impl<'a> ExprParser<'a> {
    fn peek(&self) -> Option<&'a Token> {
        self.toks.get(self.pos)
    }
    fn peek_at(&self, n: usize) -> Option<&'a Token> {
        self.toks.get(self.pos + n)
    }
    fn bump(&mut self) -> Option<&'a Token> {
        let t = self.toks.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn cur_line(&self) -> usize {
        self.peek()
            .map(|t| t.line)
            .or_else(|| self.toks.last().map(|t| t.line))
            .unwrap_or(self.line)
    }
    fn err(&self, msg: impl Into<String>) -> Error {
        syntax(self.cur_line(), msg).spanning(self.peek().map(|t| (t.col, t.end)))
    }
    /// The columns tokens `from..to` cover, when they all sit on `line`.
    fn span(&self, from: usize, to: usize, line: usize) -> Option<(usize, usize)> {
        let toks = self.toks.get(from..to)?;
        let (first, last) = (toks.first()?, toks.last()?);
        (first.line == line && last.line == line).then_some((first.col, last.end))
    }
    fn peek_sym(&self, s: &str) -> bool {
        self.peek().is_some_and(|t| t.is_sym(s))
    }
    fn peek_ident(&self, s: &str) -> bool {
        self.peek().is_some_and(|t| t.is_ident(s))
    }
    fn expect_sym(&mut self, s: &str) -> Result<(), Error> {
        if self.peek_sym(s) {
            self.pos += 1;
            Ok(())
        } else {
            match self.peek() {
                Some(t) => Err(self.err(format!("expected `{s}`, found {}", t.describe()))),
                None => Err(self.err(format!("expected `{s}`"))),
            }
        }
    }
    /// An operator after a complete quantifier means the quantifier was not
    /// the last operand on the line.
    fn operator_ok(&self) -> Result<(), Error> {
        if self.q_done {
            Err(Error::new(
                ErrorKind::QuantifierMidLine,
                self.cur_line(),
                "a quantifier must be the last operand on its line",
            ))
        } else {
            Ok(())
        }
    }

    fn parse_iff(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_implies()?;
        while self.peek_sym("<->") {
            self.operator_ok()?;
            self.bump();
            let right = self.parse_implies()?;
            left = Expr::Binary(BinOp::Iff, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_implies(&mut self) -> Result<Expr, Error> {
        let left = self.parse_or()?;
        if self.peek_sym("->") {
            self.operator_ok()?;
            self.bump();
            let right = self.parse_implies()?;
            return Ok(Expr::Binary(
                BinOp::Implies,
                Box::new(left),
                Box::new(right),
            ));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_and()?;
        while self.peek_ident("or") {
            self.operator_ok()?;
            self.bump();
            let right = self.parse_and()?;
            left = Expr::Binary(BinOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_not()?;
        while self.peek_ident("and") {
            self.operator_ok()?;
            self.bump();
            let right = self.parse_not()?;
            left = Expr::Binary(BinOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, Error> {
        if self.peek_ident("not") && !self.peek_at(1).is_some_and(|t| t.is_ident("in")) {
            self.bump();
            let inner = self.parse_not()?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_cmp()
    }

    fn parse_cmp(&mut self) -> Result<Expr, Error> {
        if self.peek_ident("unique") {
            self.bump();
            let e = self.parse_postfix()?;
            return Ok(Expr::Unique(Box::new(e)));
        }
        let left = self.parse_postfix()?;
        let Some(t) = self.peek() else {
            return Ok(left);
        };
        let op = match &t.tok {
            Tok::Sym("==") => Some(BinOp::Eq),
            Tok::Sym("!=") => Some(BinOp::Ne),
            Tok::Sym("<") => Some(BinOp::Lt),
            Tok::Sym("<=") => Some(BinOp::Le),
            Tok::Sym(">") => Some(BinOp::Gt),
            Tok::Sym(">=") => Some(BinOp::Ge),
            Tok::Sym("=") => {
                return Err(Error::new(
                    ErrorKind::AssignmentInProperty,
                    t.line,
                    "`=` in a property; use `==`",
                ));
            }
            Tok::Ident(w) if w == "in" => Some(BinOp::In),
            Tok::Ident(w) if w == "not" && self.peek_at(1).is_some_and(|t| t.is_ident("in")) => {
                self.bump();
                Some(BinOp::NotIn)
            }
            _ => None,
        };
        if let Some(op) = op {
            self.operator_ok()?;
            self.bump();
            let right = self.parse_postfix()?;
            return Ok(Expr::Binary(op, Box::new(left), Box::new(right)));
        }
        let word = t.ident().unwrap_or("");
        match word {
            "before" => {
                self.operator_ok()?;
                self.bump();
                let b = self.parse_postfix()?;
                if !self.peek_ident("in") {
                    return Err(self.err("expected `in` after `a before b`"));
                }
                self.bump();
                let list = self.parse_postfix()?;
                Ok(Expr::Before {
                    a: Box::new(left),
                    b: Box::new(b),
                    list: Box::new(list),
                })
            }
            "same_order_as" | "same_elements_as" | "drawn_from" => {
                self.operator_ok()?;
                self.bump();
                let rel = match word {
                    "same_order_as" => RelOp::SameOrderAs,
                    "same_elements_as" => RelOp::SameElementsAs,
                    _ => RelOp::DrawnFrom,
                };
                let right = self.parse_postfix()?;
                Ok(Expr::Relation(rel, Box::new(left), Box::new(right)))
            }
            "sorted_by" => {
                self.operator_ok()?;
                self.bump();
                self.expect_sym("(")?;
                let mut keys = Vec::new();
                loop {
                    let key = match self.bump() {
                        Some(t) => match t.ident() {
                            Some(k) if !is_reserved(k) => k.to_string(),
                            _ => return Err(syntax(t.line, "expected a sort key")),
                        },
                        None => return Err(self.err("expected a sort key")),
                    };
                    let dir = match self.bump() {
                        Some(t) if t.is_ident("asc") => Dir::Asc,
                        Some(t) if t.is_ident("desc") => Dir::Desc,
                        _ => return Err(self.err("expected `asc` or `desc` after a sort key")),
                    };
                    keys.push((key, dir));
                    if self.peek_sym(",") {
                        self.bump();
                        continue;
                    }
                    self.expect_sym(")")?;
                    break;
                }
                Ok(Expr::SortedBy(Box::new(left), keys))
            }
            "unordered" => {
                self.operator_ok()?;
                self.bump();
                Ok(Expr::Unordered(Box::new(left)))
            }
            _ => Ok(left),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, Error> {
        let mut e = self.parse_primary()?;
        while self.peek_sym(".") {
            let dot = self.bump().unwrap();
            let name = match self.bump() {
                Some(t) => match t.ident() {
                    Some(n) if !is_reserved(n) => n.to_string(),
                    _ => return Err(syntax(t.line, "expected a field name after `.`")),
                },
                None => return Err(syntax(dot.line, "expected a field name after `.`")),
            };
            e = Expr::Field(Box::new(e), name, dot.line);
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> Result<Expr, Error> {
        let t = self
            .bump()
            .ok_or_else(|| self.err("expected an expression"))?;
        match &t.tok {
            Tok::Int(i) => Ok(Expr::Int(*i)),
            Tok::Float(f) => Ok(Expr::Float(*f)),
            Tok::Str(s) => Ok(Expr::Text(s.clone())),
            Tok::Sym("(") => {
                if self.ctx == Ctx::Literal {
                    let mut items = Vec::new();
                    let mut saw_comma = false;
                    loop {
                        if self.peek_sym(")") {
                            self.bump();
                            break;
                        }
                        items.push(self.parse_iff()?);
                        if self.peek_sym(",") {
                            self.bump();
                            saw_comma = true;
                        } else {
                            self.expect_sym(")")?;
                            break;
                        }
                    }
                    if items.len() == 1 && !saw_comma {
                        return Ok(items.pop().unwrap());
                    }
                    return Ok(Expr::Tuple(items));
                }
                let saved = self.q_done;
                let e = self.parse_iff()?;
                self.expect_sym(")")?;
                // A quantifier closed inside parentheses is still a quantifier
                // for the "last operand" rule.
                self.q_done = saved || self.q_done;
                Ok(e)
            }
            Tok::Sym("[") => {
                let mut items = Vec::new();
                loop {
                    if self.peek_sym("]") {
                        self.bump();
                        break;
                    }
                    items.push(self.parse_iff()?);
                    if self.peek_sym(",") {
                        self.bump();
                    } else {
                        self.expect_sym("]")?;
                        break;
                    }
                }
                Ok(Expr::List(items))
            }
            Tok::Sym("{") => {
                let mut fields = Vec::new();
                let mut rest = false;
                loop {
                    if self.peek_sym("}") {
                        self.bump();
                        break;
                    }
                    if self.peek_sym("...") {
                        self.bump();
                        rest = true;
                    } else {
                        let name = match self.bump() {
                            Some(t) => match t.ident() {
                                Some(n) if !is_reserved(n) => n.to_string(),
                                _ => return Err(syntax(t.line, "expected a field name")),
                            },
                            None => return Err(self.err("unclosed `{`")),
                        };
                        self.expect_sym(":")?;
                        let v = self.parse_iff()?;
                        fields.push((name, v));
                    }
                    if self.peek_sym(",") {
                        self.bump();
                    } else {
                        self.expect_sym("}")?;
                        break;
                    }
                }
                Ok(Expr::Record(fields, rest))
            }
            Tok::Sym(s) => Err(syntax(
                t.line,
                format!("expected an expression, found `{s}`"),
            )),
            Tok::Ident(name) => match name.as_str() {
                "None" => Ok(Expr::None),
                "Ok" | "Err" if self.peek_sym("(") => {
                    if self.ctx == Ctx::Property {
                        return Err(Error::new(
                            ErrorKind::PatternAsValue,
                            t.line,
                            format!("`{name}(...)` is a pattern; a value is expected here"),
                        ));
                    }
                    self.bump();
                    let inner = self.parse_iff()?;
                    self.expect_sym(")")?;
                    Ok(if name == "Ok" {
                        Expr::Ok(Box::new(inner))
                    } else {
                        Expr::Err(Box::new(inner))
                    })
                }
                "all" | "any" | "none" | "exactly_one" => {
                    if self.ctx == Ctx::Literal {
                        return Err(syntax(t.line, "a quantifier is not a literal"));
                    }
                    self.parse_quantifier(t)
                }
                n if is_reserved(n) => Err(syntax(
                    t.line,
                    format!("expected an expression, found `{n}`"),
                )),
                n => {
                    if self.peek_sym("(") {
                        self.bump();
                        let mut args = Vec::new();
                        loop {
                            if self.peek_sym(")") {
                                self.bump();
                                break;
                            }
                            args.push(self.parse_iff()?);
                            if self.peek_sym(",") {
                                self.bump();
                            } else {
                                self.expect_sym(")")?;
                                break;
                            }
                        }
                        Ok(Expr::Call(n.to_string(), args, t.line))
                    } else {
                        Ok(Expr::Name(n.to_string(), t.line))
                    }
                }
            },
        }
    }

    fn parse_binder(&mut self) -> Result<Binder, Error> {
        let t = self.bump().ok_or_else(|| self.err("expected a binder"))?;
        let name = t
            .ident()
            .ok_or_else(|| syntax(t.line, format!("expected a binder, found {}", t.describe())))?;
        match name {
            "Ok" | "Err" if self.peek_sym("(") => {
                self.bump();
                let inner = self
                    .bump()
                    .ok_or_else(|| self.err("expected a name or `_`"))?;
                let bound = match inner.ident() {
                    Some("_") => None,
                    Some(n) if !is_reserved(n) => Some(n.to_string()),
                    _ => {
                        return Err(syntax(
                            inner.line,
                            format!(
                                "expected a name or `_` in the pattern, found {}",
                                inner.describe()
                            ),
                        ));
                    }
                };
                self.expect_sym(")")?;
                Ok(Binder {
                    side: Some(if name == "Ok" { Side::Ok } else { Side::Err }),
                    name: bound,
                })
            }
            "_" => Ok(Binder {
                side: None,
                name: None,
            }),
            n if is_reserved(n) => Err(syntax(t.line, format!("expected a binder, found `{n}`"))),
            n => Ok(Binder {
                side: None,
                name: Some(n.to_string()),
            }),
        }
    }

    fn parse_quantifier(&mut self, head: &Token) -> Result<Expr, Error> {
        let line = head.line;
        let kind = match head.ident().unwrap() {
            "all" => QKind::All,
            "any" => QKind::Any,
            "none" => QKind::None,
            _ => QKind::ExactlyOne,
        };
        let binder_at = self.pos;
        let binder = self.parse_binder()?;
        let binder_end = self.pos;
        let leftover_at = self.pos;
        let leftover = if self.peek_ident("leftover") {
            self.bump();
            true
        } else {
            false
        };
        if !self.peek_ident("in") {
            return Err(self.err("expected `in` in the quantifier head"));
        }
        self.bump();
        let domain = self.parse_postfix()?;
        let domain_end = self.pos;
        // A name straight after the domain is a second domain, not the
        // bodyless form: say so before the binder looks unused.
        if let Some(Tok::Ident(n)) = self.peek().map(|t| &t.tok) {
            if !is_reserved(n) {
                let second_at = self.pos;
                let second = self.parse_postfix()?;
                return Err(syntax(
                    line,
                    format!(
                        "two lists after `in`: `{}` and `{}`; a quantifier ranges over one",
                        crate::render::expr(&domain),
                        crate::render::expr(&second)
                    ),
                )
                .spanning(self.span(second_at, self.pos, line)));
            }
        }
        if leftover && matches!(&domain, Expr::Name(n, _) if n == "out") {
            return Err(Error::new(
                ErrorKind::EmptyLeftover,
                line,
                "`leftover in out` ranges over nothing, since `out` takes all of its own elements; range over the input `out` is drawn from",
            )
            .spanning(self.span(leftover_at, domain_end, line)));
        }
        let where_ = if self.peek_ident("where") {
            self.bump();
            Some(self.parse_iff()?)
        } else {
            None
        };
        let mut body = None;
        let mut else_ = None;
        if self.peek_sym(":") {
            self.bump();
            if self.peek().is_none() {
                if self.block.is_empty() {
                    let head_at = self.toks.iter().position(|t| std::ptr::eq(t, head));
                    return Err(syntax(line, "quantifier ends in `:` but has no body")
                        .spanning(head_at.and_then(|at| self.span(at, self.pos, line))));
                }
                if self.block.len() > 1 {
                    return Err(Error::new(
                        ErrorKind::TwoBodies,
                        self.block[1].line.line,
                        "two bodies under one quantifier",
                    ));
                }
                let item = &self.block[0];
                body = Some(parse_expr(
                    &item.line.tokens,
                    &item.children,
                    self.ctx,
                    item.line.line,
                )?);
                self.block_used = true;
            } else {
                body = Some(self.parse_iff()?);
                if !self.block.is_empty() && !self.block_used {
                    return Err(Error::new(
                        ErrorKind::TwoBodies,
                        self.block[0].line.line,
                        "two bodies under one quantifier (inline and indented)",
                    ));
                }
                if self.peek_ident("else") {
                    let t = self.bump().unwrap();
                    if where_.is_none() {
                        return Err(Error::new(
                            ErrorKind::ElseWithoutWhere,
                            t.line,
                            "`else` needs a `where` on the quantifier it attaches to",
                        ));
                    }
                    else_ = Some(self.parse_iff()?);
                }
            }
        } else {
            if where_.is_some() {
                return Err(Error::new(
                    ErrorKind::WhereWithoutBody,
                    line,
                    "`where` needs a body; the bodyless form counts the whole domain",
                ));
            }
            if self.peek_ident("else") {
                return Err(Error::new(
                    ErrorKind::ElseWithoutWhere,
                    self.cur_line(),
                    "`else` needs a `where` on the quantifier it attaches to",
                ));
            }
        }
        if let Some(name) = &binder.name {
            let used = where_.as_ref().is_some_and(|w| w.mentions(name))
                || body.as_ref().is_some_and(|b| b.mentions(name))
                || else_.as_ref().is_some_and(|e| e.mentions(name));
            if !used {
                return Err(Error::new(
                    ErrorKind::UnusedBinder,
                    line,
                    format!("binder `{name}` is never used"),
                )
                .spanning(self.span(binder_at, binder_end, line)));
            }
        }
        self.q_done = true;
        Ok(Expr::Quantifier(Box::new(Quantifier {
            kind,
            binder,
            leftover,
            domain,
            where_,
            body,
            else_,
            line,
        })))
    }
}
