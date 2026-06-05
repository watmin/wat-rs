use crate::span::{span_prefix, Span};
use std::fmt;

/// Errors during macro registration / expansion. Pattern A (Stone
/// 243.7d): span at the outer struct level; variant data in
/// `MacroErrorKind`.
#[derive(Debug)]
pub struct MacroError {
    pub span: Span,
    pub kind: MacroErrorKind,
}

/// Variant data for [`MacroError`]. Spans live in the outer struct;
/// variants carry ONLY data unique to each failure kind.
// MacroErrorKind is pub because it's the type of MacroError's pub `kind` field (no private-in-public).
#[derive(Debug)]
pub enum MacroErrorKind {
    /// Two `(:wat::core::defmacro ...)` forms registered the same name.
    DuplicateMacro(String),
    /// A user macro declared under a reserved `:wat::...` prefix.
    ReservedPrefix(String),
    /// A `defmacro` form was malformed.
    MalformedDefmacro { reason: String },
    /// The macro's body wasn't a quasiquote template — this slice only
    /// supports quasiquote bodies.
    UnsupportedBody { name: String, reason: String },
    /// A macro call passed the wrong number of arguments.
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
    /// An `unquote` reference named a parameter the macro didn't declare.
    UnboundMacroParam { name: String },
    /// `unquote-splicing` was applied to a non-sequence argument.
    /// Accepts `WatAST::List` and `WatAST::Vector` (arc 200 made splice
    /// symmetric across both); fires for any other shape (Atom, Symbol,
    /// non-Vec runtime value, etc.). wat has no user-facing List runtime
    /// type — sequence here means "splice-compatible AST shape or runtime
    /// Vec value."
    SpliceNotSequence { name: String, got: &'static str },
    /// Expansion depth exceeded a sanity limit — probably an infinite
    /// recursive macro.
    ExpansionDepthExceeded { limit: usize },
    /// Other malformation in a macro invocation or template.
    MalformedTemplate { reason: String },
}

impl fmt::Display for MacroErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacroErrorKind::DuplicateMacro(n) => {
                write!(f, "duplicate macro registration: {}", n)
            }
            MacroErrorKind::ReservedPrefix(n) => write!(
                f,
                "cannot declare macro {} — reserved prefix ({}); user macros must use their own prefix",
                n,
                crate::resolve::reserved_prefix_list()
            ),
            MacroErrorKind::MalformedDefmacro { reason } => {
                write!(f, "malformed defmacro: {}", reason)
            }
            MacroErrorKind::UnsupportedBody { name, reason } => write!(
                f,
                "macro {} body not supported: {} (this slice handles quasiquote-template bodies only)",
                name, reason
            ),
            MacroErrorKind::ArityMismatch { name, expected, got } => {
                write!(
                    f,
                    "macro {} expects {} arguments; got {}",
                    name, expected, got
                )
            }
            MacroErrorKind::UnboundMacroParam { name } => {
                write!(f, "unquote references unbound macro parameter: {}", name)
            }
            MacroErrorKind::SpliceNotSequence { name, got } => write!(
                f,
                "unquote-splicing (~@{}) requires a sequence (List/Vector AST or Vec value); got {}",
                name, got
            ),
            MacroErrorKind::ExpansionDepthExceeded { limit } => write!(
                f,
                "macro expansion exceeded depth limit {} — likely infinite recursion",
                limit
            ),
            MacroErrorKind::MalformedTemplate { reason } => {
                write!(f, "malformed template: {}", reason)
            }
        }
    }
}

impl fmt::Display for MacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = span_prefix(&self.span);
        write!(f, "{}{}", prefix, self.kind)
    }
}

impl std::error::Error for MacroError {}
