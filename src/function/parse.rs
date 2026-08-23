//! # fn-form signature parsers
//!
//! ## Why this module exists
//!
//! Stone 241.18a — moved from `src/runtime.rs` (`parse_fn_signature`) and
//! `src/check.rs` (`parse_fn_signature_for_check`, `parse_fn_signature_for_check_diag`)
//! into the dedicated `src/function/` namespaced home per
//! `feedback_namespaced_home_vigilia_gate` REMARKABLE bar.
//!
//! These parsers share a single concern: consuming the canonical fn-form
//! signature layout `([name <- :T ...] -> :Ret)` and producing the parsed
//! triple `(param_names, param_types, ret_type)`. They differ only in their
//! error-handling contract:
//!
//! - `parse_fn_signature` — eval-tier; hard `RuntimeError` on malformed input.
//! - `parse_fn_signature_for_check` — check-tier silent; `Err(())` on mismatch.
//!
//! `parse_fn_signature_for_check_diag` — diagnostic check-tier — lives in
//! `infer.rs` because `infer_fn` is its only caller (keeping caller and callee
//! co-located per solvere).
//!
//! All three route argspec-triples through `crate::argspec::parse_argspec_triples`
//! (the canonical parser established at Stone 241.1-241.3).
//!
//! ## Arity contract
//!
//! Parsers accept exactly 3 args (after body and metadata are stripped by callers):
//!   `[ARGS-VECTOR, ->, :RET-TYPE]`
//! Body is NOT a parser concern — callers synthesize body independently.
//! Arity is enforced by the type (`&[WatAST; 3]`) — a wrong-length slice is
//! rejected at the call site, not inside the prefix. `ArityMismatch` as a
//! runtime variant is structurally impossible (Stone 243.4.1).

use crate::argspec::{parse_argspec_triples, ArgSpec, ArgSpecErrorKind, ParseOptions};
use crate::ast::WatAST;
use crate::function::FN_HEAD;
use crate::runtime::{RuntimeError, RuntimeErrorKind};
use crate::scope::Identifier;
use crate::span::Span;
use crate::types::{parse_type_node, TypeErrorKind, TypeExpr};

/// Parsed fn-form signature — named struct eliminating the 4-tuple type complexity.
///
/// Generic over the name type `N` so the single struct serves both the internal
/// prefix tier (`N = Identifier`) and the eval-tier callers (`N = String`, after
/// env_key mapping).  Both `parse_fn_signature_prefix` and
/// `parse_fn_signature_with_rest` return this type, removing their
/// `#[allow(clippy::type_complexity)]` exemptions.
///
/// WHY pub(in crate::function): the prefix is home-internal; eval.rs (in the
/// same home) destructures the String-typed variant directly.
pub(in crate::function) struct ParsedFnSignature<N> {
    /// Fixed positional parameter names.
    pub params: Vec<N>,
    /// Type annotations for each fixed parameter, parallel to `params`.
    pub param_types: Vec<TypeExpr>,
    /// Declared return type.
    pub ret_type: TypeExpr,
    /// Optional rest-binder `& name <- :T` — `None` for non-variadic forms.
    pub rest: Option<(N, TypeExpr)>,
}

/// Step-location type for `parse_fn_signature_prefix` — Pattern A
/// (CONFORMARE.md §"The pattern").
///
/// `ParseStep` is the outer struct; span is STRUCTURALLY required — a
/// spanless parse-step error is uncompilable. `ParseStepKind` carries
/// variant-specific data only (no per-variant span fields).
///
/// The three outer parsers (`parse_fn_signature`, `parse_fn_signature_for_check`,
/// `parse_fn_signature_for_check_diag`) map this type to their tier's error
/// contract; the prefix itself does NOT produce error messages.
///
/// WHY pub(in crate::function): ParseStep is a fn-form parser intermediary
/// shared by parse.rs and infer.rs; pub(crate) would expose an internal
/// ladder type to the whole substrate; private would prevent infer.rs access.
pub(in crate::function) struct ParseStep {
    pub span: Span,
    pub kind: ParseStepKind,
}

/// Variant-specific data for `ParseStep` (Pattern A kind enum).
///
/// No span fields per variant — span lives on the outer `ParseStep` struct.
/// `ArityMismatch` has been eliminated: arity is now type-impossible via
/// `&[WatAST; 3]` (Stone 243.4.1 / CONFORMARE.md worked example).
pub(in crate::function) enum ParseStepKind {
    /// The first slot was not a `WatAST::Vector`. Carries the variant name
    /// of the form that was found instead.
    ArgsVecNotVector { found_variant: &'static str },
    /// The second slot was not the bare symbol `->`. Carries the variant name
    /// of the form that was found instead.
    ArrowMissing { found_variant: &'static str },
    /// The third slot was not a keyword. Carries the variant name of the form
    /// that was found instead.
    RetTypeNotKeyword { found_variant: &'static str },
    /// The return-type keyword failed `parse_type_expr_with_span`. Wraps the
    /// span-free `TypeErrorKind` — the outer `ParseStep.span` carries the
    /// location; storing `Box<TypeErrorKind>` avoids the double-stamp that
    /// `TypeError.to_string()` (which embeds the span in its Display) would
    /// produce when later placed into a `RuntimeError { span: step.span, .. }`.
    BadRetType(Box<TypeErrorKind>),
    /// The argspec triples failed. Wraps the span-free `ArgSpecErrorKind` —
    /// the outer `ParseStep.span` carries the location; `ArgSpecErrorKind::reason()`
    /// produces the span-free message.
    ArgSpecFailed(Box<ArgSpecErrorKind>),
}

impl ParseStepKind {
    /// Span-free human reason for this parse-step failure. Both tier mappers
    /// (eval RuntimeError, check CheckError) render through this — one source of truth.
    pub(in crate::function) fn reason(&self) -> String {
        match self {
            ParseStepKind::ArgsVecNotVector { found_variant } =>
                format!("fn signature: expected a vector `[name <- :T ...]` as the args-vector; got {found_variant}"),
            ParseStepKind::ArrowMissing { found_variant } =>
                format!("fn signature: expected `->` between args-vector and return type; got {found_variant}"),
            ParseStepKind::RetTypeNotKeyword { found_variant } =>
                format!("fn signature: expected a return-type keyword after `->` (e.g. `:wat::core::i64`); got {found_variant}"),
            ParseStepKind::BadRetType(k) => format!("invalid return type: {k}"),
            ParseStepKind::ArgSpecFailed(k) => k.reason(),
        }
    }
}

/// Shared structural prefix: Vector destructure + arrow match
/// + ret_type parse + argspec parse.
///
/// Returns `Err(ParseStep)` at the first structural mismatch; the three
/// outer parsers map the step to their tier's error contract.
///
/// Callers pass a fixed 3-element array `&[WatAST; 3]`: `[ARGS-VECTOR, ->, :RET-TYPE]`.
/// Arity is type-guaranteed — no runtime arity check required.
///
/// `parse_fn_signature_for_check_diag` in `src/function/infer.rs` routes through
/// this prefix and maps each `ParseStep` variant to its diagnostic tier contract.
///
/// ## Rest-binder support
///
/// `options.allow_rest_binder` controls whether `& name <- :T` is accepted in the
/// args-vector. When `true`, the `rest` field of `ParsedFnSignature` carries the
/// rest-binder pair; when `false` (strict callers: `parse_fn_signature`,
/// `parse_fn_signature_for_check`), `&` in the args-vector produces an error and
/// `rest` is always `None`.
pub(in crate::function) fn parse_fn_signature_prefix(
    sig: &[WatAST; 3],
    options: ParseOptions,
) -> Result<ParsedFnSignature<Identifier>, ParseStep> {
    let (args_vec, args_vec_span) = match &sig[0] {
        WatAST::Vector(items, span) => (items.as_slice(), span),
        other => {
            return Err(ParseStep {
                span: other.span().clone(),
                kind: ParseStepKind::ArgsVecNotVector {
                    found_variant: other.variant_name(),
                },
            });
        }
    };
    // Arc 251.4a — accept the `:-` annotation keyword (core.typed parity) as a
    // dual-read alias for the legacy `->` return arrow. The `->` arrow HARD-CUTs at
    // 251.5. (This is the fn-SIGNATURE arrow at sig[1], distinct from the `->`
    // threading-macro call head and from the `:->` fn-TYPE arrow of 251.4c.)
    let is_annotation_arrow =
        sig[1].is_bare_symbol("->") || crate::types::is_binder_marker(&sig[1]);
    if !is_annotation_arrow {
        return Err(ParseStep {
            span: sig[1].span().clone(),
            kind: ParseStepKind::ArrowMissing {
                found_variant: sig[1].variant_name(),
            },
        });
    }
    // Arc 251.3a — accept Keyword (existing), Symbol (wat.type/X pre-normalize), or
    // List ((wat.type/Vector wat.type/i64) parametric form) in the return-type slot.
    let ret_type: TypeExpr = match &sig[2] {
        WatAST::Keyword(_, _) | WatAST::Symbol(_, _) | WatAST::List(_, _) | WatAST::Vector(_, _) => {
            parse_type_node(&sig[2]).map_err(|te| ParseStep {
                span: te.span().clone(),
                kind: ParseStepKind::BadRetType(Box::new(te.into_kind())),
            })?
        }
        other => {
            return Err(ParseStep {
                span: other.span().clone(),
                kind: ParseStepKind::RetTypeNotKeyword {
                    found_variant: other.variant_name(),
                },
            });
        }
    };
    let argspec: ArgSpec = parse_argspec_triples(
        args_vec,
        FN_HEAD,
        args_vec_span,
        options,
    )
    .map_err(|ae| ParseStep {
        span: ae.span,
        kind: ParseStepKind::ArgSpecFailed(Box::new(ae.kind)),
    })?;
    let rest = argspec.rest_param;
    let (params, param_types): (Vec<Identifier>, Vec<TypeExpr>) =
        argspec.fixed_params.into_iter().unzip();
    Ok(ParsedFnSignature { params, param_types, ret_type, rest })
}

/// Arc 167 — flat-shape fn signature parser (eval tier).
///
/// Consumes the canonical fn-form signature layout (3 elements after body is
/// stripped by caller):
///
///   `[ARGS-VECTOR, ->, :RET-TYPE]`
///
/// `ARGS-VECTOR` is a `WatAST::Vector` whose body is flat triples
/// `name <- :T name <- :T ...` (empty vector → zero-arity fn). The
/// `<-` token reads as "consumes" — input direction; the sibling
/// `->` reads as "produces" — output direction. Arrows-as-duals.
///
/// Per 058-029, every parameter is typed and the return type is
/// required. No "untyped fn" exists in wat. This parser rejects
/// malformed flat-shape signatures with location-bearing errors.
///
/// Caller synthesizes body independently; parser sees only the 3-element
/// signature prefix. Moved from `src/runtime.rs` at Stone 241.18a.
pub(crate) fn parse_fn_signature(
    args: &[WatAST; 3],
) -> Result<(Vec<crate::scope::Identifier>, Vec<TypeExpr>, TypeExpr), RuntimeError> {
    let sig = parse_fn_signature_prefix(args, ParseOptions { allow_rest_binder: false }).map_err(|step| RuntimeError::new(step.span, RuntimeErrorKind::MalformedForm {
        head: FN_HEAD.into(),
        reason: step.kind.reason()
    }))?;
    // Arc 170 — carry the binders THEMSELVES; flattening to env_key here is
    // what baked a scope id into a name and made a binder un-remappable.
    Ok((sig.params, sig.param_types, sig.ret_type))
}

/// Arc 150 — eval-tier fn signature parser that accepts `& name <- :T` rest binders.
///
/// Extends `parse_fn_signature` with rest-binder support: parses with
/// `allow_rest_binder: true` and returns the optional rest `(name, type)` pair.
/// Called by `eval_fn` so that variadic fn-forms (produced by expanding a variadic
/// `defn`) can be evaluated into a `Function` value with `rest_param` set.
///
/// Returns a `ParsedFnSignature<Identifier>` — the binders whole, so callers bind
/// directly into `Function.params` (`Vec<Identifier>`). `Function.rest_param` is
/// a lookup KEY rather than a re-emitted binder node, so the caller flattens
/// that one with `env_key`.
/// `rest` is `Some((name, type))` when a `& name <- :T` binder was present;
/// `None` otherwise.
pub(crate) fn parse_fn_signature_with_rest(
    args: &[WatAST; 3],
) -> Result<ParsedFnSignature<crate::scope::Identifier>, RuntimeError> {
    let sig = parse_fn_signature_prefix(args, ParseOptions { allow_rest_binder: true }).map_err(|step| RuntimeError::new(step.span, RuntimeErrorKind::MalformedForm {
        head: FN_HEAD.into(),
        reason: step.kind.reason()
    }))?;
    // Arc 170 — binders carried whole (see parse_fn_signature); the caller
    // flattens the rest name to an env_key where `Function.rest_param` wants a
    // lookup key rather than a binder node.
    Ok(ParsedFnSignature { params: sig.params, param_types: sig.param_types, ret_type: sig.ret_type, rest: sig.rest })
}

/// Arc 167 — mirror of `parse_fn_signature` for the check pass.
///
/// Consumes the flat-shape fn-form signature layout (3 elements after body is
/// stripped by caller):
///
///   `[ARGS-VECTOR, ->, :RET-TYPE]`
///
/// Returns (names, types, ret). This is the SILENT CLASSIFIER: it answers
/// "is this a well-formed fn-shape?" for the `:ensure :fn` validator, NOT the
/// DIAGNOSTIC parser (`parse_fn_signature_for_check_diag` in `infer.rs`,
/// which surfaces each `ParseStep` as a `CheckError::MalformedForm` whose reason
/// comes from `ParseStepKind::reason()` — for the argspec case, that delegates to
/// `ArgSpecErrorKind::reason()`).
///
/// rune:sequi(reclassified-by-caller) — the `ArgSpecError` detail is
/// intentionally discarded to `()`. The sole caller (`:ensure :fn` validation
/// in `check.rs`) re-surfaces a coarser `CheckError::EnsureFnInvalid`
/// ("malformed :fn signature …") on the `Err(())` arm — that message IS the
/// intended UX for this path. Threading the sub-step detail through would
/// impair the deliberately-coarse `:ensure` diagnostic, so the discard is the
/// fix, not a defect. (The fine-grained path is A3, which keeps the detail.)
///
/// Moved from `src/check.rs` at Stone 241.18a.
pub(crate) fn parse_fn_signature_for_check(
    args: &[WatAST; 3],
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ()> {
    let sig = parse_fn_signature_prefix(args, ParseOptions { allow_rest_binder: false }).map_err(|_| ())?;
    let params = sig.params.iter().map(|id| crate::scope::env_key(id).into_owned()).collect();
    Ok((params, sig.param_types, sig.ret_type))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::Identifier;
    use crate::span::Span;

    fn span() -> Span {
        crate::rust_caller_span!()
    }

    fn arrow() -> WatAST {
        WatAST::Symbol(Identifier::bare("->"), span())
    }

    /// Helper: build a minimal valid 3-element signature prefix `[empty-vec, ->, :ret]`.
    fn sig_vec_arrow_ret(ret_kw: &str) -> [WatAST; 3] {
        [
            WatAST::Vector(vec![], span()),
            arrow(),
            WatAST::Keyword(ret_kw.into(), span()),
        ]
    }

    // ─── Lines 151-157: ArgsVecNotVector production site ──────────────────────

    /// `parse_fn_signature_prefix` must return `ArgsVecNotVector` (lines 151-157)
    /// when `sig[0]` is not a `WatAST::Vector`. The `reason()` arm for
    /// `ArgsVecNotVector` (lines 114-115) is exercised via `parse_fn_signature`
    /// which calls `step.kind.reason()` in its error mapper.
    #[test]
    fn prefix_non_vector_args_returns_args_vec_not_vector() {
        // sig[0] is a Keyword, not a Vector — triggers ArgsVecNotVector.
        let sig: [WatAST; 3] = [
            WatAST::Keyword(":not-a-vec".into(), span()),
            arrow(),
            WatAST::Keyword(":wat::core::i64".into(), span()),
        ];
        let err = parse_fn_signature(&sig).unwrap_err();
        let reason = match err.kind() {
            crate::runtime::RuntimeErrorKind::MalformedForm { reason, .. } => reason,
            _ => panic!("expected MalformedForm, got {:?}", err),
        };
        // reason() for ArgsVecNotVector (lines 114-115) is "fn signature: expected a vector …"
        assert_eq!(reason, "fn signature: expected a vector `[name <- :T ...]` as the args-vector; got keyword");
    }

    // ─── Lines 171-173: BadRetType production + lines 120: reason() ───────────

    /// `parse_fn_signature_prefix` must return `BadRetType` (lines 171-173)
    /// when `sig[2]` is syntactically a keyword but fails `parse_type_expr_with_span`
    /// (e.g. `:Any` is banned). The `reason()` arm for `BadRetType` (line 120)
    /// formats the inner `TypeErrorKind` via its `Display` impl.
    #[test]
    fn prefix_banned_ret_type_returns_bad_ret_type() {
        // `:Any` is a valid keyword syntactically but fails reject_any check.
        let sig = sig_vec_arrow_ret(":Any");
        let err = parse_fn_signature(&sig).unwrap_err();
        let reason = match err.kind() {
            crate::runtime::RuntimeErrorKind::MalformedForm { reason, .. } => reason,
            _ => panic!("expected MalformedForm, got {:?}", err),
        };
        // reason() for BadRetType (line 120): "invalid return type: {kind}"
        // Arc 296 — the remedy text CHANGED, and this pin is why the change was safe to make.
        // It used to steer every `:Any` offender to `:wat::holon::HolonAST for any algebra
        // value`. Builder ruling, 2026-08-15: *"we shouldn't be using HolonAST for anything but
        // VSA ops — its been misused extensively — WatAST is the replacement for HolonAST in all
        // places but VSA/HDC."* A diagnostic is a TEACHER, and this one was teaching the misuse
        // (it steered the author of this change, an hour before the ruling landed).
        assert_eq!(reason, "invalid return type: :Any is not part of the type system (058-030); use :wat::WatAST for any wat form, :wat::holon::HolonAST ONLY for a VSA/HDC algebra value, a named enum for closed heterogeneous sets, or parametric T/K/V for generics. Offending expression: :Any");
    }

    // ─── Lines 244-246: parse_fn_signature_with_rest error mapper ─────────────

    /// `parse_fn_signature_with_rest` (lines 244-246) must propagate a
    /// `RuntimeError` when `parse_fn_signature_prefix` fails. Triggered by
    /// a non-Vector args slot (ArgsVecNotVector path).
    #[test]
    fn parse_fn_signature_with_rest_non_vector_args_returns_runtime_error() {
        let sig: [WatAST; 3] = [
            WatAST::IntLit(42, span()),
            arrow(),
            WatAST::Keyword(":wat::core::i64".into(), span()),
        ];
        // Map to string inline — ParsedFnSignature<String> doesn't implement Debug,
        // so unwrap_err() is not usable directly; match avoids the Debug bound.
        let err = match parse_fn_signature_with_rest(&sig) {
            Ok(_) => panic!("expected Err for non-Vector args-slot; got Ok"),
            Err(e) => e,
        };
        let reason = match err.kind() {
            crate::runtime::RuntimeErrorKind::MalformedForm { reason, .. } => reason,
            _ => panic!("expected MalformedForm, got {:?}", err),
        };
        assert_eq!(reason, "fn signature: expected a vector `[name <- :T ...]` as the args-vector; got int");
    }

    // ─── Lines 161-167: ArrowMissing production site ──────────────────────────

    /// `parse_fn_signature_prefix` must return `ArrowMissing` when `sig[1]` is
    /// not the bare symbol `->`. The existing stone18a_errors.rs E03 exercises
    /// this via `infer_fn`; this unit test exercises the prefix directly.
    #[test]
    fn prefix_missing_arrow_returns_arrow_missing() {
        let sig: [WatAST; 3] = [
            WatAST::Vector(vec![], span()),
            WatAST::Keyword(":not-arrow".into(), span()), // not "->"
            WatAST::Keyword(":wat::core::i64".into(), span()),
        ];
        let err = parse_fn_signature(&sig).unwrap_err();
        let reason = match err.kind() {
            crate::runtime::RuntimeErrorKind::MalformedForm { reason, .. } => reason,
            _ => panic!("expected MalformedForm, got {:?}", err),
        };
        assert_eq!(reason, "fn signature: expected `->` between args-vector and return type; got keyword");
    }
}
