use crate::span::Span;
use crate::types::TypeError;

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
    /// A type keyword in slot 2 (or the ret-type slot) failed
    /// `parse_type_expr_with_span`. The inner error carries the
    /// specific parse failure.
    MalformedTypeKeyword {
        span: Span,
        head: String,
        inner: Box<TypeError>,
    },
    /// `include_ret_type = true` but no `"->"` symbol found after
    /// the final fixed-param triple.
    MissingRetArrow { span: Span, head: String },
    /// `"->"` found but the following slot was not a Keyword.
    RetTypeNotKeyword { span: Span, head: String },
    /// Items remain after the expected end of the argspec
    /// (after the ret-type slot when `include_ret_type = true`,
    /// or after the final triple when `include_ret_type = false`).
    TrailingItems { span: Span, head: String, count: usize },
    /// The triple at some position is incomplete — fewer than 3
    /// items remain before end-of-slice or a terminator.
    IncompleteSignature { span: Span, head: String },
    /// A `&` rest-binder marker is present in the args-vector but
    /// `allow_rest_binder = false`. Rest-binder support is Stone 241.4;
    /// 241.1 rejects this case explicitly so the error is surfaced
    /// rather than silently misinterpreted as a malformed name slot.
    RestBinderNotSupported { span: Span, head: String },
}

// ─── From<ArgSpecError> impls ─────────────────────────────────────────────────
//
// Per AUDIT.md "Recommendation for 241.1": these wire the canonical error into
// each call-site's native error class. 241.2/241.3 callers convert at their
// boundary; the parser itself emits only ArgSpecError.

impl From<ArgSpecError> for crate::runtime::RuntimeError {
    fn from(err: ArgSpecError) -> Self {
        // Map each variant to RuntimeError::MalformedForm, mirroring
        // parse_fn_signature / parse_defclause_args error construction
        // at src/runtime.rs:6750 / src/runtime.rs:6880.
        match err {
            ArgSpecError::NameNotSymbol { span, head } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: "arg-vector name slot must be a plain symbol (not a keyword, literal, or nested form)".into(),
                    span,
                }
            }
            ArgSpecError::MissingArrow { span, head } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: "arg-vector triple must be `name <- :T`; `<-` arrow not found at slot 1".into(),
                    span,
                }
            }
            ArgSpecError::TypeNotKeyword { span, head } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: "arg-vector type slot must be a keyword (e.g. `:wat::core::i64`); got a non-keyword".into(),
                    span,
                }
            }
            ArgSpecError::MalformedTypeKeyword { span, head, inner } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: format!("arg-vector type keyword is malformed: {inner}"),
                    span,
                }
            }
            ArgSpecError::MissingRetArrow { span, head } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: "expected `->` return-type arrow after arg triples; not found".into(),
                    span,
                }
            }
            ArgSpecError::RetTypeNotKeyword { span, head } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: "return-type slot after `->` must be a keyword; got a non-keyword".into(),
                    span,
                }
            }
            ArgSpecError::TrailingItems { span, head, count } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: format!("arg-vector has {count} trailing item(s) beyond the expected signature shape"),
                    span,
                }
            }
            ArgSpecError::IncompleteSignature { span, head } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: "arg-vector triple is incomplete; expected `name <- :T` but ran out of items".into(),
                    span,
                }
            }
            ArgSpecError::RestBinderNotSupported { span, head } => {
                crate::runtime::RuntimeError::MalformedForm {
                    head,
                    reason: "`&` rest-binder is not supported at this binding site".into(),
                    span,
                }
            }
        }
    }
}

impl From<ArgSpecError> for crate::check::CheckError {
    fn from(err: ArgSpecError) -> Self {
        // Map to CheckError::MalformedForm, mirroring parse_fn_signature_for_check_diag
        // at src/check.rs:15258.
        match err {
            ArgSpecError::NameNotSymbol { span, head } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: "arg-vector name slot must be a plain symbol".into(),
                    span,
                }
            }
            ArgSpecError::MissingArrow { span, head } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: "arg-vector triple must be `name <- :T`; `<-` arrow not found".into(),
                    span,
                }
            }
            ArgSpecError::TypeNotKeyword { span, head } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: "arg-vector type slot must be a keyword".into(),
                    span,
                }
            }
            ArgSpecError::MalformedTypeKeyword { span, head, inner } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: format!("arg-vector type keyword is malformed: {inner}"),
                    span,
                }
            }
            ArgSpecError::MissingRetArrow { span, head } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: "expected `->` return-type arrow after arg triples; not found".into(),
                    span,
                }
            }
            ArgSpecError::RetTypeNotKeyword { span, head } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: "return-type slot after `->` must be a keyword".into(),
                    span,
                }
            }
            ArgSpecError::TrailingItems { span, head, count } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: format!("{count} trailing item(s) beyond the expected signature shape"),
                    span,
                }
            }
            ArgSpecError::IncompleteSignature { span, head } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: "arg-vector triple is incomplete; ran out of items before `name <- :T` was satisfied".into(),
                    span,
                }
            }
            ArgSpecError::RestBinderNotSupported { span, head } => {
                crate::check::CheckError::MalformedForm {
                    head,
                    reason: "`&` rest-binder is not supported at this binding site".into(),
                    span,
                }
            }
        }
    }
}

impl From<ArgSpecError> for TypeError {
    fn from(err: ArgSpecError) -> Self {
        // Map to TypeError::MalformedDecl, mirroring parse_struct / parse_struct_restricted
        // at src/types.rs:2002+ (B1/B2 sites).
        match err {
            ArgSpecError::NameNotSymbol { span, head } => TypeError::MalformedDecl {
                head,
                reason: "field/arg name slot must be a plain symbol".into(),
                span,
            },
            ArgSpecError::MissingArrow { span, head } => TypeError::MalformedDecl {
                head,
                reason: "field/arg triple must be `name <- :T`; `<-` arrow not found".into(),
                span,
            },
            ArgSpecError::TypeNotKeyword { span, head } => TypeError::MalformedDecl {
                head,
                reason: "field/arg type slot must be a keyword".into(),
                span,
            },
            ArgSpecError::MalformedTypeKeyword { span, head, inner } => TypeError::MalformedDecl {
                head,
                reason: format!("field/arg type keyword is malformed: {inner}"),
                span,
            },
            ArgSpecError::MissingRetArrow { span, head } => TypeError::MalformedDecl {
                head,
                reason: "expected `->` return-type arrow; not found".into(),
                span,
            },
            ArgSpecError::RetTypeNotKeyword { span, head } => TypeError::MalformedDecl {
                head,
                reason: "return-type slot after `->` must be a keyword".into(),
                span,
            },
            ArgSpecError::TrailingItems { span, head, count } => TypeError::MalformedDecl {
                head,
                reason: format!("{count} trailing item(s) beyond the expected signature shape"),
                span,
            },
            ArgSpecError::IncompleteSignature { span, head } => TypeError::MalformedDecl {
                head,
                reason: "arg/field triple is incomplete; ran out of items".into(),
                span,
            },
            ArgSpecError::RestBinderNotSupported { span, head } => TypeError::MalformedDecl {
                head,
                reason: "`&` rest-binder is not supported at this declaration site".into(),
                span,
            },
        }
    }
}
