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
//!
//! ## Arc 109 addendum — `defclause`/`extend-type`/`derive` parsers join this file
//!
//! A `defclause` is the fn-form's multi-clause shape, so its parser
//! (`parse_defclause_clause`, `parse_defclause_form`, `is_defclause_form`) and the two
//! declaration-form parsers that share its clause-body grammar (`parse_extend_type_form`,
//! `parse_derive_form`) moved verbatim out of `src/runtime.rs` here (arc 109 the
//! defclause-into-function-home stone) — parsing joins parsing, per this home's ACT split.
//! `mod arc109_two_iii_defclause_return_slot` travels with `parse_defclause_clause` as its
//! own `#[cfg(test)]` probe.
//!
//! `declared_type_subsumes` (the reachability wall `parse_defclause_form` calls) lives in
//! the new `subsume.rs` sibling — runtime type-matching, not a parser concern.
//!
//! `parse_defclause_form` and `is_defclause_form` were the two DESIGN-listed items that were
//! fully `pub` in `src/runtime.rs` (a `pub mod`); every other item in this addendum was
//! already `pub(crate)`/private. Both are narrowed to `pub(crate)` here to match this
//! home's convention (`pub(crate) mod function` in `lib.rs` already caps their real
//! reachability to the crate, so this is a visibility-narrowing that changes no behaviour) —
//! their only callers, measured, are `src/check.rs` (5 sites) and
//! `src/declare/register.rs` (4 sites), both crate-internal.

use crate::argspec::{parse_argspec_triples, ArgSpec, ArgSpecErrorKind, ParseOptions};
use crate::ast::WatAST;
use crate::declare::parse::{parse_type_keyword, try_parse_metadata_map};
use crate::function::subsume::declared_type_subsumes;
use crate::function::FN_HEAD;
use crate::scope::Identifier;
use crate::span::Span;
use crate::types::{parse_type_node, TypeErrorKind, TypeExpr};
use crate::value::{Clause, ClauseSet, Function, FunctionBody, RuntimeError, RuntimeErrorKind};
use std::sync::Arc;

// `synthesize_fn_body` is genuinely defined in `crate::runtime` (not a facade re-export of a
// `crate::value` type — see STOP-2) and stays there; it is not one of this stone's 12 items.
use crate::runtime::synthesize_fn_body;

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

// ─── Stone 237.2 — defclause parse + eval ────────────────────────────────────

/// Parse a single clause from a defclause.
///
/// Stone 237.3 full shape:
///   `([args] :guard? expr :ensure? :fn-form -> :RetType? body)`
///
/// Keyword order FIXED: args → :guard? → :ensure? → -> :T? → body.
/// `:ensure` before `:guard` is a parse-time error (probe 13).
/// Multiple `:guard` in the same clause is a parse-time error (probe 12).
fn parse_defclause_clause(
    clause_form: &WatAST,
    head: &str,
    shared_return: Option<&crate::types::TypeExpr>,
) -> Result<Clause, RuntimeError> {
    let form_span = clause_form.span().clone();
    let items = match clause_form {
        WatAST::List(items, _) => items,
        other => {
            return Err(RuntimeError::new(form_span.clone(), RuntimeErrorKind::MalformedForm {
                head: head.into(),
                reason: format!(
                    "each defclause clause must be a list `([args] -> :Ret body)` or `([args] body)` (with shared return); got {}",
                    other.variant_name()
                )
            }));
        }
    };
    if items.is_empty() {
        return Err(RuntimeError::new(
            form_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: head.into(),
                reason: "defclause clause must not be empty".into(),
            },
        ));
    }

    // items[0] must be the args-vector.
    let args_vec = match &items[0] {
        WatAST::Vector(v, _) => v,
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: format!(
                        "defclause clause must start with args-vector `[name <- :T ...]`; got {}",
                        other.variant_name()
                    ),
                },
            ));
        }
    };

    let spec = crate::argspec::parse_argspec_triples(
        args_vec,
        head,
        &form_span,
        crate::argspec::ParseOptions {
            allow_rest_binder: true,
        },
    )?;

    // Arc 293.4e-pre: store the ArgSpec directly — no unroll into Vec<(String,TypeExpr)>.
    // Consumers read clause.args.fixed_params / clause.args.rest_param, using
    // env_key(&id) where a String binder name is needed.

    // Stone 237.3: flexible scan of items[1..] for optional :guard, :ensure, ->, body.
    //
    // Only hard ordering constraint: :guard (if present) must appear BEFORE :ensure
    // (if present). The `-> :T` annotation may appear in any position between the
    // keyword/value pairs and the body. Remaining items after consuming :guard,
    // :ensure, and -> :T are the body.
    //
    // Probes exercise these orderings:
    //   :guard expr -> :T body             (probes 1, 3, 4)
    //   -> :T :ensure (:fn) body           (probes 6–10)
    //   :guard expr :ensure (:fn) -> :T body  (probe 11)
    //   :ensure (:fn) -> :T body           (probe 14 3-arity)
    //
    // Rejected: :guard after :ensure (probe 13), multiple :guard (probe 12).
    let rest = &items[1..];

    // --- Phase 1: Pre-scan to locate :guard, :ensure, -> positions and validate order ---
    // Scan all items; track first positions of each special token.
    let mut guard_kw_pos: Option<usize> = None;
    let mut ensure_kw_pos: Option<usize> = None;
    let mut arrow_pos: Option<usize> = None;
    let mut second_guard_pos: Option<usize> = None;

    let mut i = 0;
    while i < rest.len() {
        if matches!(&rest[i], WatAST::Keyword(k, _) if k.as_str() == ":guard") {
            if guard_kw_pos.is_none() {
                guard_kw_pos = Some(i);
                i += 2; // skip :guard + expr
            } else {
                // Second :guard found — record for rejection below.
                second_guard_pos = Some(i);
                break;
            }
        } else if matches!(&rest[i], WatAST::Keyword(k, _) if k.as_str() == ":ensure") {
            if ensure_kw_pos.is_none() {
                ensure_kw_pos = Some(i);
                i += 2; // skip :ensure + fn-form
            } else {
                i += 1;
            }
        } else if matches!(&rest[i], WatAST::Symbol(s, _) if s.as_str() == "->") {
            if arrow_pos.is_none() {
                arrow_pos = Some(i);
                i += 2; // skip -> + :T
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    // Reject: :guard after :ensure (probe 13 — order violation).
    if let (Some(gpos), Some(epos)) = (guard_kw_pos, ensure_kw_pos) {
        if gpos > epos {
            return Err(RuntimeError::new(rest[gpos].span().clone(), RuntimeErrorKind::MalformedForm {
                head: head.into(),
                reason: "defclause clause has `:guard` after `:ensure` — fixed order is: args → :guard? → :ensure? → body".into()
            }));
        }
    }

    // Reject: multiple :guard (probe 12).
    if let Some(spos) = second_guard_pos {
        return Err(RuntimeError::new(rest[spos].span().clone(), RuntimeErrorKind::MalformedForm {
            head: head.into(),
            reason: "defclause clause has multiple `:guard` keywords — only one `:guard` per clause is permitted (compose multiple conditions with :and)".into()
        }));
    }

    // --- Phase 2: Extract :guard expression ---
    let guard_ast: Option<Arc<WatAST>> = if let Some(gpos) = guard_kw_pos {
        let expr_pos = gpos + 1;
        if expr_pos >= rest.len() {
            return Err(RuntimeError::new(
                form_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: "`:guard` keyword must be followed by a guard expression".into(),
                },
            ));
        }
        Some(Arc::new(rest[expr_pos].clone()))
    } else {
        None
    };

    // --- Phase 3: Extract :ensure fn-form ---
    let ensure_ast: Option<Arc<WatAST>> = if let Some(epos) = ensure_kw_pos {
        let fn_pos = epos + 1;
        if fn_pos >= rest.len() {
            return Err(RuntimeError::new(
                form_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: "`:ensure` keyword must be followed by a :fn form".into(),
                },
            ));
        }
        Some(Arc::new(rest[fn_pos].clone()))
    } else {
        None
    };

    // --- Phase 4: Extract return type from -> :T (if present) ---
    let return_type: crate::types::TypeExpr = if let Some(apos) = arrow_pos {
        let type_pos = apos + 1;
        if type_pos >= rest.len() {
            return Err(RuntimeError::new(
                form_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: "defclause clause `->` must be followed by a return type keyword"
                        .into(),
                },
            ));
        }
        match &rest[type_pos] {
            WatAST::Keyword(k, _) => parse_type_keyword(k)?,
            // Arc 109 ②-iii — widen to accept the `:-` reference FORM
            // `(Head :- [T …])` too, routed through `parse_type_node` (the
            // substrate's one door reading all four type node shapes,
            // src/types/surface.rs:345), same as γ-i's
            // `src/function/parse.rs:178`. Additive only — the `Keyword`
            // arm above is untouched, so the keyword path stays
            // byte-identical.
            list @ WatAST::List(_, _) => crate::types::parse_type_node(list).map_err(|e| {
                RuntimeError::new(
                    e.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: head.into(),
                        reason: e.to_string(),
                    },
                )
            })?,
            other => {
                return Err(RuntimeError::new(
                    other.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: head.into(),
                        reason: format!(
                        "defclause clause `->` must be followed by a return type keyword; got {}",
                        other.variant_name()
                    ),
                    },
                ));
            }
        }
    } else {
        // No per-clause `->` — use shared_return.
        match shared_return {
            Some(r) => r.clone(),
            None => {
                return Err(RuntimeError::new(form_span.clone(), RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: "defclause clause missing `-> :RetType`; either add per-clause `-> :T` or provide a top-level shared return type".into()
                }));
            }
        }
    };

    // --- Phase 5: Body = all items not consumed by :guard, :ensure, -> :T ---
    // Consumed positions: guard_kw_pos, guard_kw_pos+1, ensure_kw_pos, ensure_kw_pos+1,
    //                     arrow_pos, arrow_pos+1.
    let mut consumed: std::collections::HashSet<usize> = std::collections::HashSet::new();
    if let Some(gpos) = guard_kw_pos {
        consumed.insert(gpos);
        consumed.insert(gpos + 1);
    }
    if let Some(epos) = ensure_kw_pos {
        consumed.insert(epos);
        consumed.insert(epos + 1);
    }
    if let Some(apos) = arrow_pos {
        consumed.insert(apos);
        consumed.insert(apos + 1);
    }
    let body_items: Vec<WatAST> = rest
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed.contains(idx))
        .map(|(_, item)| item.clone())
        .collect();

    if body_items.is_empty() {
        return Err(RuntimeError::new(
            form_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: head.into(),
                reason: "defclause clause must have a body expression".into(),
            },
        ));
    }
    let body_ast = synthesize_fn_body(&body_items);

    Ok(Clause {
        args: spec,
        return_type,
        guard: guard_ast,
        ensure_fn: ensure_ast,
        body: Arc::new(body_ast),
        // Filled in at ClauseSet assembly, where the set's NAME is in scope (the Function's
        // name is what a call-stack frame shows).
        func: None,
    })
}

// ─── Arc 109 ②-iii — acceptance rows 1 & 3 for `defclause`'s return-type slot ────────
//
// `parse_defclause_clause`'s `-> :T` slot now accepts the `:-` reference FORM
// `(Head :- [T …])` alongside the existing `Keyword`, routed through
// `crate::types::parse_type_node`. Calls the parser DIRECTLY (not via
// `startup_from_source`/`--check`) for the same reason `collection/eval.rs`'s sibling
// probes do: the corpus migration this stone ships exposed a THIRD, out-of-boundary
// keyword-only guard (`crate::check::infer_list_constructor` /
// `infer_hashset_constructor`) that currently fails to freeze the stdlib itself — see
// `src/collection/eval.rs`'s `arc109_two_iii_ctor_guard_widening` module doc for the
// full account. `defclause` itself is unaffected by that third class (this guard is
// reused by BOTH `runtime.rs` eval and `check.rs`'s `infer_defclause`, which calls
// `parse_defclause_form` — the SAME parser under test here), but calling it directly
// keeps this probe independent of whether the wider corpus happens to check clean.
#[cfg(test)]
mod arc109_two_iii_defclause_return_slot {
    use super::{parse_defclause_clause, RuntimeErrorKind};

    /// Row 1 — `defclause` takes a form return: `-> (:wat::core::Vector :- [:wat::core::i64])`.
    #[test]
    fn row1_defclause_form_return_type() {
        let clause = crate::parse_one!(
            "([n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::i64]) n)"
        )
        .expect("parse defclause clause with form return");
        let parsed = parse_defclause_clause(&clause, ":probe::defclause-row1", None)
            .unwrap_or_else(|e| panic!("form return type must parse: {e:?}"));
        let type_node = crate::parse_one!("(:wat::core::Vector :- [:wat::core::i64])")
            .expect("parse the equivalent bare type form");
        let expected = crate::types::parse_type_node(&type_node)
            .expect("the reference form itself must parse via the substrate's one door");
        assert_eq!(
            parsed.return_type, expected,
            "defclause's `-> (Head :- [T …])` must parse to exactly what parse_type_node \
             yields for the same form standing alone"
        );
    }

    /// Row 3 (the row that decides the stone) — the KEYWORD return-type path is
    /// untouched: same probe, keyword spelling, must still parse to the SAME shape it
    /// always did.
    #[test]
    fn row3_defclause_keyword_return_type_unchanged() {
        let clause = crate::parse_one!("([n <- :wat::core::i64] -> :wat::core::i64 n)")
            .expect("parse defclause clause with keyword return");
        let parsed = parse_defclause_clause(&clause, ":probe::defclause-row3", None)
            .unwrap_or_else(|e| panic!("keyword return type must still parse: {e:?}"));
        assert_eq!(
            parsed.return_type,
            crate::types::TypeExpr::Path(":wat::core::i64".into())
        );
    }

    /// Row 3 negative control — a return slot that was rejected BEFORE the widening
    /// (neither `Keyword` nor now `List` — a bare symbol) must still be rejected, with
    /// the SAME diagnostic shape.
    #[test]
    fn row3_defclause_still_rejects_non_type_return_slot() {
        let clause = crate::parse_one!("([n <- :wat::core::i64] -> n n)")
            .expect("parse defclause clause with a bare-symbol return slot");
        let err = parse_defclause_clause(&clause, ":probe::defclause-row3-neg", None)
            .expect_err("a bare symbol return slot must still be rejected");
        // Structured, not string-matched (`no_loose_string_assert`'s own remedy — ask
        // through the door, whose argument is an enum): the pre-existing diagnostic
        // shape, byte-identical to before the widening.
        assert_eq!(
            format!("{:?}", err.kind()),
            format!(
                "{:?}",
                RuntimeErrorKind::MalformedForm {
                    head: ":probe::defclause-row3-neg".into(),
                    reason: "defclause clause `->` must be followed by a return type keyword; \
                             got symbol"
                        .into()
                }
            )
        );
    }
}

/// Stone 237.2 — parse and register a defclause form.
///
/// Form: `(:wat::core::defclause :name [-> :T] (clause...) ...)` where
/// each clause is `([args] -> :T body)` (Option B) or `([args] body)` with a
/// top-level shared return type (Option A).
///
/// Returns the name + Arc<ClauseSet> on success.
pub(crate) fn parse_defclause_form(
    form: &WatAST,
    privilege: crate::resolve::Privilege,
) -> Result<(String, Arc<ClauseSet>), RuntimeError> {
    const HEAD: &str = ":wat::core::defclause";
    let form_span = form.span().clone();
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(RuntimeError::new(
                form_span,
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: "expected list".into(),
                },
            ))
        }
    };
    // items[0] = :wat::core::defclause keyword
    // items[1] = :name keyword
    // Optional: items[2] could be `->` symbol followed by :T keyword (Option A)
    // Then: one or more clause forms
    if items.len() < 2 {
        return Err(RuntimeError::new(
            form_span,
            RuntimeErrorKind::MalformedForm {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::defclause :name [-> :T] clause ...); got {} elements",
                    items.len()
                ),
            },
        ));
    }

    // items[1] must be the name keyword.
    let name = match &items[1] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "defclause first arg must be a keyword name (e.g. `:my::name`); got {}",
                        other.variant_name()
                    ),
                },
            ));
        }
    };

    // Check for reserved prefix / namespacing on :name (user-code guard; skipped for
    // privileged stdlib calls via allow_reserved=true). Stone 237.8b: stdlib defclauses
    // live under :wat::core::*. Phase-1 migration to the ONE gate (defclause). A
    // standalone declaration guard with no adjacent dedup, so Existing::Absent — gate
    // yields Unnamespaced (reject), DottedName (reject), Reserved (reject), or Insert
    // (proceed). Arc 296 stone H-1: DottedName gets its own explicit arm here (not
    // folded into the `_` wildcard below) so a dotted defclause name is refused the
    // same as everywhere else the gate is consulted — the wildcard would otherwise have
    // silently treated it as Insert.
    // The name-legality check happens here; the actual defclause-table insert happens
    // downstream in `register_defclause` (Stub/Runtime phase, keyed off its own presence
    // check) — this door has nothing of its own to insert, so the closure is a no-op.
    crate::resolve::register(
        &name,
        privilege,
        crate::resolve::Existing::Absent,
        &form_span,
        || -> Result<(), RuntimeError> { Ok(()) },
    )?;

    // Optional metadata-map, mirroring def/defn: `(:wat::core::defclause :name
    // {meta} [-> :T] clause ...)`. Detected structurally via `is_metadata_map`
    // (accepts both the native `WatAST::Map` literal and the legacy
    // `:wat::core::HashMap` constructor-call List shape) so it can never be
    // confused with a `-> :T` sugar pair or a clause list (a clause is always
    // a `WatAST::List` starting with a `WatAST::Vector` args-list — neither
    // shape `is_metadata_map` accepts unless it is literally the HashMap
    // constructor form). Consumed BEFORE the `-> :T` detection below, so
    // `-> :T` sugar still works with or without a preceding metadata-map.
    let (metadata, items) = if items.len() > 2 && items[2].is_metadata_map() {
        let meta = try_parse_metadata_map(&items[2]).ok_or_else(|| {
            RuntimeError::new(
                items[2].span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: "defclause metadata-map has a non-keyword key — every key in \
                    `{...}` must be a keyword (e.g. `{:restricted-to [...]}`)"
                        .into(),
                },
            )
        })?;
        let mut rest = Vec::with_capacity(items.len() - 1);
        rest.push(items[0].clone());
        rest.push(items[1].clone());
        rest.extend_from_slice(&items[3..]);
        (Some(meta), std::borrow::Cow::Owned(rest))
    } else {
        (None, std::borrow::Cow::Borrowed(items.as_slice()))
    };
    let items: &[WatAST] = &items;

    // Detect Option A: `-> :T` after the name keyword.
    let (shared_return, clause_offset) = {
        let after_name = &items[2..];
        if after_name.len() >= 2 {
            match (&after_name[0], &after_name[1]) {
                (WatAST::Symbol(s, _), WatAST::Keyword(k, _)) if s.as_str() == "->" => {
                    let ret = parse_type_keyword(k)?;
                    (Some(ret), 4usize) // items[0]=head items[1]=name items[2]='-> items[3]=:T items[4..]=clauses
                }
                _ => (None, 2usize), // items[2..] = clauses
            }
        } else {
            (None, 2usize)
        }
    };

    let clause_items = &items[clause_offset..];
    if clause_items.is_empty() {
        return Err(RuntimeError::new(
            form_span,
            RuntimeErrorKind::MalformedForm {
                head: HEAD.into(),
                reason: "defclause must have at least one clause".into(),
            },
        ));
    }

    let mut clauses = Vec::with_capacity(clause_items.len());
    for clause_form in clause_items {
        let mut clause = parse_defclause_clause(clause_form, HEAD, shared_return.as_ref())?;
        // Clause-TCO stone — compile the clause to an ordinary `Function` ONCE, here, where the
        // set name is in scope. `eval_tail` hands this to the existing TailCall signal, so a
        // clause head tail-calls exactly like a `defn` head. Mirrors the extend-type method
        // synthesis (this file, `register_extend_type_methods`), which is why extend-type
        // methods already had TCO and defclause verbs did not.
        clause.func = Some(Arc::new(Function {
            name: Some(name.clone()),
            params: clause.args.fixed_params.iter().map(|(n, _)| n.clone()).collect(),
            type_params: vec![],
            param_types: clause.args.fixed_params.iter().map(|(_, t)| t.clone()).collect(),
            ret_type: clause.return_type.clone(),
            rest_param: clause
                .args
                .rest_param
                .as_ref()
                .map(|(n, _)| crate::scope::env_key(n).into_owned()),
            rest_param_type: clause.args.rest_param.as_ref().map(|(_, t)| t.clone()),
            body: FunctionBody::Wat(clause.body.clone()),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        }));
        clauses.push(clause);
    }

    // ── Stone 118.B2c strike 1 — THE REACHABILITY WALL ──────────────────────────────────
    //
    // An arm that no input can ever reach is dead code, and until this wall nothing said so:
    // `(defclause :my::pick ([x <- :i64] "FIRST") ([x <- :i64] "SECOND"))` type-checked, ran,
    // and answered "FIRST" forever while the second body sat unreachable and silent.
    //
    // ★ THIS IS THE REDEF RULE REACHING THE ONE REGISTRY IT NEVER COVERED. Arc 054 made
    // typealias/define/defmacro "if byte-equivalent, no-op", else DuplicateDefine; clause ARMS
    // were exempt because an arm is not a definition BY NAME — so the only registry that
    // dispatches on TYPES had no define-once rule. An arm that can never fire is a definition
    // with no effect. Builder, 2026-08-18: "you may only express something's def once and all
    // other attempts must be identical."
    //
    // ARMED AT ZERO OFFENDERS (the house pattern). A corpus census over 1,457 .wat files found
    // exactly ONE unreachable arm, and it is the fixture written to be refused
    // (tests/types/probe_stone_118_b2c_overlapping_arms_are_silent.wat). See
    // MEASURED-118.B2c-strike1-the-corpus-is-NOT-clean.md for the run and for the WRONG first
    // predicate (intersection) that this one replaced.
    //
    // Three deliberate conservatisms, each so the wall refuses only what is PROVABLY dead:
    //   - a GUARDED earlier arm never subsumes — `:guard` can evaluate false
    //     (`ClauseFailureReason::GuardFalse` is a real dispatch outcome), so it cannot render a
    //     later arm unreachable;
    //   - VARIADIC arms are skipped entirely — a rest-param accepts a range of arities and the
    //     pairwise test does not model that;
    //   - PAIRWISE only — three arms whose first two JOINTLY exhaust the type universe would
    //     leave a third provably dead and this wall will not see it. That is undecidable in
    //     general, and under-firing is the correct bias for a wall.
    for later_idx in 1..clauses.len() {
        for earlier_idx in 0..later_idx {
            let earlier = &clauses[earlier_idx];
            let later = &clauses[later_idx];
            if earlier.guard.is_some() {
                continue;
            }
            if earlier.args.rest_param.is_some() || later.args.rest_param.is_some() {
                continue;
            }
            if earlier.args.fixed_params.len() != later.args.fixed_params.len() {
                continue;
            }
            let subsumed = earlier
                .args
                .fixed_params
                .iter()
                .zip(later.args.fixed_params.iter())
                .all(|((_, e_ty), (_, l_ty))| declared_type_subsumes(e_ty, l_ty));
            if subsumed {
                return Err(RuntimeError::new(
                    form.span().clone(),
                    RuntimeErrorKind::UnreachableClause {
                        name: name.clone(),
                        clause_index: later_idx,
                        subsumed_by: earlier_idx,
                        declared_arg_types: later
                            .args
                            .fixed_params
                            .iter()
                            .map(|(_, t)| crate::check::format_type(t))
                            .collect(),
                    },
                ));
            }
        }
    }

    Ok((
        name.clone(),
        Arc::new(ClauseSet {
            name,
            clauses,
            shared_return,
            metadata,
        }),
    ))
}

// ─── Arc 232 Stone 232.1 — extend-type parse fn ──────────────────────────────

/// Arc 232 Stone 232.1 — parse an `(:wat::core::extend-type :T :P (m1 [self ...] body) ...)` form.
///
/// Shape:
/// ```
/// (:wat::core::extend-type :t::Robot :t::Greeter
///   (greet [self loudness] "beep"))
/// ```
///
/// Each impl is parsed as a defclause clause body (argspec WITHOUT type
/// annotations for the self/arg binders, then body). Returns `(canonical_key, Arc<ExtendDef>)`.
/// Canonical key: `"extend:<P>:<T>"` — unique per `(P, T)` pair.
pub(crate) fn parse_extend_type_form(
    form: &WatAST,
) -> Result<(String, Arc<crate::value::ExtendDef>), RuntimeError> {
    const HEAD: &str = ":wat::core::extend-type";
    let form_span = form.span().clone();
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(RuntimeError::new(
                form_span,
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: "expected list".into(),
                },
            ))
        }
    };
    // items[0] = :wat::core::extend-type
    // items[1] = :T type name keyword
    // items[2] = :P protocol name keyword
    // items[3..] = method impl lists (method-name [self ...] body)
    if items.len() < 3 {
        return Err(RuntimeError::new(
            form_span,
            RuntimeErrorKind::MalformedForm {
                head: HEAD.into(),
                reason: format!(
                "expected (:wat::core::extend-type :T :P (method-impl ...) ...); got {} elements",
                items.len()
            ),
            },
        ));
    }
    // Arc 109 identity 2c remainder — the TARGET slot also accepts a parametric-type FORM
    // (`(Head :- [args])`, the ANNOTATION migration's spelling) alongside the bare
    // (non-parametric) Keyword surface — angle brackets can never reach the Keyword arm at
    // all: the lexer refuses `<` inside a keyword token outright (arc 109 "annihilate the
    // angle bracket"). Both the bare Keyword and the FORM go through the SAME door —
    // `parse_type_node` — then rendered back through `check::format_type`, the ONE
    // authoritative TypeExpr renderer (already crossing this exact runtime.rs/check.rs
    // boundary elsewhere, e.g. line 8146) — so `type_name` stays the FULL identity string
    // (base + args), byte-identical to what the Keyword arm always produced. Dropping to a
    // base-only name here would silently starve `is_subtype`'s exact-string edge lookup
    // (types.rs's `register_subtype`) and the `transport_edge_keys` guess-set, both of which
    // key on the full `(Head :- [T])`/`(Head :- [Wire])` spelling verbatim.
    // Arc 109 ③ — keep the STRUCTURED `TypeExpr` alongside the rendered `type_name` string
    // (below, `ExtendDef::type_te`) so a consumer needing the target's structure (self's
    // param type at each impl method, `register_extend_type_surface_impls`) never has to
    // re-parse `type_name`'s angle-bracket string — the exact spelling this stone's wall
    // refuses. The Keyword arm parses too now (`.ok()`, best-effort: a non-parametric
    // keyword like `:t::Robot` always parses; only a malformed one falls back to `None`).
    let (type_name, type_te) = match &items[1] {
        WatAST::Keyword(k, _) => (k.clone(), crate::types::parse_type_node(&items[1]).ok()),
        node @ WatAST::List(_, _) => {
            let te = crate::types::parse_type_node(node).map_err(|e| {
                RuntimeError::new(
                    node.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: HEAD.into(),
                        reason: format!("extend-type first arg type form: {e}"),
                    },
                )
            })?;
            let raw = crate::check::format_type(&te);
            (raw, Some(te))
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "extend-type first arg must be a keyword or type-form type name; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    // Arc 170 C2 — split a parametric protocol/surface target (e.g. `(:Holds :- [wat::core::i64])`
    // — the ONE live spelling; a keyword can never carry `<...>`, the lexer refuses it) into
    // the BARE name (`:probe::Holds`, matching how the surface is registered in
    // `TypeEnv`) plus the concrete type args (`[Path(":wat::core::i64")]`). A plain
    // (non-generic) target (`:t::Greeter`) parses to `TypeExpr::Path` — `protocol_name`
    // is unchanged, `protocol_type_args` is empty (the monomorphic no-op path).
    // Parse failure (should not happen for a well-formed keyword) falls back to the raw
    // keyword verbatim — preserves the prior behavior rather than fabricating a split.
    //
    // Arc 109 identity 2c remainder — the SATISFIED-SURFACE slot also accepts the `:-` FORM.
    // Both routes (Keyword string via `parse_type_expr`, List via `parse_type_node`) converge
    // on the SAME `TypeExpr`, then the SAME match below splits it — one splitter, two doors in,
    // so a List input keeps its `protocol_type_args` (no base-fqdn drop: `register_extend_type_
    // surface_impls`'s `surface_type_subst` needs the real args to substitute the surface's own
    // `<T>` in each impl's inherited signature).
    // `protocol_name_raw` is only meaningful for the Keyword arm's own fallback (a keyword that
    // fails to re-parse as a type expr — should not happen for well-formed input — falls back to
    // itself verbatim, preserving the prior behavior rather than fabricating a split). The List
    // arm has no analogous "raw string": a malformed List already errored out above via `?`.
    let (protocol_te, protocol_name_raw) = match &items[2] {
        WatAST::Keyword(k, _) => (crate::types::parse_type_expr(k).ok(), k.clone()),
        node @ WatAST::List(_, _) => {
            let te = crate::types::parse_type_node(node).map_err(|e| {
                RuntimeError::new(
                    node.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: HEAD.into(),
                        reason: format!("extend-type second arg type form: {e}"),
                    },
                )
            })?;
            let raw = crate::check::format_type(&te);
            (Some(te), raw)
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "extend-type second arg must be a keyword or type-form protocol name; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    let (protocol_name, protocol_type_args) = match protocol_te {
        Some(crate::types::TypeExpr::Parametric { head, args }) => {
            (crate::types::parametric_head_fqdn(&head), args)
        }
        Some(crate::types::TypeExpr::Path(p)) => (p, Vec::new()),
        _ => (protocol_name_raw, Vec::new()),
    };

    let mut impl_clauses: std::collections::HashMap<String, crate::value::Clause> =
        std::collections::HashMap::new();
    for impl_form in &items[3..] {
        let impl_span = impl_form.span().clone();
        let impl_items = match impl_form {
            WatAST::List(items, _) => items,
            other => {
                return Err(RuntimeError::new(
                    impl_span,
                    RuntimeErrorKind::MalformedForm {
                        head: HEAD.into(),
                        reason: format!(
                            "each method impl must be a list `(method-name [args] body)`; got {}",
                            other.variant_name()
                        ),
                    },
                ))
            }
        };
        if impl_items.len() < 3 {
            return Err(RuntimeError::new(
                impl_span,
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "method impl must have at least 3 elements `(name [args] body)`; got {}",
                        impl_items.len()
                    ),
                },
            ));
        }
        let method_name = match &impl_items[0] {
            WatAST::Symbol(s, _) => {
                // STONE reap-the-angle-machinery (arc 109) — Stone 6b-DEP used to strip a
                // `<T>` suffix from the impl method name here. A Symbol carrying `<` is a
                // LEXER error now (verified directly), so `s` can never carry one; use it
                // directly.
                s.as_str().to_owned()
            }
            other => {
                return Err(RuntimeError::new(
                    other.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: HEAD.into(),
                        reason: format!(
                            "method impl first element must be a Symbol method name; got {}",
                            other.variant_name()
                        ),
                    },
                ))
            }
        };
        // Parse the impl as a defclause clause. The argspec in extend-type impls uses
        // bare binders WITHOUT type annotations (self, loudness — no `<-` triples).
        // `parse_defclause_clause` uses `parse_argspec_triples` which requires `<-` triples.
        //
        // For extend-type impls we use a simplified parse: collect bare symbol params
        // from the argvec (no type annotations), use :wat::core::nil as placeholder types
        // (the real types come from the surface member sig at dispatch time).
        let argvec_items = match &impl_items[1] {
            WatAST::Vector(v, _) => v,
            other => {
                return Err(RuntimeError::new(
                    other.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: HEAD.into(),
                        reason: format!(
                            "method impl `{}` second element must be an args Vector; got {}",
                            method_name,
                            other.variant_name()
                        ),
                    },
                ))
            }
        };
        // Collect bare symbol param names. extend-type impls do NOT carry type annotations
        // (the types live in the surface member sig). Each arg must be a bare Symbol.
        // Arc 293.4e-pre: build an ArgSpec (Identifier-keyed) instead of Vec<(String,TypeExpr)>.
        let mut fixed_params: Vec<(crate::scope::Identifier, crate::types::TypeExpr)> = Vec::new();
        for arg_item in argvec_items {
            match arg_item {
                WatAST::Symbol(s, _) => {
                    // Placeholder type — 232.3 dispatch resolves real types from the protocol sig.
                    fixed_params.push((s.clone(), crate::types::TypeExpr::Path(":wat::core::nil".into())));
                }
                other => return Err(RuntimeError::new(other.span().clone(), RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "method impl `{}` arg must be a bare Symbol (no type annotation in extend-type); got {}",
                        method_name, other.variant_name()
                    )
                })),
            }
        }
        let args = crate::argspec::ArgSpec {
            fixed_params,
            rest_param: None,
        };
        // Body: items[2..] — synthesize a multi-form body if needed.
        let body_items: Vec<WatAST> = impl_items[2..].to_vec();
        if body_items.is_empty() {
            return Err(RuntimeError::new(
                impl_span,
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!("method impl `{}` must have a body expression", method_name),
                },
            ));
        }
        // Arc 293.4c — strip optional `-> :RetType` from the body so that surface
        // extend-type impls like `(tag [self] -> :wat::core::i64 42)` work correctly.
        // Protocol impls don't use `->` so this is a no-op for them.
        // If body_items starts with Symbol("->") followed by a Keyword, strip them and
        // capture the return type; otherwise use :nil as the placeholder (protocol path).
        let (body_forms, clause_return_type) = if body_items.len() >= 3
            && matches!(&body_items[0], WatAST::Symbol(arrow, _) if arrow.as_str() == "->")
        {
            match &body_items[1] {
                WatAST::Keyword(ret_kw, _) => {
                    let ret = crate::types::parse_type_expr(ret_kw)
                        .unwrap_or_else(|_| crate::types::TypeExpr::Path(":wat::core::nil".into()));
                    (body_items[2..].to_vec(), ret)
                }
                // Arc 109 Stone ⑥ — the method-member RETURN slot also accepts the `:-`
                // reference FORM (`(Head :- [T …])`), through the same `parse_type_node` door
                // the sibling extend-type slots above (items[1]'s type_name, items[2]'s
                // protocol_te) already use. Unlike the Keyword arm's `unwrap_or_else(nil)`
                // fallback (kept byte-identical — it preserves prior behavior for a keyword
                // that somehow fails to re-parse, which should not happen for well-formed
                // input), a malformed List here PROPAGATES its parse error via `?` rather than
                // silently defaulting to `:nil`: a List only lands in this arm when the source
                // is unambiguously the `(Head :- [...])` form, so a parse failure is a real
                // malformed annotation, not a benign non-annotation shape — masking it as
                // `:nil` would hide exactly the defect class this arc has spent the day
                // digging out.
                node @ WatAST::List(_, _) => {
                    let te = crate::types::parse_type_node(node).map_err(|e| {
                        RuntimeError::new(
                            node.span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: HEAD.into(),
                                reason: format!(
                                    "method impl `{}` return type after `->`: {}",
                                    method_name, e
                                ),
                            },
                        )
                    })?;
                    (body_items[2..].to_vec(), te)
                }
                _ => (
                    body_items,
                    crate::types::TypeExpr::Path(":wat::core::nil".into()),
                ),
            }
        } else {
            (
                body_items,
                crate::types::TypeExpr::Path(":wat::core::nil".into()),
            )
        };
        if body_forms.is_empty() {
            return Err(RuntimeError::new(
                impl_span,
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "method impl `{}` must have a body expression after `-> :T`",
                        method_name
                    ),
                },
            ));
        }
        let body_ast = synthesize_fn_body(&body_forms);
        let clause = crate::value::Clause {
            args,
            return_type: clause_return_type,
            guard: None,
            ensure_fn: None,
            body: Arc::new(body_ast),
            // extend-type methods are registered as real Functions in `sym.functions`
            // (see `register_extend_type_methods`), so they already tail-call and never
            // dispatch through `eval_call_to_defclause_with_vals`.
            func: None,
        };
        impl_clauses.insert(method_name, clause);
    }

    let canonical_key = format!("extend:{}:{}", protocol_name, type_name);
    let ed = Arc::new(crate::value::ExtendDef {
        type_name,
        type_te,
        protocol_name,
        protocol_type_args,
        impl_clauses,
    });
    Ok((canonical_key, ed))
}

/// Arc 237 follow-on — parse a `(:wat::core::derive :Child :Parent)` form.
///
/// Returns `(child, parent)` keyword strings. Shape: exactly 3 items;
/// items[1] = :Child keyword, items[2] = :Parent keyword. No method-impl
/// loop — `derive` is the edge-only half of `extend-type`.
pub(crate) fn parse_derive_form(form: &WatAST) -> Result<(String, String), RuntimeError> {
    const HEAD: &str = ":wat::core::derive";
    let form_span = form.span().clone();
    let items = match form {
        WatAST::List(items, _) => items,
        _ => {
            return Err(RuntimeError::new(
                form_span,
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: "expected list".into(),
                },
            ))
        }
    };
    // items[0] = :wat::core::derive
    // items[1] = :Child keyword
    // items[2] = :Parent keyword
    if items.len() != 3 {
        return Err(RuntimeError::new(
            form_span,
            RuntimeErrorKind::MalformedForm {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::derive :Child :Parent); got {} elements",
                    items.len()
                ),
            },
        ));
    }
    let child = match &items[1] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "derive first arg must be a keyword child type name; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    let parent = match &items[2] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: HEAD.into(),
                    reason: format!(
                        "derive second arg must be a keyword parent type name; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };
    Ok((child, parent))
}

// ─── end Arc 232 Stone 232.1 parse fns ───────────────────────────────────────

// ⛔ `is_defclause_form` DELETED here, not relocated. It arrived with this stone and the
// compiler immediately reported it unused — it had ZERO callers at HEAD too (its single
// occurrence in runtime.rs was its own definition). It never warned there because it was
// `pub` inside `pub mod runtime`, so rustc treats it as reachable API; narrowing it to
// `pub(crate)` for this home made the dead-code analysis apply for the first time. Second
// instance of that class this campaign (see `Binding`'s fields, arc 109 the-reflect-home).
// Deleted rather than exempted: a home being FILLED OUT must not be seeded with a
// graveyard, and unlike a type's field list this costs no contract — nothing calls it.

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
