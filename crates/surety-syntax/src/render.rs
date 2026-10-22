//! The syntax tree printed back as Surety: the inverse of the parser up to
//! the hash. Types, named values, abbreviations, the signature, the note,
//! the function's abbreviations, each property under its sentence, the
//! accept lines and the examples, in the corpus's layout; comments at the
//! line they were pinned to. With fresh names, binders are renamed in order
//! of appearance, which must not change the hash.

use std::fmt::Write;

use crate::ast::*;

struct Printer {
    fresh: bool,
    counter: usize,
    /// Binder renames in scope, innermost last.
    scope: Vec<(String, String)>,
}

/// Binding strength, higher binds tighter; a child weaker than its parent
/// is parenthesized, and so is a right child of the same strength, so the
/// tree the parser builds is the tree that was printed.
fn strength(e: &Expr) -> u8 {
    match e {
        Expr::Quantifier(_) => 0,
        Expr::Binary(op, _, _) => match op {
            BinOp::Iff => 1,
            BinOp::Implies => 2,
            BinOp::Or => 3,
            BinOp::And => 4,
            _ => 6,
        },
        Expr::Not(_) => 5,
        Expr::Relation(..) | Expr::Before { .. } | Expr::Unordered(_) | Expr::Unique(_) => 6,
        Expr::SortedBy(..) => 6,
        _ => 9,
    }
}

impl Printer {
    fn name(&self, n: &str) -> String {
        self.scope
            .iter()
            .rev()
            .find(|(old, _)| old == n)
            .map(|(_, new)| new.clone())
            .unwrap_or_else(|| n.to_string())
    }

    fn operand(&mut self, e: &Expr, parent: u8, right: bool) -> String {
        let s = strength(e);
        let text = self.expr(e);
        let needs = s < parent || (s == parent && right && s < 9) || (s == 0 && !right);
        if needs { format!("({text})") } else { text }
    }

    fn primary(&mut self, e: &Expr) -> String {
        let text = self.expr(e);
        if strength(e) < 9 {
            format!("({text})")
        } else {
            text
        }
    }

    fn expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Int(i) => i.to_string(),
            Expr::Float(f) => format!("{f:?}"),
            Expr::Text(s) => format!("\"{s}\""),
            Expr::None => "None".to_string(),
            Expr::Name(n, _) => self.name(n),
            Expr::Field(base, f, _) => format!("{}.{f}", self.primary(base)),
            Expr::Call(n, args, _) => {
                let args: Vec<String> = args.iter().map(|a| self.expr(a)).collect();
                format!("{n}({})", args.join(", "))
            }
            Expr::Ok(x) => format!("Ok({})", self.expr(x)),
            Expr::Err(x) => format!("Err({})", self.expr(x)),
            Expr::List(xs) => {
                let xs: Vec<String> = xs.iter().map(|x| self.expr(x)).collect();
                format!("[{}]", xs.join(", "))
            }
            Expr::Record(fields, rest) => {
                let mut parts: Vec<String> = fields
                    .iter()
                    .map(|(n, v)| format!("{n}: {}", self.expr(v)))
                    .collect();
                if *rest {
                    parts.push("...".to_string());
                }
                format!("{{{}}}", parts.join(", "))
            }
            Expr::Tuple(xs) => {
                let xs: Vec<String> = xs.iter().map(|x| self.expr(x)).collect();
                format!("({})", xs.join(", "))
            }
            Expr::Not(x) => format!("not {}", self.operand(x, 5, true)),
            Expr::Binary(op, a, b) => {
                let p = strength(e);
                format!(
                    "{} {} {}",
                    self.operand(a, p, false),
                    op.symbol(),
                    self.operand(b, p, true)
                )
            }
            Expr::Before { a, b, list } => format!(
                "{} before {} in {}",
                self.primary(a),
                self.primary(b),
                self.primary(list)
            ),
            Expr::Relation(op, a, b) => {
                format!("{} {} {}", self.primary(a), op.symbol(), self.primary(b))
            }
            Expr::SortedBy(x, keys) => {
                let keys: Vec<String> = keys
                    .iter()
                    .map(|(k, d)| {
                        format!(
                            "{k} {}",
                            match d {
                                Dir::Asc => "asc",
                                Dir::Desc => "desc",
                            }
                        )
                    })
                    .collect();
                format!("{} sorted_by ({})", self.primary(x), keys.join(", "))
            }
            Expr::Unordered(x) => format!("{} unordered", self.primary(x)),
            Expr::Unique(x) => format!("unique {}", self.primary(x)),
            Expr::Quantifier(q) => self.quantifier(q),
        }
    }

    fn quantifier(&mut self, q: &Quantifier) -> String {
        let domain = self.expr(&q.domain);
        let pushed = match &q.binder.name {
            Some(n) if self.fresh => {
                let new = format!("b{}", self.counter);
                self.counter += 1;
                self.scope.push((n.clone(), new));
                true
            }
            _ => false,
        };
        let inner = match &q.binder.name {
            Some(n) => self.name(n),
            None => "_".to_string(),
        };
        let binder = match q.binder.side {
            Some(Side::Ok) => format!("Ok({inner})"),
            Some(Side::Err) => format!("Err({inner})"),
            None => inner,
        };
        let mut s = format!(
            "{} {binder}{} in {domain}",
            q.kind.word(),
            if q.leftover { " leftover" } else { "" }
        );
        if let Some(w) = &q.where_ {
            let _ = write!(s, " where {}", self.expr(w));
        }
        if let Some(b) = &q.body {
            let _ = write!(s, ": {}", self.expr(b));
        }
        if let Some(e) = &q.else_ {
            let _ = write!(s, " else {}", self.expr(e));
        }
        if pushed {
            self.scope.pop();
        }
        s
    }

    fn abbrev(&mut self, a: &Abbrev) -> String {
        let body = self.expr(&a.body);
        if a.params.is_empty() {
            format!("{} = {body}", a.name)
        } else {
            let ps: Vec<String> = a.params.iter().map(|(p, t)| format!("{p}: {t}")).collect();
            format!("{}({}) = {body}", a.name, ps.join(", "))
        }
    }
}

/// One expression as Surety text, binder names as written.
pub fn expr(e: &Expr) -> String {
    let mut p = Printer {
        fresh: false,
        counter: 0,
        scope: Vec::new(),
    };
    p.expr(e)
}

/// An abbreviation as Surety text.
pub fn abbrev(a: &Abbrev) -> String {
    let mut p = Printer {
        fresh: false,
        counter: 0,
        scope: Vec::new(),
    };
    p.abbrev(a)
}

/// The column prose wraps at. A tab counts as four columns.
pub const WIDTH: usize = 100;

/// The columns a line occupies, a tab counting as four.
pub fn columns(s: &str) -> usize {
    s.chars().map(|c| if c == '\t' { 4 } else { 1 }).sum()
}

/// Prose under a prefix (`\t# `, `\t// `, `// `), wrapped at the width:
/// broken at spaces, every continuation line under the same prefix, a
/// word longer than the room on its own line.
pub fn wrap(prefix: &str, text: &str) -> String {
    let room = WIDTH.saturating_sub(columns(prefix)).max(1);
    let mut out = String::new();
    let mut line = String::new();
    for word in text.split(' ').filter(|w| !w.is_empty()) {
        if line.is_empty() {
            line.push_str(word);
        } else if line.chars().count() + 1 + word.chars().count() <= room {
            line.push(' ');
            line.push_str(word);
        } else {
            out.push_str(prefix);
            out.push_str(&line);
            out.push('\n');
            line = word.to_string();
        }
    }
    if line.is_empty() {
        out.push_str(prefix.trim_end());
    } else {
        out.push_str(prefix);
        out.push_str(&line);
    }
    out.push('\n');
    out
}

enum Item<'a> {
    Abbrev(&'a Abbrev),
    Property(&'a Property),
    Accept(&'a Property),
    Example(&'a Example),
    /// A sentence with no property beneath it: a draft's, kept in place.
    Sentence(&'a Pin),
    Comment(&'a Pin),
}

fn line_of(i: &Item) -> usize {
    match i {
        Item::Abbrev(a) => a.line,
        Item::Property(p) | Item::Accept(p) => p.line,
        Item::Example(e) => e.line,
        Item::Sentence(p) | Item::Comment(p) => p.line,
    }
}

fn decl_line(d: &Decl) -> usize {
    match d {
        Decl::Type(t) => t.line,
        Decl::NamedValue(nv) => nv.line,
        Decl::Abbrev(a) => a.line,
        Decl::Function(f) => f.line,
    }
}

/// Where a pin belongs: before the declaration at this index, or inside
/// the body of the function at this index.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Before(usize),
    Body(usize),
}

/// A pin is anchored to the line that follows it: the next declaration, or,
/// when the declaration before it is a function, that function's body.
fn slot_of(line: usize, decls: &[Decl]) -> Slot {
    let next = decls
        .iter()
        .position(|d| decl_line(d) > line)
        .unwrap_or(decls.len());
    match next.checked_sub(1).map(|i| &decls[i]) {
        Some(Decl::Function(_)) => Slot::Body(next - 1),
        _ => Slot::Before(next),
    }
}

/// The whole file. `fresh` renames every binder in order of appearance.
/// Prose is wrapped at `WIDTH`; comments stay anchored to the line that
/// follows them; a sentence with no property beneath it stays in place with
/// a blank line between it and the next, since consecutive `#` lines read
/// as one sentence.
pub fn file(f: &File, fresh: bool) -> String {
    let mut p = Printer {
        fresh,
        counter: 0,
        scope: Vec::new(),
    };
    let mut out = String::new();
    for (_, l) in &f.header {
        out.push_str(l);
        out.push('\n');
    }
    let attached: Vec<&str> = f
        .decls
        .iter()
        .filter_map(|d| match d {
            Decl::Function(fun) => Some(fun),
            _ => None,
        })
        .flat_map(|fun| fun.properties.iter().chain(&fun.accepts))
        .filter_map(|pr| pr.sentence.as_deref())
        .collect();
    let pins: Vec<(&Pin, Slot)> = f
        .pins
        .iter()
        .filter(|x| match x.kind {
            PinKind::Comment => true,
            PinKind::Sentence => !attached.contains(&x.text.as_str()),
            PinKind::Note => false,
        })
        .map(|x| (x, slot_of(x.line, &f.decls)))
        .collect();
    let top_level = |out: &mut String, slot: Slot| {
        let mut after_sentence = false;
        for (pin, _) in pins.iter().filter(|(_, s)| *s == slot) {
            let is_sentence = pin.kind == PinKind::Sentence;
            if is_sentence && after_sentence {
                out.push('\n');
            }
            after_sentence = is_sentence;
            let prefix = if is_sentence { "# " } else { "// " };
            out.push_str(&wrap(prefix, &pin.text));
        }
    };
    let mut last_was_type = false;
    for (i, d) in f.decls.iter().enumerate() {
        match d {
            Decl::Type(t) => {
                top_level(&mut out, Slot::Before(i));
                let _ = writeln!(out, "type {} = {}", t.name, t.ty);
                last_was_type = true;
            }
            Decl::NamedValue(nv) => {
                if last_was_type {
                    out.push('\n');
                }
                top_level(&mut out, Slot::Before(i));
                let value = p.expr(&nv.value);
                match &nv.ty {
                    Some(t) => {
                        let _ = writeln!(out, "{}: {t} = {value}", nv.name);
                    }
                    None => {
                        let _ = writeln!(out, "{} = {value}", nv.name);
                    }
                }
                last_was_type = false;
            }
            Decl::Abbrev(a) => {
                if last_was_type {
                    out.push('\n');
                }
                top_level(&mut out, Slot::Before(i));
                let line = p.abbrev(a);
                out.push_str(&line);
                out.push('\n');
                last_was_type = false;
            }
            Decl::Function(fun) => {
                out.push('\n');
                top_level(&mut out, Slot::Before(i));
                let ins: Vec<String> = fun
                    .inputs
                    .iter()
                    .map(|(n, t)| format!("{n}: {t}"))
                    .collect();
                let _ = writeln!(out, "{}({}) -> {}", fun.name, ins.join(", "), fun.output);
                if let Some(n) = &fun.note {
                    let _ = writeln!(out, "\tnote \"{n}\"");
                }
                let mut items: Vec<Item> = Vec::new();
                items.extend(fun.abbrevs.iter().map(Item::Abbrev));
                items.extend(fun.properties.iter().map(Item::Property));
                items.extend(fun.accepts.iter().map(Item::Accept));
                items.extend(fun.examples.iter().map(Item::Example));
                items.extend(
                    pins.iter()
                        .filter(|(_, s)| *s == Slot::Body(i))
                        .map(|(pin, _)| match pin.kind {
                            PinKind::Sentence => Item::Sentence(pin),
                            _ => Item::Comment(pin),
                        }),
                );
                items.sort_by_key(line_of);
                let mut after_sentence = false;
                for item in items {
                    // Any `#` line after a sentence standing alone starts a
                    // sentence of its own, so a blank line keeps the two
                    // apart: another sentence standing alone, or one over a
                    // property or an accept line.
                    let opens_with_sentence = match &item {
                        Item::Sentence(_) => true,
                        Item::Property(pr) | Item::Accept(pr) => pr.sentence.is_some(),
                        _ => false,
                    };
                    if opens_with_sentence && after_sentence {
                        out.push('\n');
                    }
                    after_sentence = matches!(item, Item::Sentence(_));
                    match item {
                        Item::Abbrev(a) => {
                            let line = p.abbrev(a);
                            let _ = writeln!(out, "\t{line}");
                        }
                        Item::Property(pr) => {
                            if let Some(s) = &pr.sentence {
                                out.push_str(&wrap("\t# ", s));
                            }
                            let e = p.expr(&pr.expr);
                            let _ = writeln!(out, "\t{e}");
                        }
                        Item::Accept(pr) => {
                            if let Some(s) = &pr.sentence {
                                out.push_str(&wrap("\t# ", s));
                            }
                            let e = p.expr(&pr.expr);
                            let _ = writeln!(out, "\taccept {e}");
                        }
                        Item::Example(ex) => {
                            out.push_str("\n\te:\n");
                            if let Some(n) = &ex.note {
                                let _ = writeln!(out, "\t\tnote \"{n}\"");
                            }
                            let i = p.expr(&ex.input);
                            let o = p.expr(&ex.output);
                            let _ = writeln!(out, "\t\tin: {i}");
                            let _ = writeln!(out, "\t\tout: {o}");
                        }
                        Item::Sentence(pin) => {
                            out.push_str(&wrap("\t# ", &pin.text));
                        }
                        Item::Comment(pin) => {
                            out.push_str(&wrap("\t// ", &pin.text));
                        }
                    }
                }
                last_was_type = false;
            }
        }
    }
    top_level(&mut out, Slot::Before(f.decls.len()));
    out
}
