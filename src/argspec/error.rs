use crate::span::Span;
use crate::types::{TypeError, TypeErrorKind};

/// Canonical argspec parse error — Pattern A (Stone 243.3 / CONFORMARE.md).
///
/// The outer struct carries the universal `span` and `head` fields; span is
/// STRUCTURALLY imposed — a spanless argspec error is uncompilable. The kind
/// enum carries only variant-specific data. This home was Pattern A's founding
/// precedent (arc 241 enforced span discipline by convention); Stone 243.3
/// elevates the convention to structure.
///
/// Callers convert at their site boundary via `From<ArgSpecError>` into their
/// native error class (`RuntimeError`, `CheckError`, `TypeError`, `MacroError`).
/// The canonical error type stays the single source of truth for argspec
/// malformedness; the four `From<>` impls are forward-compatible substrate.
#[derive(Debug)]
pub struct ArgSpecError {
    pub span: Span,
    pub head: String,
    pub kind: ArgSpecErrorKind,
}

/// Variant-specific data for `ArgSpecError` (Pattern A kind enum).
///
/// Universal fields (`span`, `head`) live on the outer struct; every variant
/// here carries ONLY what differs between failure modes.
#[derive(Debug)]
pub enum ArgSpecErrorKind {
    /// Slot 0 of a name-arrow-type triple was not a Symbol.
    NameNotSymbol,
    /// Slot 1 of a triple was not the bare Symbol `"<-"`.
    MissingArrow,
    /// Slot 2 of a triple was not a Keyword.
    TypeNotKeyword,
    /// A type keyword in slot 2 failed `parse_type_expr_with_span`.
    /// The inner error carries the specific parse failure.
    MalformedTypeKeyword { inner: Box<TypeError> },
    /// Items remain after the expected end of the argspec
    /// (after the rest-binder triple).
    TrailingItems { count: usize },
    /// The triple at some position is incomplete — fewer than 3
    /// items remain before end-of-slice or a rest-marker.
    IncompleteTriple,
    /// A `&` rest-binder marker is present in the args-vector but
    /// `allow_rest_binder = false`. Rest-binder support is Stone 241.4;
    /// 241.1 rejects this case explicitly so the error is surfaced
    /// rather than silently misinterpreted as a malformed name slot.
    RestBinderNotSupported,
}

impl ArgSpecErrorKind {
    /// The human-readable reason for this failure shape.
    pub(crate) fn reason(&self) -> String {
        match self {
            ArgSpecErrorKind::NameNotSymbol =>
                "name slot must be a plain symbol (not a keyword, literal, or nested form)".into(),
            ArgSpecErrorKind::MissingArrow =>
                "triple must be `name <- :T`; `<-` arrow not found at slot 1".into(),
            ArgSpecErrorKind::TypeNotKeyword =>
                "type slot must be a keyword (e.g. `:wat::core::i64`); got a non-keyword".into(),
            ArgSpecErrorKind::MalformedTypeKeyword { inner } =>
                format!("type keyword is malformed: {inner}"),
            ArgSpecErrorKind::TrailingItems { count } =>
                format!("{count} trailing item(s) beyond the expected argspec shape"),
            ArgSpecErrorKind::IncompleteTriple =>
                "triple is incomplete; expected `name <- :T` but ran out of items".into(),
            ArgSpecErrorKind::RestBinderNotSupported =>
                "`&` rest-binder is not supported at this binding site".into(),
        }
    }
}

// ─── From<ArgSpecError> impls ─────────────────────────────────────────────────
//
// Per AUDIT.md "Recommendation for 241.1": these wire the canonical error into
// each call-site's native error class. 241.2/241.3 callers convert at their
// boundary; the parser itself emits only ArgSpecError.

impl From<ArgSpecError> for crate::runtime::RuntimeError {
    fn from(e: ArgSpecError) -> Self {
        let reason = e.kind.reason();
        Self::MalformedForm { head: e.head, reason, span: e.span }
    }
}

impl From<ArgSpecError> for crate::check::CheckError {
    fn from(e: ArgSpecError) -> Self {
        let reason = e.kind.reason();
        Self::MalformedForm { head: e.head, reason, span: e.span, remedies: vec![] }
    }
}

impl From<ArgSpecError> for TypeError {
    fn from(e: ArgSpecError) -> Self {
        let reason = e.kind.reason();
        TypeError { span: e.span, kind: TypeErrorKind::MalformedDecl { head: e.head, reason } }
    }
}

impl From<ArgSpecError> for crate::macros::MacroError {
    fn from(e: ArgSpecError) -> Self {
        let reason = e.kind.reason();
        crate::macros::MacroError::MalformedDefmacro { reason, span: e.span }
    }
}
