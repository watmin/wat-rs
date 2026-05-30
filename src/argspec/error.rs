use crate::span::Span;
use crate::types::{TypeError, TypeErrorKind};

/// Sum of failure modes for canonical argspec parsing.
///
/// Every variant carries `span: Span` (per AUDIT.md line 161) so error
/// reporting always points at the user's source, even when no specific
/// offending element provides a finer span.
///
/// Callers convert at their site boundary via `From<ArgSpecError>` into
/// their native error class (`RuntimeError`, `CheckError`, `TypeError`).
/// The canonical error type stays the single source of truth for
/// argspec malformedness; the three `From<>` impls are forward-compatible
/// substrate for 241.2/241.3/241.7 migrations.
#[derive(Debug)]
pub enum ArgSpecError {
    /// Slot 0 of a name-arrow-type triple was not a Symbol.
    NameNotSymbol { span: Span, head: String },
    /// Slot 1 of a triple was not the bare Symbol `"<-"`.
    MissingArrow { span: Span, head: String },
    /// Slot 2 of a triple was not a Keyword.
    TypeNotKeyword { span: Span, head: String },
    /// A type keyword in slot 2 failed `parse_type_expr_with_span`.
    /// The inner error carries the specific parse failure.
    MalformedTypeKeyword {
        span: Span,
        head: String,
        inner: Box<TypeError>,
    },
    /// Items remain after the expected end of the argspec
    /// (after the rest-binder triple).
    TrailingItems { span: Span, head: String, count: usize },
    /// The triple at some position is incomplete — fewer than 3
    /// items remain before end-of-slice or a rest-marker.
    IncompleteTriple { span: Span, head: String },
    /// A `&` rest-binder marker is present in the args-vector but
    /// `allow_rest_binder = false`. Rest-binder support is Stone 241.4;
    /// 241.1 rejects this case explicitly so the error is surfaced
    /// rather than silently misinterpreted as a malformed name slot.
    RestBinderNotSupported { span: Span, head: String },
}

impl ArgSpecError {
    /// Decompose into `(span, head, reason)` with domain-neutral reason strings.
    ///
    /// The `head` field carries form context (`:wat::core::defn` vs
    /// `:wat::core::defstruct`); reasons stay neutral — no "arg-vector"
    /// or "field/arg" prefix. Each `From<>` impl collapses to a 4-line
    /// wrapper around this method.
    fn into_parts(self) -> (Span, String, String) {
        match self {
            ArgSpecError::NameNotSymbol { span, head } => (
                span,
                head,
                "name slot must be a plain symbol (not a keyword, literal, or nested form)".into(),
            ),
            ArgSpecError::MissingArrow { span, head } => (
                span,
                head,
                "triple must be `name <- :T`; `<-` arrow not found at slot 1".into(),
            ),
            ArgSpecError::TypeNotKeyword { span, head } => (
                span,
                head,
                "type slot must be a keyword (e.g. `:wat::core::i64`); got a non-keyword".into(),
            ),
            ArgSpecError::MalformedTypeKeyword { span, head, inner } => (
                span,
                head,
                format!("type keyword is malformed: {inner}"),
            ),
            ArgSpecError::TrailingItems { span, head, count } => (
                span,
                head,
                format!("{count} trailing item(s) beyond the expected argspec shape"),
            ),
            ArgSpecError::IncompleteTriple { span, head } => (
                span,
                head,
                "triple is incomplete; expected `name <- :T` but ran out of items".into(),
            ),
            ArgSpecError::RestBinderNotSupported { span, head } => (
                span,
                head,
                "`&` rest-binder is not supported at this binding site".into(),
            ),
        }
    }
}

// ─── From<ArgSpecError> impls ─────────────────────────────────────────────────
//
// Per AUDIT.md "Recommendation for 241.1": these wire the canonical error into
// each call-site's native error class. 241.2/241.3 callers convert at their
// boundary; the parser itself emits only ArgSpecError.

impl From<ArgSpecError> for crate::runtime::RuntimeError {
    fn from(err: ArgSpecError) -> Self {
        let (span, head, reason) = err.into_parts();
        Self::MalformedForm { head, reason, span }
    }
}

impl From<ArgSpecError> for crate::check::CheckError {
    fn from(err: ArgSpecError) -> Self {
        let (span, head, reason) = err.into_parts();
        Self::MalformedForm { head, reason, span, remedies: vec![] }
    }
}

impl From<ArgSpecError> for TypeError {
    fn from(err: ArgSpecError) -> Self {
        let (span, head, reason) = err.into_parts();
        TypeError { span, kind: TypeErrorKind::MalformedDecl { head, reason } }
    }
}
