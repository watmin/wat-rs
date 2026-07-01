//! Arc 296 — EDN serializers for [`CheckError`] and [`CheckErrors`].
//!
//! Extends arc 233's `runtime_error_to_edn` + arc 296.2's `macro_error_to_edn`
//! pattern upward through the check pass. Each error variant serializes as a
//! tagged EDN envelope in the `wat.kernel` namespace:
//!
//! ```text
//! #wat.kernel/TypeMismatch {:callee ":user::greet" :param "name"
//!                           :expected ":wat::core::String" :got ":wat::core::i64"
//!                           :span {:file "user.wat" :line 10 :col 5}}
//! ```
//!
//! ## Tag convention
//!
//! `#wat.kernel/<VariantName>` — the variant name from `CheckErrorKind` is
//! the tag discriminator. The outer struct's span is included as `:span` when
//! it is not `Span::unknown()`.
//!
//! ## Field naming
//!
//! Single-word field names keep their name (`:callee`, `:expected`, `:got`).
//! Multi-word snake_case field names from the Rust struct are translated to
//! kebab-case (`:thread-binding`, `:process-identifier`). This mirrors the
//! EDN idiom used throughout `runtime_error_edn.rs` and `macros/error_edn.rs`.

use std::borrow::Cow;
use wat_edn::{Keyword, OwnedValue, Tag};

use crate::span::Span;
use super::error::{CheckError, CheckErrorKind};

// ─── Public API ──────────────────────────────────────────────────────────────

/// Serialize a [`CheckError`] to a tagged [`OwnedValue`].
///
/// The outer struct's span is included under `:span` for variants whose
/// `diagnostic()` used `loc_field(diag, "span", span)`. Variants that used
/// `"location"` or two-field span annotations preserve their original
/// field naming.
///
/// Secondary spans (e.g. `output_accessor_span`) are included only when they
/// are not `Span::unknown()`.
pub fn check_error_to_edn(err: &CheckError) -> OwnedValue {
    let span = &err.span;
    match &err.kind {
        CheckErrorKind::ArityMismatch { callee, expected, got } => {
            let mut fields = vec![
                (kw("callee"), str_val(callee)),
                (kw("expected"), int_val(*expected)),
                (kw("got"), int_val(*got)),
            ];
            push_span(&mut fields, "span", span);
            tagged("ArityMismatch", OwnedValue::Map(fields))
        }

        CheckErrorKind::TypeMismatch { callee, param, expected, got } => {
            let mut fields = vec![
                (kw("callee"), str_val(callee)),
                (kw("param"), str_val(param)),
                (kw("expected"), str_val(expected)),
                (kw("got"), str_val(got)),
            ];
            // Arc 296 remediation collapse: structured :remedies field; prose hint annihilated.
            fields.push((kw("remedies"), crate::remedy::remedies_to_edn(
                &super::type_error_remedies(callee, expected, got),
            )));
            push_span(&mut fields, "span", span);
            tagged("TypeMismatch", OwnedValue::Map(fields))
        }

        CheckErrorKind::ReturnTypeMismatch { function, expected, got, remedies } => {
            let mut fields = vec![
                (kw("function"), str_val(function)),
                (kw("expected"), str_val(expected)),
                (kw("got"), str_val(got)),
            ];
            // Arc 296 remediation collapse: merge stored remedies with computed type_error_remedies,
            // dedup by form (stored leads — retirement-table hits first), prose hint annihilated.
            let mut merged: Vec<crate::remedy::Remedy> = remedies.clone();
            merged.extend(super::type_error_remedies(function, expected, got));
            let mut seen = std::collections::HashSet::new();
            merged.retain(|r| seen.insert(r.form.clone()));
            fields.push((kw("remedies"), crate::remedy::remedies_to_edn(&merged)));
            push_span(&mut fields, "span", span);
            tagged("ReturnTypeMismatch", OwnedValue::Map(fields))
        }

        CheckErrorKind::UnknownCallee { callee } => {
            let mut fields = vec![(kw("callee"), str_val(callee))];
            push_span(&mut fields, "span", span);
            tagged("UnknownCallee", OwnedValue::Map(fields))
        }

        CheckErrorKind::MalformedForm { head, reason, remedies } => {
            let mut fields = vec![
                (kw("head"), str_val(head)),
                (kw("reason"), str_val(reason)),
            ];
            // Arc 296 D1: remedies travel as a structured Vector, never a prose blob.
            fields.push((kw("remedies"), crate::remedy::remedies_to_edn(remedies)));
            push_span(&mut fields, "span", span);
            tagged("MalformedForm", OwnedValue::Map(fields))
        }

        CheckErrorKind::CommCallOutOfPosition { callee } => {
            let mut fields = vec![(kw("callee"), str_val(callee))];
            push_span(&mut fields, "span", span);
            tagged("CommCallOutOfPosition", OwnedValue::Map(fields))
        }

        CheckErrorKind::ScopeDeadlock { thread_binding, offending_binding, offending_kind } => {
            let mut fields = vec![
                (kw("thread-binding"), str_val(thread_binding)),
                (kw("offending-binding"), str_val(offending_binding)),
                (kw("offending-kind"), str_val(offending_kind)),
            ];
            push_span(&mut fields, "location", span);
            tagged("ScopeDeadlock", OwnedValue::Map(fields))
        }

        CheckErrorKind::ProcessJoinBeforeOutputDrain {
            process_identifier,
            output_accessor,
            output_accessor_span,
        } => {
            let mut fields = vec![
                (kw("process-identifier"), str_val(process_identifier)),
                (kw("output-accessor"), str_val(output_accessor)),
            ];
            push_span(&mut fields, "join-location", span);
            push_span(&mut fields, "output-location", output_accessor_span);
            tagged("ProcessJoinBeforeOutputDrain", OwnedValue::Map(fields))
        }

        CheckErrorKind::ProcessJoinHoldsStdinSender { process_identifier, stdin_sender_span } => {
            let mut fields = vec![(kw("process-identifier"), str_val(process_identifier))];
            push_span(&mut fields, "join-location", span);
            push_span(&mut fields, "bind-location", stdin_sender_span);
            tagged("ProcessJoinHoldsStdinSender", OwnedValue::Map(fields))
        }

        CheckErrorKind::ChannelPairDeadlock { callee, sender_arg, receiver_arg, pair_anchor } => {
            let mut fields = vec![
                (kw("callee"), str_val(callee)),
                (kw("sender-arg"), str_val(sender_arg)),
                (kw("receiver-arg"), str_val(receiver_arg)),
                (kw("pair-anchor"), str_val(pair_anchor)),
            ];
            push_span(&mut fields, "location", span);
            tagged("ChannelPairDeadlock", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyPrimitive { primitive, fqdn } => {
            let mut fields = vec![
                (kw("primitive"), str_val(primitive)),
                (kw("fqdn"), str_val(fqdn)),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyPrimitive", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyUnitType => {
            let mut fields = vec![
                (kw("primitive"), str_val(":()")),
                (kw("fqdn"), str_val(":wat::core::nil")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyUnitType", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyUnitName => {
            let mut fields = vec![
                (kw("retired"), str_val(":wat::core::unit")),
                (kw("fqdn"), str_val(":wat::core::nil")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyUnitName", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyLetStar => {
            let mut fields = vec![
                (kw("retired"), str_val(":wat::core::let*")),
                (kw("fqdn"), str_val(":wat::core::let")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyLetStar", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyLambda => {
            let mut fields = vec![
                (kw("retired"), str_val(":wat::core::lambda")),
                (kw("fqdn"), str_val(":wat::core::fn")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyLambda", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyLowercaseFn => {
            let mut fields = vec![
                (kw("retired"), str_val(":fn(...)->ret")),
                (kw("fqdn"), str_val(":wat::core::Fn(...)->ret")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyLowercaseFn", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyContainerHead { head, fqdn } => {
            let mut fields = vec![
                (kw("head"), str_val(head)),
                (kw("fqdn"), str_val(fqdn)),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyContainerHead", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyStreamPath { old, new } => {
            let mut fields = vec![
                (kw("old"), str_val(old)),
                (kw("new"), str_val(new)),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyStreamPath", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyTelemetryServicePath { old, new } => {
            let mut fields = vec![
                (kw("old"), str_val(old)),
                (kw("new"), str_val(new)),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyTelemetryServicePath", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyLruCacheServicePath { old, new } => {
            let mut fields = vec![
                (kw("old"), str_val(old)),
                (kw("new"), str_val(new)),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyLruCacheServicePath", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyKernelQueuePath { old, new } => {
            let mut fields = vec![
                (kw("old"), str_val(old)),
                (kw("new"), str_val(new)),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyKernelQueuePath", OwnedValue::Map(fields))
        }

        CheckErrorKind::SandboxScopeLeak { offending_name, outer_define_span } => {
            let mut fields = vec![(kw("offending-name"), str_val(offending_name))];
            push_span(&mut fields, "call-span", span);
            push_span(&mut fields, "outer-define-span", outer_define_span);
            tagged("SandboxScopeLeak", OwnedValue::Map(fields))
        }

        CheckErrorKind::DefRedefForbidden { name, original_def_span } => {
            let mut fields = vec![(kw("name"), str_val(name))];
            push_span(&mut fields, "prior-loc", original_def_span);
            push_span(&mut fields, "current-loc", span);
            tagged("DefRedefForbidden", OwnedValue::Map(fields))
        }

        CheckErrorKind::DefRedefTypeChange { name, prior_type, new_type, original_def_span } => {
            let mut fields = vec![
                (kw("name"), str_val(name)),
                (kw("prior-type"), str_val(prior_type)),
                (kw("new-type"), str_val(new_type)),
            ];
            push_span(&mut fields, "prior-loc", original_def_span);
            push_span(&mut fields, "current-loc", span);
            tagged("DefRedefTypeChange", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyMainSignature => {
            let mut fields = vec![
                (kw("canonical-signature"), str_val("[] -> :wat::core::nil")),
                (kw("rationale"), str_val(
                    "arc 170 slice 1e (REALIZATIONS pass 7 + pass 10): argv ambient via (:wat::runtime::argv); stdio via three substrate services (slice 1f); nil IS the success exit code",
                )),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyMainSignature", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyForkProgram { verb } => {
            let mut fields = vec![
                (kw("retired"), str_val(verb)),
                (kw("canonical"), str_val(":wat::kernel::spawn-process")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyForkProgram", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacySpawnProgram { verb } => {
            let mut fields = vec![
                (kw("retired"), str_val(verb)),
                (kw("canonical-fork-semantics"), str_val(":wat::kernel::spawn-process")),
                (kw("canonical-thread-semantics"), str_val(":wat::kernel::spawn-thread")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacySpawnProgram", OwnedValue::Map(fields))
        }

        CheckErrorKind::BareLegacyConsolePath { path } => {
            let mut fields = vec![
                (kw("retired-namespace"), str_val(":wat::console::*")),
                (kw("offending-token"), str_val(path)),
                (kw("canonical-stdout"), str_val(":wat::kernel::println")),
                (kw("canonical-stderr"), str_val(":wat::kernel::eprintln")),
                (kw("canonical-stdin"), str_val(":wat::kernel::readln")),
            ];
            push_span(&mut fields, "location", span);
            tagged("BareLegacyConsolePath", OwnedValue::Map(fields))
        }

        CheckErrorKind::DefRestrictedCallerNotAllowed { callee, enclosing_fn, prefixes } => {
            // Arc 296 D1: prefixes travel as a Vector, never a space-joined prose blob.
            let prefixes_edn = OwnedValue::Vector(prefixes.iter().map(|s| str_val(s)).collect());
            let mut fields = vec![
                (kw("callee"), str_val(callee)),
                (kw("enclosing-fn"), str_val(enclosing_fn)),
                (kw("prefixes"), prefixes_edn),
            ];
            push_span(&mut fields, "location", span);
            tagged("DefRestrictedCallerNotAllowed", OwnedValue::Map(fields))
        }

        CheckErrorKind::NoMatchingClauseAtCallSite {
            name,
            called_arity,
            called_arg_types,
            attempted_clauses,
        } => {
            // Arc 296 D1: called-arg-types → Vector; attempted-clauses no longer dropped.
            let called_arg_types_edn = OwnedValue::Vector(
                called_arg_types.iter().map(|s| str_val(s)).collect(),
            );
            // Each element is (arity: usize, param_types: Vec<String>).
            // Mirrors runtime's clause_attempt_to_edn shape: {:arity N :param-types [str …]}.
            let attempted_clauses_edn = OwnedValue::Vector(
                attempted_clauses.iter().map(|(arity, param_types)| {
                    OwnedValue::Map(vec![
                        (kw("arity"), OwnedValue::Integer(*arity as i64)),
                        (kw("param-types"), OwnedValue::Vector(
                            param_types.iter().map(|s| str_val(s)).collect(),
                        )),
                    ])
                }).collect(),
            );
            let mut fields = vec![
                (kw("name"), str_val(name)),
                (kw("called-arity"), int_val(*called_arity)),
                (kw("called-arg-types"), called_arg_types_edn),
                (kw("attempted-clauses"), attempted_clauses_edn),
            ];
            push_span(&mut fields, "span", span);
            tagged("NoMatchingClauseAtCallSite", OwnedValue::Map(fields))
        }

        CheckErrorKind::GuardExprNotBoolean { defclause_name, clause_index, got_type } => {
            let mut fields = vec![
                (kw("defclause-name"), str_val(defclause_name)),
                (kw("clause-index"), int_val(*clause_index)),
                (kw("got-type"), str_val(got_type)),
            ];
            push_span(&mut fields, "span", span);
            tagged("GuardExprNotBoolean", OwnedValue::Map(fields))
        }

        CheckErrorKind::EnsureFnInvalid { defclause_name, clause_index, reason } => {
            let mut fields = vec![
                (kw("defclause-name"), str_val(defclause_name)),
                (kw("clause-index"), int_val(*clause_index)),
                (kw("reason"), str_val(reason)),
            ];
            push_span(&mut fields, "span", span);
            tagged("EnsureFnInvalid", OwnedValue::Map(fields))
        }

        CheckErrorKind::HygieneScopeDivergence { name, ref_key, binder_key } => {
            let mut fields = vec![
                (kw("name"), str_val(name)),
                (kw("ref-key"), str_val(ref_key)),
                (kw("binder-key"), str_val(binder_key)),
            ];
            push_span(&mut fields, "span", span);
            tagged("HygieneScopeDivergence", OwnedValue::Map(fields))
        }
    }
}

// ─── ToEdn + WatError impls ──────────────────────────────────────────────────

impl crate::to_edn::ToEdn for CheckError {
    fn to_edn(&self) -> OwnedValue {
        check_error_to_edn(self)
    }
}

impl crate::to_edn::WatError for CheckError {
    /// Concise single-line headline: the span-free kind Display's first line
    /// (no `file:line` prefix, no multi-line hint/remedy sections — those live
    /// in `:location` and the structured variant fields).
    fn message(&self) -> String {
        crate::to_edn::first_line(self.kind.to_string())
    }
    fn location(&self) -> OwnedValue {
        crate::to_edn::location_from_span(&self.span)
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> OwnedValue {
        crate::to_edn::strip_span_from_tagged(check_error_to_edn(self))
    }
}

impl crate::to_edn::ToEdn for super::error::CheckErrors {
    /// `#wat.kernel/CheckErrors {:errors [#wat.kernel/<Variant> {…} …]}` —
    /// each `CheckError` in the collection is a navigable tagged value, not a
    /// line in a `:detail` prose blob. This is the structured form the
    /// process-boundary IPC path and `--check-output` consumers read.
    fn to_edn(&self) -> OwnedValue {
        let items: Vec<OwnedValue> = self.0.iter().map(check_error_to_edn).collect();
        tagged(
            "CheckErrors",
            OwnedValue::Map(vec![(kw("errors"), OwnedValue::Vector(items))]),
        )
    }
}

impl crate::to_edn::WatError for super::error::CheckErrors {
    /// Concise COLLECTION summary — a count, NOT the concatenated multi-line
    /// render of every item. Each item carries its own single-line `:message`
    /// inside the recursively-floored `:errors` array, so re-rendering them
    /// here would double-encode the exact content the floor already holds.
    fn message(&self) -> String {
        let n = self.0.len();
        format!("{} type-check error{}", n, if n == 1 { "" } else { "s" })
    }
    /// `CheckErrors` is a collection; no single primary span exists at this
    /// level. Individual `CheckError` items carry their own `:location`.
    fn location(&self) -> OwnedValue {
        OwnedValue::Nil
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    /// Arc 296 strike 2 — RECURSIVE floor: each `CheckError` in `:errors` is
    /// embedded via its `WatError::error_edn()` (floor form: single-line
    /// `:message`, `:location` never `:span`), NOT its raw `to_edn()`. The
    /// collection envelope itself carries no top-level `:span`.
    fn variant(&self) -> OwnedValue {
        let items: Vec<OwnedValue> = self.0.iter().map(|e| e.error_edn()).collect();
        tagged(
            "CheckErrors",
            OwnedValue::Map(vec![(kw("errors"), OwnedValue::Vector(items))]),
        )
    }
}

// ─── Low-level builders (mirrors runtime_error_edn.rs) ───────────────────────

fn tagged(variant: &str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns("wat.kernel", variant), Box::new(body))
}

fn kw(name: &str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

fn str_val(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}

fn int_val(n: usize) -> OwnedValue {
    OwnedValue::Integer(n as i64)
}

/// Append a span field to the fields vec, but ONLY when the span is known.
/// Unknown spans are elided — the same as `loc_field` in `diagnostic()`.
fn push_span(fields: &mut Vec<(OwnedValue, OwnedValue)>, key: &str, span: &Span) {
    if !span.is_unknown() {
        fields.push((kw(key), crate::panic_hook::span_to_edn(span)));
    }
}
