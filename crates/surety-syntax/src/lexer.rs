//! Lines and tokens.
//!
//! Indentation follows Python's rule: the leading whitespace of a line is
//! compared as a raw prefix, with no tab width, and line structure is
//! suspended while any bracket is open. Comment lines (`//`), sentence lines
//! (`#`) and blank lines never open or close a block.

use crate::ast::{Pin, PinKind};
use crate::error::{Error, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Sym(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

impl Token {
    pub fn is_sym(&self, s: &str) -> bool {
        matches!(&self.tok, Tok::Sym(x) if *x == s)
    }
    pub fn is_ident(&self, s: &str) -> bool {
        matches!(&self.tok, Tok::Ident(x) if x == s)
    }
    pub fn ident(&self) -> Option<&str> {
        match &self.tok {
            Tok::Ident(x) => Some(x),
            _ => None,
        }
    }
    pub fn describe(&self) -> String {
        match &self.tok {
            Tok::Ident(x) => format!("`{x}`"),
            Tok::Int(i) => format!("`{i}`"),
            Tok::Float(f) => format!("`{f}`"),
            Tok::Str(s) => format!("\"{s}\""),
            Tok::Sym(s) => format!("`{s}`"),
        }
    }
}

/// One logical line: a physical line plus any continuation lines that an
/// open bracket pulled in.
#[derive(Debug, Clone)]
pub struct Line {
    pub indent: String,
    pub line: usize,
    pub tokens: Vec<Token>,
    /// The source text of the logical line, continuation lines joined by
    /// single spaces.
    pub text: String,
    /// The `#` sentence directly above this line, if any.
    pub sentence: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Lexed {
    pub header: Vec<(usize, String)>,
    pub lines: Vec<Line>,
    pub pins: Vec<Pin>,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Whether a column-0 line begins a declaration: `type X =`, `name: T = …`,
/// `name = …`, `name(…) = …`, or `name(…) -> …`. Authoring notes such as
/// `Thoughts:` are not declarations: a named value needs its `=`.
pub fn is_decl_start(rest: &str) -> bool {
    let rest = rest.trim_end();
    if let Some(after) = rest.strip_prefix("type") {
        if after.starts_with(char::is_whitespace) {
            return true;
        }
    }
    let mut chars = rest.char_indices();
    match chars.next() {
        Some((_, c)) if is_ident_start(c) => {}
        _ => return false,
    }
    let mut end = rest.len();
    for (i, c) in chars {
        if !is_ident_char(c) {
            end = i;
            break;
        }
    }
    let after = rest[end..].trim_start();
    if let Some(inner) = after.strip_prefix('(') {
        let mut depth = 1;
        let mut close = None;
        for (i, c) in inner.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(close) = close else { return false };
        let tail = inner[close + 1..].trim_start();
        return tail.starts_with("->") || (tail.starts_with('=') && !tail.starts_with("=="));
    }
    if let Some(tail) = after.strip_prefix(':') {
        return match tail.find('=') {
            Some(i) => !tail[i..].starts_with("==") && !tail[..i].ends_with(['!', '<', '>']),
            None => false,
        };
    }
    after.starts_with('=') && !after.starts_with("==")
}

struct Scanner<'a> {
    src: &'a str,
    pos: usize,
    line: usize,
}

impl Scanner<'_> {
    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }
    fn peek2(&self) -> Option<char> {
        let mut it = self.src[self.pos..].chars();
        it.next();
        it.next()
    }
    fn peek3(&self) -> Option<char> {
        let mut it = self.src[self.pos..].chars();
        it.next();
        it.next();
        it.next()
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn err(&self, msg: impl Into<String>) -> Error {
        Error::new(ErrorKind::Syntax, self.line, msg)
    }

    /// Tokenize one physical line's remainder, updating the bracket depth.
    fn run(&mut self, depth: &mut i32, out: &mut Vec<Token>) -> Result<(), Error> {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.bump();
                continue;
            }
            if c == '"' {
                self.bump();
                let start = self.pos;
                loop {
                    match self.peek() {
                        None => return Err(self.err("unterminated string")),
                        Some('"') => break,
                        Some(_) => {
                            self.bump();
                        }
                    }
                }
                let s = self.src[start..self.pos].to_string();
                self.bump();
                out.push(Token {
                    tok: Tok::Str(s),
                    line: self.line,
                });
                continue;
            }
            if c.is_ascii_digit() {
                let start = self.pos;
                while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                    self.bump();
                }
                let mut is_float = false;
                if self.peek() == Some('.') && matches!(self.peek2(), Some(d) if d.is_ascii_digit())
                {
                    is_float = true;
                    self.bump();
                    while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                        self.bump();
                    }
                }
                let text = &self.src[start..self.pos];
                let tok = if is_float {
                    Tok::Float(text.parse().map_err(|_| self.err("bad number"))?)
                } else {
                    Tok::Int(text.parse().map_err(|_| self.err("bad number"))?)
                };
                out.push(Token {
                    tok,
                    line: self.line,
                });
                continue;
            }
            if is_ident_start(c) {
                let start = self.pos;
                while matches!(self.peek(), Some(d) if is_ident_char(d)) {
                    self.bump();
                }
                out.push(Token {
                    tok: Tok::Ident(self.src[start..self.pos].to_string()),
                    line: self.line,
                });
                continue;
            }
            let c2 = self.peek2();
            let c3 = self.peek3();
            let (sym, len) = match (c, c2, c3) {
                ('<', Some('-'), Some('>')) => ("<->", 3),
                ('.', Some('.'), Some('.')) => ("...", 3),
                ('-', Some('>'), _) => ("->", 2),
                ('<', Some('='), _) => ("<=", 2),
                ('>', Some('='), _) => (">=", 2),
                ('=', Some('='), _) => ("==", 2),
                ('!', Some('='), _) => ("!=", 2),
                ('.', Some('.'), _) => ("..", 2),
                ('{', _, _) => ("{", 1),
                ('}', _, _) => ("}", 1),
                ('[', _, _) => ("[", 1),
                (']', _, _) => ("]", 1),
                ('(', _, _) => ("(", 1),
                (')', _, _) => (")", 1),
                ('<', _, _) => ("<", 1),
                ('>', _, _) => (">", 1),
                (',', _, _) => (",", 1),
                (':', _, _) => (":", 1),
                ('=', _, _) => ("=", 1),
                ('.', _, _) => (".", 1),
                ('/', Some('/'), _) => {
                    return Err(self.err("`//` comments must start a line"));
                }
                _ => return Err(self.err(format!("unexpected character `{c}`"))),
            };
            for _ in 0..len {
                self.bump();
            }
            match sym {
                "{" | "[" | "(" => *depth += 1,
                "}" | "]" | ")" => {
                    *depth -= 1;
                    if *depth < 0 {
                        return Err(self.err(format!("unmatched `{sym}`")));
                    }
                }
                _ => {}
            }
            out.push(Token {
                tok: Tok::Sym(sym),
                line: self.line,
            });
        }
        Ok(())
    }
}

fn leading_ws(s: &str) -> &str {
    let end = s
        .char_indices()
        .find(|(_, c)| !(*c == ' ' || *c == '\t'))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    &s[..end]
}

/// Split source text into logical lines.
pub fn lex(src: &str) -> Result<Lexed, Error> {
    let mut lexed = Lexed::default();
    let mut depth: i32 = 0;
    let mut current: Option<Line> = None;
    let mut pending_sentence: Option<String> = None;
    let mut seen_decl = false;

    for (idx, raw) in src.lines().enumerate() {
        let line_no = idx + 1;
        if depth == 0 {
            let indent = leading_ws(raw);
            let rest = &raw[indent.len()..];
            if rest.trim().is_empty() {
                continue;
            }
            if let Some(c) = rest.strip_prefix("//") {
                lexed.pins.push(Pin {
                    line: line_no,
                    kind: PinKind::Comment,
                    text: c.trim().to_string(),
                });
                continue;
            }
            if let Some(s) = rest.strip_prefix('#') {
                let text = s.trim().to_string();
                lexed.pins.push(Pin {
                    line: line_no,
                    kind: PinKind::Sentence,
                    text: text.clone(),
                });
                pending_sentence = Some(text);
                continue;
            }
            if indent.is_empty() && !seen_decl && !is_decl_start(rest) {
                lexed.header.push((line_no, rest.trim_end().to_string()));
                continue;
            }
            if indent.is_empty() && is_decl_start(rest) {
                seen_decl = true;
            }
            let mut tokens = Vec::new();
            let mut sc = Scanner {
                src: rest,
                pos: 0,
                line: line_no,
            };
            sc.run(&mut depth, &mut tokens)?;
            let line = Line {
                indent: indent.to_string(),
                line: line_no,
                tokens,
                text: rest.trim_end().to_string(),
                sentence: pending_sentence.take(),
            };
            if depth == 0 {
                lexed.lines.push(line);
            } else {
                current = Some(line);
            }
        } else {
            let rest = raw.trim();
            if rest.is_empty() {
                continue;
            }
            if let Some(c) = rest.strip_prefix("//") {
                lexed.pins.push(Pin {
                    line: line_no,
                    kind: PinKind::Comment,
                    text: c.trim().to_string(),
                });
                continue;
            }
            if let Some(s) = rest.strip_prefix('#') {
                lexed.pins.push(Pin {
                    line: line_no,
                    kind: PinKind::Sentence,
                    text: s.trim().to_string(),
                });
                continue;
            }
            let cur = current.as_mut().expect("continuation without a line");
            cur.text.push(' ');
            cur.text.push_str(rest);
            let mut sc = Scanner {
                src: rest,
                pos: 0,
                line: line_no,
            };
            sc.run(&mut depth, &mut cur.tokens)?;
            if depth == 0 {
                lexed.lines.push(current.take().unwrap());
            }
        }
    }
    if let Some(cur) = current {
        return Err(Error::new(
            ErrorKind::Syntax,
            cur.line,
            "bracket opened here is never closed",
        ));
    }
    Ok(lexed)
}
