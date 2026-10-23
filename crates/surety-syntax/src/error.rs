//! One error type for the parser, the resolver, and the extractor.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Syntax,
    UnexpectedIndent,
    UnboundName,
    QuantifierMidLine,
    WhereWithoutBody,
    TwoBodies,
    UnusedBinder,
    ElementTypeMismatch,
    PatternAsValue,
    ElseWithoutWhere,
    AssignmentInProperty,
    ExampleInNotLiteral,
    ExampleOutNotLiteral,
    ExampleOutMissing,
    TypeError,
    UnknownType,
    AmbiguousType,
    UnknownField,
    DuplicateDefinition,
    EmptyLeftover,
    UnresolvedChoice,
    NotARecord,
}

impl ErrorKind {
    /// The spelling used in `.err` files.
    pub fn name(self) -> &'static str {
        match self {
            ErrorKind::Syntax => "syntax",
            ErrorKind::UnexpectedIndent => "unexpected_indent",
            ErrorKind::UnboundName => "unbound_name",
            ErrorKind::QuantifierMidLine => "quantifier_mid_line",
            ErrorKind::WhereWithoutBody => "where_without_body",
            ErrorKind::TwoBodies => "two_bodies",
            ErrorKind::UnusedBinder => "unused_binder",
            ErrorKind::ElementTypeMismatch => "element_type_mismatch",
            ErrorKind::PatternAsValue => "pattern_as_value",
            ErrorKind::ElseWithoutWhere => "else_without_where",
            ErrorKind::AssignmentInProperty => "assignment_in_property",
            ErrorKind::ExampleInNotLiteral => "example_in_not_literal",
            ErrorKind::ExampleOutNotLiteral => "example_out_not_literal",
            ErrorKind::ExampleOutMissing => "example_out_missing",
            ErrorKind::TypeError => "type_error",
            ErrorKind::UnknownType => "unknown_type",
            ErrorKind::AmbiguousType => "ambiguous_type",
            ErrorKind::UnknownField => "unknown_field",
            ErrorKind::DuplicateDefinition => "duplicate_definition",
            ErrorKind::EmptyLeftover => "empty_leftover",
            ErrorKind::UnresolvedChoice => "unresolved_choice",
            ErrorKind::NotARecord => "not_a_record",
        }
    }
}

/// An edit that mends an error: on the error's line, the term `find`
/// becomes `replace`. The editor offers it as a quick fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub title: String,
    pub find: String,
    pub replace: String,
}

/// An edit built on the tree and verified (`mend.rs`): lines `from..=to`
/// of the file replaced by `text`, one line or several, indentation
/// included. Offered only when `text` parses to the tree it was printed
/// from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub title: String,
    pub from: usize,
    pub to: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub message: String,
    /// The columns on `line` the error is about, start inclusive and end
    /// exclusive, counted in characters from 0; none when only the line is
    /// known.
    pub span: Option<(usize, usize)>,
    /// The edit that mends it, when there is one beyond doubt.
    pub fix: Option<Box<Fix>>,
    /// The whole-line edit that mends it, verified on the tree.
    pub edit: Option<Box<Edit>>,
}

impl Error {
    pub fn new(kind: ErrorKind, line: usize, message: impl Into<String>) -> Self {
        Error {
            kind,
            line,
            message: message.into(),
            span: None,
            fix: None,
            edit: None,
        }
    }

    /// The same error with the edit that mends it.
    pub fn fixed_by(mut self, fix: Fix) -> Self {
        self.fix = Some(Box::new(fix));
        self
    }

    /// The same error about these columns of its line; an empty span is
    /// left out.
    pub fn spanning(mut self, span: Option<(usize, usize)>) -> Self {
        self.span = span.filter(|(from, to)| to > from);
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}: {} ({})",
            self.line,
            self.message,
            self.kind.name()
        )
    }
}

impl std::error::Error for Error {}
