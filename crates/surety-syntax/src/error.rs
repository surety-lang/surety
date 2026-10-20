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
    TypeError,
    UnknownType,
    AmbiguousType,
    UnknownField,
    DuplicateDefinition,
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
            ErrorKind::TypeError => "type_error",
            ErrorKind::UnknownType => "unknown_type",
            ErrorKind::AmbiguousType => "ambiguous_type",
            ErrorKind::UnknownField => "unknown_field",
            ErrorKind::DuplicateDefinition => "duplicate_definition",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, line: usize, message: impl Into<String>) -> Self {
        Error {
            kind,
            line,
            message: message.into(),
        }
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
