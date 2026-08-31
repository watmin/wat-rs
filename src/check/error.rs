//! Check-pass error types — Pattern A (Stone 243.6a) home.
//!
//! [`CheckError`] carries the source span at the outer struct; variant data
//! lives in [`CheckErrorKind`]. The `span` field is mandatory at construction —
//! Rust's struct-literal rule makes a span-less `CheckError` uncompilable.
//! `crate::rust_caller_span!()` is the explicit sentinel for the rare site with no
//! recoverable source location; `Display`/`diagnostic()` elide unknown spans.
//!
//! vigilatum: 2026-06-01T19:18:06Z — vigilia 7-spell L1+L2=0

use crate::span::{span_prefix, Span};
use std::fmt;

/// Type-check errors. Pattern A (Stone 243.6a): span at the outer struct
/// level; variant data in `CheckErrorKind`.
///
/// The `span` field is mandatory at construction — Rust's struct-literal rule
/// makes a span-less `CheckError` uncompilable. `crate::rust_caller_span!()` is the
/// explicit sentinel for the rare site with no recoverable source location;
/// `Display`/`diagnostic()` elide unknown spans.
///
/// Multiple errors accumulate in a single pass so users get one batch of findings.
#[derive(Clone)]
pub struct CheckError {
    pub span: Span,
    pub kind: CheckErrorKind,
}

/// Arc 296 stone I — the taxonomy conversion `resolve::register`'s `?` performs at every
/// check-side call site. `Rejection::verdict` is never `Insert`/`NoOp` (see its doc), so
/// those two arms are unreachable by construction.
impl From<crate::resolve::Rejection> for CheckError {
    fn from(r: crate::resolve::Rejection) -> Self {
        use crate::resolve::Registration;
        let kind = match r.verdict {
            Registration::Duplicate => CheckErrorKind::DuplicateScheme { name: r.name },
            Registration::Reserved => CheckErrorKind::ReservedPrefix { name: r.name },
            Registration::Unnamespaced => CheckErrorKind::UnnamespacedName { name: r.name },
            Registration::DottedName => CheckErrorKind::DottedName { name: r.name },
            Registration::Insert | Registration::NoOp => {
                unreachable!("resolve::register never rejects with Insert/NoOp")
            }
        };
        CheckError { span: r.span, kind }
    }
}

/// `CheckErrorKind` (168 bytes, unboxed — unlike `RuntimeErrorKind`/`TypeErrorKind`/
/// `MacroErrorKind`, which box their kind specifically to duck this) trips
/// clippy's `result_large_err` the moment a bare `Result<_, CheckError>` appears in a
/// signature, which a couple of `resolve::register` call sites now do. Boxing AT THOSE
/// TWO SITES (not restructuring `CheckError` itself — out of this stone's blast radius)
/// is what this impl is for.
impl From<crate::resolve::Rejection> for Box<CheckError> {
    fn from(r: crate::resolve::Rejection) -> Self {
        Box::new(CheckError::from(r))
    }
}

/// Variant data for [`CheckError`]. Spans live in the outer struct; variants
/// carry ONLY data unique to each failure kind.
///
/// **Multi-span variants** keep their SECONDARY spans as domain-named kind
/// fields per CONFORMARE.md § Multi-span. The outer `span` is the
/// most-actionable location (the site the user edits to fix).
///
/// Arc 296 Strike 2b: `#[derive(ToEdn)]` generates the `impl ToEdn for CheckErrorKind`
/// body structurally from the Rust type. The outer `CheckError` wrapper calls
/// `splice_span(self.kind.to_edn(), &self.span)` to append `:span` uniformly
/// (D1: primary span key normalized to `:span` across all variants).
///
/// ## Attribute DSL used here
///
/// - `#[to_edn(via(key="remedies", fn=..., args(...)))]` on a variant: computed
///   field appended after field pairs (elide-on-None; always-Some → always-emit).
/// - `#[to_edn(literal(k="v",...))]` on a variant: synthetic constant fields prepended.
/// - `#[to_edn(key="edn-key")]` on a field: override the default snake→kebab EDN key.
/// - `#[to_edn(via = path)]` on a field: call `path(field)` instead of `field.to_edn()`.
///   Required for fields whose type has no `ToEdn` impl (`Vec<(usize, Vec<String>)>`).
///
/// Secondary `Span`-typed fields use `#[to_edn(key="domain-key")]` to preserve
/// their domain names (e.g. `"output-location"`, `"bind-location"`). Primary spans
/// are never in the kind enum — they live on the outer `CheckError.span` field and
/// are spliced in uniformly by `impl ToEdn for CheckError`.
#[derive(Debug, Clone, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::CHECK)]
pub enum CheckErrorKind {
    /// Arc 138 slice 1 — arity mismatch at a call site.
    ArityMismatch {
        callee: String,
        expected: usize,
        got: usize,
    },
    /// Arc 138 slice 1 — type mismatch at a call-site parameter.
    ///
    /// `:remedies` is computed at serialization time via `type_error_remedies`
    /// (retirement-table + shape-specific candidates). Always emitted, even when
    /// the list is empty (matches the golden `check_error_to_edn` always-push behavior).
    #[to_edn(via(key = "remedies", fn = crate::check::type_error_remedies_via, args(callee)))]
    TypeMismatch {
        callee: String,
        param: String,
        expected: String,
        got: String,
    },
    /// Arc 138 slice 1 — function body type does not match declared return type.
    ///
    /// `remedies` stores the typo-based candidates from `variant_typo_remedies`
    /// at construction time. The golden `check_error_to_edn` merged these with
    /// `type_error_remedies(function)` at serialization time
    /// (DESIGN-296-remediation-collapse line 32). The derive preserves that
    /// merge via a variant-level `via`: `return_type_remedies_via` folds the
    /// stored field into the computed retirement/shape candidates (dedup by
    /// `.form`). The `remedies` field is `#[to_edn(skip)]` so it does NOT also
    /// serialize plainly — the `via` OWNS the `:remedies` key (no duplicate).
    #[to_edn(via(key = "remedies", fn = crate::check::return_type_remedies_via, args(remedies, function)))]
    ReturnTypeMismatch {
        function: String,
        expected: String,
        got: String,
        /// Stone 241.10 — ranked structured remediation candidates.
        /// Empty vec = no remedy. Per `feedback_no_semantic_abuse_of_option`:
        /// `Vec<Remedy>` not `Option<Vec<Remedy>>`.
        ///
        /// Skipped from plain serialization: the variant-level `via` above
        /// merges this stored field with `type_error_remedies(callee)` and emits
        /// the union under `:remedies`. Emitting it plainly too would produce a
        /// duplicate, wrong `:remedies` key.
        #[to_edn(skip)]
        remedies: Vec<crate::remedy::Remedy>,
    },
    /// Arc 138 slice 1 — unknown callee at a call site.
    UnknownCallee { callee: String },
    /// A built-in form is structurally malformed in a way the syntax-level
    /// grammar doesn't catch.
    MalformedForm {
        head: String,
        reason: String,
        /// Stone 241.10 — ranked structured remediation candidates.
        /// Empty vec = no remedy.
        remedies: Vec<crate::remedy::Remedy>,
    },
    /// Arc 110 — a comm-call appeared outside every permitted position.
    ///
    /// Permitted positions (per `src/check.rs` `validate_comm_positions`):
    /// - scrutinee of `:wat::core::match` (`MatchScrutinee`)
    /// - value-position of `:wat::core::Result/expect` (`ResultExpectValue`)
    /// - value-position of `:wat::core::Option/expect` (`OptionExpectValue`)
    /// - right-hand side of a `:wat::core::let` binding (`LetBindingRhs`)
    ///
    /// Applies to the thread-tier comm-call set:
    /// `:wat::kernel::send` / `:wat::kernel::recv`.
    /// (Arc 278 IPC de-prime: the process-tier comm verbs were annihilated;
    /// process IPC now flows through the `spawn-program' (process)` peer model
    /// via `send'`/`recv'`/`recv-all'`.)
    CommCallOutOfPosition { callee: String },
    /// Arc 170 — Process-output-channel join-before-drain rule.
    /// Outer span = join-result call site (most-actionable).
    /// Secondary: `output_accessor_span` = conflicting output accessor call.
    ///
    /// D1: primary span was `:join-location`; normalized to `:span`.
    ProcessJoinBeforeOutputDrain {
        process_identifier: String,
        output_accessor: String,
        /// Source location of the conflicting output accessor call.
        #[to_edn(key = "output-location")]
        output_accessor_span: Span,
    },
    /// Arc 202 — Process input-channel held-at-join rule.
    /// Outer span = join-result call site (most-actionable).
    /// Secondary: `stdin_sender_span` = where the process identifier was bound.
    ///
    /// D1: primary span was `:join-location`; normalized to `:span`.
    ProcessJoinHoldsStdinSender {
        process_identifier: String,
        /// Source location where `<process_identifier>` was bound.
        #[to_edn(key = "bind-location")]
        stdin_sender_span: Span,
    },
    /// Arc 109 slice 1c — bare primitive type in user code.
    ///
    /// D1: primary span was `:location`; normalized to `:span`.
    BareLegacyPrimitive { primitive: String, fqdn: String },
    /// Arc 109 slice 1d — bare unit type annotation in user code.
    ///
    /// D1: primary span was `:location`; normalized to `:span`.
    /// Synthetic constant fields `:primitive` and `:fqdn` replace the
    /// struct-less unit variant's hand-written pair list.
    #[to_edn(literal(primitive = ":()", fqdn = ":wat::core::nil"))]
    BareLegacyUnitType,
    /// Arc 179 — bare `()` empty-list literal in VALUE position retired.
    ///
    /// Mirrors `BareLegacyUnitType` (the type-position sibling, arc 109
    /// slice 1d) but at the value layer: `nil` is the sole unit value,
    /// so an empty list literal used as an expression is no longer a
    /// second spelling of it. `()` survives ONLY as empty-parameter-list
    /// SYNTAX (`Fn()->T`, `(() -> T)`), which never reaches this arm —
    /// the expression checker distinguishes value position from
    /// parameter-list syntax structurally, not textually.
    #[to_edn(literal(retired = "()", fqdn = ":wat::core::nil"))]
    BareLegacyUnitValue,
    /// Arc 153 — `:wat::core::unit` retired in favor of `:wat::core::nil`.
    #[to_edn(literal(retired = ":wat::core::unit", fqdn = ":wat::core::nil"))]
    BareLegacyUnitName,
    /// Arc 154 — `:wat::core::let*` retired.
    #[to_edn(literal(retired = ":wat::core::let*", fqdn = ":wat::core::let"))]
    BareLegacyLetStar,
    /// Arc 155 — `:wat::core::lambda` retired.
    #[to_edn(literal(retired = ":wat::core::lambda", fqdn = ":wat::core::fn"))]
    BareLegacyLambda,
    /// Arc 155 — bare `:fn(...)` type-position spelling retired.
    #[to_edn(literal(retired = ":fn(...)->ret", fqdn = ":wat::core::Fn(...)->ret"))]
    BareLegacyLowercaseFn,
    /// Arc 109 slice 1e — bare substrate-named parametric type head.
    BareLegacyContainerHead { head: String, fqdn: String },
    /// Arc 109 slice 9d — legacy `:wat::std::stream::` prefix.
    BareLegacyStreamPath { old: String, new: String },
    /// Arc 109 slice K.lru — legacy `:wat::lru::CacheService::` prefix.
    BareLegacyLruCacheServicePath { old: String, new: String },
    /// Arc 109 slice K.kernel-channel — legacy `:wat::kernel::Queue*` names.
    BareLegacyKernelQueuePath { old: String, new: String },
    /// Arc 157 — `:wat::core::def` redef forbidden.
    /// Outer span = the new (colliding) binding site (most-actionable).
    /// Secondary: `original_def_span` = source location of the prior binding.
    ///
    /// D1: primary span was `:current-loc`; normalized to `:span`.
    DefRedefForbidden {
        name: String,
        /// Source location of the prior (first) binding. Key `:prior-loc`.
        #[to_edn(key = "prior-loc")]
        original_def_span: Span,
    },
    /// Arc 157 slice 1a-ii — `:wat::core::def` redef changes type.
    /// Outer span = the new (colliding) binding site (most-actionable).
    /// Secondary: `original_def_span` = source location of the prior binding.
    ///
    /// D1: primary span was `:current-loc`; normalized to `:span`.
    DefRedefTypeChange {
        name: String,
        prior_type: String,
        new_type: String,
        /// Source location of the prior (first) binding. Key `:prior-loc`.
        #[to_edn(key = "prior-loc")]
        original_def_span: Span,
    },
    /// Arc 278 BRIEF-scalar-def-reaches-the-gate — a top-level `:wat::core::def`
    /// binding with no namespace on its name. `register_defines` (runtime.rs)
    /// only routes FN-SHAPED defs through `resolve::gate`; a plain scalar def
    /// falls to `extract_def_binding` / `collect_splice_defs_ctx` at check-time,
    /// which is now the door that calls `resolve::gate` for the shape the
    /// runtime-side door never sees. Fourth taxonomy entry for
    /// `Registration::Unnamespaced` — `TypeErrorKind`, `RuntimeErrorKind` and
    /// `MacroErrorKind` got theirs in `72a1ac3d`.
    UnnamespacedName { name: String },
    /// Arc 278 BRIEF-scalar-def-reaches-the-gate — a top-level `:wat::core::def`
    /// binding whose name uses a reserved prefix (the live list is
    /// `resolve::reserved::RESERVED_PREFIXES` — do NOT re-spell it here; it
    /// grew a third entry at arc 251.8a and this comment was stale within the
    /// hour) from
    /// unprivileged (user) source. Same door as `UnnamespacedName` above: a
    /// scalar def never reached `resolve::gate` before this change, so this
    /// hole was open too — found by grounding the BRIEF's `Reserved` question
    /// with a run, not closed as a silent bonus.
    ReservedPrefix { name: String },
    /// Arc 296 stone H-1 — the name (the segment after the last `::`) reached the
    /// registration gate containing a `.`. Same door as `UnnamespacedName` /
    /// `ReservedPrefix` above — fifth taxonomy entry for `Registration::DottedName`
    /// (`TypeErrorKind`, `RuntimeErrorKind`, `MacroErrorKind`, `CheckErrorKind`).
    /// Reserved because a dotted NAME is the wire discriminator for a tagged-enum
    /// variant (`#ns/Enum.Variant`); a record whose name contained a dot could forge it.
    DottedName { name: String },
    /// Arc 296 stone I — a name landed on `CheckEnv`'s overlay scheme table
    /// ([`crate::check::env::CheckEnv::register_overlay`]) DIVERGENT from what is already
    /// registered there: the same FQDN meaning two different signatures. Sixth taxonomy
    /// entry for `Registration::Duplicate` (`TypeErrorKind::DuplicateType`,
    /// `RuntimeErrorKind::DuplicateDefine`, `MacroErrorKind::DuplicateMacro` already had
    /// theirs) — the overlay table is the one registry `resolve::gate`'s `Duplicate`
    /// verdict was previously observed but never actually surfaced as more than a bare
    /// `eprintln!` (see `from_symbols`'s `GATE-REJECT` swallow, closed by this stone).
    DuplicateScheme { name: String },
    /// Arc 170 slice 1e — `:user::main` with non-canonical signature.
    ///
    /// D1: primary span was `:location`; normalized to `:span`.
    #[to_edn(literal(
        canonical_signature = "[] -> :wat::core::nil",
        rationale = "arc 170 slice 1e (REALIZATIONS pass 7 + pass 10): argv ambient via (:wat::runtime::argv); stdio via three substrate services (slice 1f); nil IS the success exit code"
    ))]
    BareLegacyMainSignature,
    /// Arc 109 § kill-std — retired `:wat::console::*` namespace.
    ///
    /// D1: primary span was `:location`; normalized to `:span`.
    /// `path` field emitted as `:offending-token`; remaining keys are synthetic literals.
    #[to_edn(literal(
        retired_namespace = ":wat::console::*",
        canonical_stdout = ":wat::kernel::println",
        canonical_stderr = ":wat::kernel::eprintln",
        canonical_stdin = ":wat::kernel::readln"
    ))]
    BareLegacyConsolePath {
        #[to_edn(key = "offending-token")]
        path: String,
    },
    /// Stone 241.14 — restricted-caller whitelist violation.
    ///
    /// D1: primary span was `:location`; normalized to `:span`.
    /// `prefixes: Vec<String>` serializes via `Vec<String>.to_edn()` = EDN Vector.
    DefRestrictedCallerNotAllowed {
        callee: String,
        enclosing_fn: String,
        prefixes: Vec<String>,
    },
    /// Stone 237.2 — no clause of a defclause matches at call site.
    ///
    /// `attempted_clauses: Vec<(usize, Vec<String>)>` has no `ToEdn` impl;
    /// `#[to_edn(via = clause_attempts_to_edn)]` routes it through a named helper
    /// that produces `[{:arity N :param-types [...]} ...]`.
    NoMatchingClauseAtCallSite {
        name: String,
        called_arity: usize,
        called_arg_types: Vec<String>,
        #[to_edn(via = crate::check::clause_attempts_to_edn)]
        attempted_clauses: Vec<(usize, Vec<String>)>,
    },
    /// Arc <post-278> — open-surface `defclause` dispatch matched MULTIPLE
    /// concrete-satisfier ("narrowing") clauses whose declared return types do
    /// NOT unify. First-match-wins would statically commit to one clause's
    /// return type while the runtime dispatches on the value's real class to a
    /// possibly different clause — unsound. `candidate_returns` carries the
    /// distinct formatted return types found among the matching clauses, in
    /// first-seen order.
    AmbiguousClauseReturnAtCallSite {
        name: String,
        called_arg_types: Vec<String>,
        candidate_returns: Vec<String>,
    },
    /// Stone 237.3 — `:guard` expression not boolean in defclause.
    GuardExprNotBoolean {
        defclause_name: String,
        clause_index: usize,
        got_type: String,
    },
    /// Stone 237.3 — `:ensure :fn` invalid in defclause.
    EnsureFnInvalid {
        defclause_name: String,
        clause_index: usize,
        reason: EnsureFnInvalidReason,
    },
    /// Arc 291 — hygiene-scope divergence: a reference is unbound, but a
    /// binder of the SAME NAME exists under a DIFFERENT hygiene scope.
    /// This is always a faulty macro that rebuilt a binder from its name
    /// (stripping/changing its ScopeId) instead of reusing the original node.
    HygieneScopeDivergence {
        name: String,
        ref_key: String,
        binder_key: String,
    },
    /// Arc 278 BRIEF-arming-is-internal-only — a `:wat::service::Alarm`'s `op` field is
    /// constructed from a literal `<service>::Op` variant ctor whose name does NOT begin
    /// with `-` (i.e. a PUBLIC, client-facing op). An alarm fires with no client in the
    /// `idx` slot — only an op *declared* to have no client (internal, dash-prefixed,
    /// `wat/service.wat:876-892`) may be armed. Constructing this with a public variant
    /// compiles clean today and silently discards the reply when the timer fires
    /// (proven by run: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-call-context.md`
    /// § "RUN 2026-08-09").
    ///
    /// `variant` is the offending ctor's full keyword (e.g. `:probe::tick2::Op::Bump`);
    /// `op_type` is the service's `Op` enum path it belongs to (e.g. `:probe::tick2::Op`).
    ///
    /// NAME PLACEHOLDER — an intueri cast on this error kind's name is OWED, not ratified
    /// (per the brief; do not treat this name as settled).
    PublicOpInAlarm {
        variant: String,
        op_type: String,
    },
    /// Excursus 002 stone 1 — a `Peer` escaped a scope that CREATED its service's
    /// `Handle` (a call that returns a Handle and does not take one). Outer span
    /// is the escape site (the let value, or the function body). `created_at` is
    /// the creating call. No remedy string: the fix is context-dependent (thread
    /// the Handle, or return it too).
    HandleCreationEscape {
        function: String,
        service: String,
        /// Source location of the Handle-creating call (`<svc>/start` or equivalent).
        #[to_edn(key = "created-at")]
        created_at: Span,
    },
    /// Excursus 002 stone 2 — a `let` that CREATES a Handle is itself in tail
    /// position, and its tail expression is a user-function call taking a `Peer`
    /// of that service. The scope (and Handle) die before the call runs.
    HandleTailEscape {
        function: String,
        service: String,
        #[to_edn(key = "created-at")]
        created_at: Span,
        #[to_edn(key = "tail-call")]
        tail_call: Span,
    },
}

/// Arc 296 S7 — the 5 structural failure modes for `:ensure :fn` validation.
///
/// Replaces the discriminant-as-prose `reason: String` in
/// `CheckErrorKind::EnsureFnInvalid`. The derive emits each variant as a
/// `#wat.kernel/<Tag>` tagged value with its fields as a map — structure
/// preserved by construction. `Display` reproduces the original prose
/// byte-for-byte so the human-visible sentence is unchanged.
#[derive(Debug, Clone, wat_edn::ToEdn)]
pub enum EnsureFnInvalidReason {
    /// The `:ensure` form is not a `:wat::core::fn` list.
    NotFnForm,
    /// The `:ensure :fn` has the wrong number of parameters (must be 1).
    ArityNotOne { got: usize },
    /// The `:ensure :fn` arg type does not match the clause return type.
    ArgTypeMismatch {
        arg_type: String,
        clause_return_type: String,
    },
    /// The `:ensure :fn` return type is not `:bool`.
    ReturnTypeNotBool { got: String },
    /// The `:ensure :fn` signature is structurally malformed.
    MalformedSignature,
}

impl fmt::Display for EnsureFnInvalidReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnsureFnInvalidReason::NotFnForm => {
                write!(f, "must be :wat::core::fn form")
            }
            EnsureFnInvalidReason::ArityNotOne { got } => {
                write!(f, "arity must be 1 (one parameter for the result); got {got}")
            }
            EnsureFnInvalidReason::ArgTypeMismatch { arg_type, clause_return_type } => {
                write!(
                    f,
                    "arg type must match clause return type: :ensure :fn takes `{arg_type}` but clause returns `{clause_return_type}`"
                )
            }
            EnsureFnInvalidReason::ReturnTypeNotBool { got } => {
                write!(f, "return type must be :bool; got `{got}`")
            }
            EnsureFnInvalidReason::MalformedSignature => {
                write!(
                    f,
                    "malformed :fn signature \u{2014} expected (:wat::core::fn [param <- :T] -> :bool body)"
                )
            }
        }
    }
}

impl CheckErrorKind {
    /// Render this error kind's human-facing message with the span woven in
    /// where it currently appears.
    ///
    /// `span` is `Some(&outer_span)` when rendering a full [`CheckError`]
    /// (span-bearing form) and `None` when rendering the kind alone (span-free
    /// form).  The rendered text is byte-identical to what the two former
    /// `Display` impls produced: message text lives here exactly once.
    fn fmt_with_span(
        &self,
        span: Option<&Span>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        // Arc 298.2: every span is real; always emit. `prefix` is `"file:line:col: "`
        // when span is Some, empty when None (the outer struct may omit the span
        // in kind-only Display). `shown` = span (always real when present).
        let prefix = span.map(span_prefix).unwrap_or_default();
        let shown: Option<&Span> = span;
        match self {
            CheckErrorKind::ArityMismatch { callee, expected, got } => {
                write!(f, "{}{}: expected {} argument(s); got {}", prefix, callee, expected, got)
            }
            CheckErrorKind::TypeMismatch { callee, param, expected, got } => {
                write!(
                    f,
                    "{}{}: parameter {} expects {}; got {}",
                    prefix, callee, param, expected, got
                )?;
                // Arc 296 remediation collapse: render structured remedies instead of prose hint.
                let section = crate::remedy::render_remedies(
                    &super::type_error_remedies(callee),
                );
                if !section.is_empty() {
                    write!(f, "\n{}", section)?;
                }
                Ok(())
            }
            CheckErrorKind::ReturnTypeMismatch { function, expected, got, remedies } => {
                write!(
                    f,
                    "{}{}: body produces {}; signature declares {}",
                    prefix, function, got, expected
                )?;
                // Arc 296 remediation collapse: merge stored remedies with computed type_error_remedies,
                // dedup by form, render once — no prose hint section.
                let mut merged: Vec<crate::remedy::Remedy> = remedies.clone();
                merged.extend(super::type_error_remedies(function));
                let mut seen = std::collections::HashSet::new();
                merged.retain(|r| seen.insert(r.form.clone()));
                let section = crate::remedy::render_remedies(&merged);
                if !section.is_empty() {
                    write!(f, "\n{}", section)?;
                }
                Ok(())
            }
            CheckErrorKind::UnknownCallee { callee } => {
                write!(f, "{}unknown callee: {}", prefix, callee)
            }
            CheckErrorKind::MalformedForm { head, reason, remedies } => {
                write!(f, "{}malformed {} form: {}", prefix, head, reason)?;
                let section = crate::remedy::render_remedies(remedies);
                if !section.is_empty() {
                    write!(f, "\n{}", section)?;
                }
                Ok(())
            }
            CheckErrorKind::CommCallOutOfPosition { callee } => write!(
                f,
                "{}{} may appear only as the scrutinee of `:wat::core::match`, the value-position of `:wat::core::Result/expect`, or the value-position of `:wat::core::Option/expect`; silent disconnect must be handled at every comm call",
                prefix, callee
            ),
            CheckErrorKind::ProcessJoinBeforeOutputDrain {
                process_identifier,
                output_accessor,
                output_accessor_span,
            } => {
                if let Some(s) = shown {
                    write!(f, "process-join-before-output-drain at {s}: ")?;
                }
                // Arc 298.2: span is always real; always emit the location.
                let out_loc_clause = format!("the output accessor call is at {}", output_accessor_span);
                write!(
                    f,
                    "`:wat::kernel::Process/join-result {p}` and `{acc} {p}` appear in the same `let` form (sibling bindings or body). `Process/join-result` BLOCKS until the forked child exits. The substrate's internal drain threads consume the child's OS stdout/stderr pipes and push lines into the wat-level Receivers obtained via `{acc}`. If those Receivers are bounded and the parent has not yet drained them, the substrate's drain threads block on send when full; the child's stdout writes fill the OS pipe and block; the child CANNOT EXIT; `Process/join-result` BLOCKS FOREVER. ILLEGAL STATEMENT ORIENTATION: {out_loc_clause}. Fix per SERVICE-PROGRAMS.md § \"The lockstep\" applied at the Process boundary: outer scope holds the Process; INNER scope owns every output-channel Receiver derived from it and drains them; outer scope's `Process/join-result` runs only AFTER the inner has consumed-and-disconnected (Receivers dropped at inner-scope exit). DO NOT add a wall-clock timeout to mask this — restructure the let.",
                    p = process_identifier,
                    acc = output_accessor,
                )
            }
            CheckErrorKind::ProcessJoinHoldsStdinSender {
                process_identifier,
                stdin_sender_span,
            } => {
                if let Some(s) = shown {
                    write!(f, "process-join-holds-stdin-sender at {s}: ")?;
                }
                // Arc 298.2: span is always real; always emit the location.
                let bind_clause = format!("the Process handle bound at {}", stdin_sender_span);
                write!(
                    f,
                    "`:wat::kernel::Process/join-result {p}` blocks until the forked child exits, but `:wat::kernel::Process/stdin {p}` was never extracted from the Process handle anywhere in this `let` scope. The substrate's child has a structural StdInService (arc 170 slice 1f) blocked on `read(fd 0)` waiting for EOF. The parent holds the write-end of the child's stdin pipe via {bind_clause}. Without EOF on that pipe, the child cannot exit; parent's join blocks forever — a true deadlock. ILLEGAL STATEMENT ORIENTATION. Fix per SERVICE-PROGRAMS.md § \"The lockstep\" applied at the Process boundary: extract `:wat::kernel::Process/stdin {p}` in an INNER `let` (nested inside an outer binding before the join binding) so the Sender drops at inner-let exit before the outer join runs. The inner let should also contain the output Receivers and drain them before returning. DO NOT add a wall-clock timeout to mask this — restructure the let.",
                    p = process_identifier,
                )
            }
            CheckErrorKind::BareLegacyPrimitive { primitive, fqdn } => {
                write!(f, "bare primitive type '{}'", primitive)?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 109 slice 1c); canonical FQDN form is '{}'. Substrate-provided primitives live under :wat::core::* (see arc 109 § A). Rename '{}' → '{}' at the offending site.",
                    fqdn, primitive, fqdn
                )
            }
            CheckErrorKind::BareLegacyUnitType => {
                write!(f, "bare unit type '()'")?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 109 slice 1d); canonical FQDN form is ':wat::core::nil' (arc 153 renamed unit -> nil). Substrate-provided primitives live under :wat::core::* (see arc 109 § A). The empty-tuple LITERAL VALUE `()` is unaffected; only the type-position spelling renames. Rename ':()' -> ':wat::core::nil' (or '()' -> 'wat::core::nil' inside parametrics) at the offending site."
                )
            }
            CheckErrorKind::BareLegacyUnitValue => {
                write!(f, "bare unit value '()'")?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 179); `nil` is the sole unit value. An empty list literal `()` in expression position is no longer a second spelling of unit. `()` survives ONLY as empty-parameter-list syntax (e.g. ':wat::core::Fn()->T', '(() -> T)'), which is a distinct grammatical position from a value expression and is unaffected by this error. Rename '()' -> 'nil' at the offending site."
                )
            }
            CheckErrorKind::BareLegacyUnitName => {
                write!(f, "':wat::core::unit'")?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 153); canonical FQDN is ':wat::core::nil'. Same role (singleton type, 'no meaningful return value'); rename ships the marker effect of a Lisp's nil while preserving wat's existing (Option :- [T])::None / Some(t) discipline. Rename ':wat::core::unit' -> ':wat::core::nil' at the offending site."
                )
            }
            CheckErrorKind::BareLegacyLetStar => {
                write!(f, "':wat::core::let*'")?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 154); canonical FQDN is ':wat::core::let'. Same sequential semantics, single name (Clojure-faithful: Clojure's user-facing `let` IS the sequential primitive; `let*` is a substrate-internal form not part of normal user code). Rename ':wat::core::let*' -> ':wat::core::let' at the offending site."
                )
            }
            CheckErrorKind::BareLegacyLambda => {
                write!(f, "':wat::core::lambda'")?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 155); canonical FQDN is ':wat::core::fn'. Clojure-faithful single-letform vocabulary: lowercase 'fn' for function values (matches Clojure's user-facing `fn`). Rename ':wat::core::lambda' -> ':wat::core::fn' at the offending site."
                )
            }
            CheckErrorKind::BareLegacyLowercaseFn => {
                write!(f, "bare ':fn(...)' type")?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 155); canonical FQDN is ':wat::core::Fn(...)'. Cap'd type head per Clojure-faithful capitalization convention: 'Fn' = function type, 'fn' = function value (closes arc 109 slice 1e's last ungrabbed parametric type head). Rename ':fn(args)->ret' -> ':wat::core::Fn(args)->ret' at the offending site."
                )
            }
            CheckErrorKind::BareLegacyContainerHead { head, fqdn } => {
                write!(f, "bare container type '{}'", head)?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 109 slice 1e); canonical FQDN form is '{}'. Substrate-provided container types live under :wat::core::* (see arc 109 § B). Rename '{}' → '{}' at the offending site (works in both keyword position like ':{}' → ':{}' and a type reference like '({} :- [args])' → '({} :- [args])').",
                    fqdn, head, fqdn, head, fqdn, head, fqdn
                )
            }
            CheckErrorKind::BareLegacyStreamPath { old, new } => {
                write!(f, "legacy stream path '{}'", old)?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 109 slice 9d); canonical form is '{}'. The stream stdlib graduated to :wat::stream::* per § G's three-tier substrate organization (every substrate concern earns its own top-level tier; :wat::std::* empties out). File path mirrors: wat/std/stream.wat → wat/stream.wat. Rename '{}' → '{}' at the offending site.",
                    new, old, new
                )
            }
            CheckErrorKind::BareLegacyLruCacheServicePath { old, new } => {
                write!(f, "legacy lru-cache-service path '{}'", old)?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 109 slice K.lru); canonical form is '{}'. The :wat::lru::CacheService grouping noun retired per § K's '/ requires a real Type' doctrine. Real types Stats / MetricsCadence / State / Report keep PascalCase + /methods (just one less namespace segment). Plus Pattern B canonicalization: ReqPair renamed to ReqChannel (in-crate ReqPair/ReplyChannel mumble); ReplyRx<V> + ReplyChannel<V> typealiases minted to complete the Pattern B reference. Rename '{}' → '{}' at the offending site.",
                    new, old, new
                )
            }
            CheckErrorKind::BareLegacyKernelQueuePath { old, new } => {
                write!(f, "legacy kernel queue path '{}'", old)?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 109 slice K.kernel-channel); canonical form is '{}'. The :wat::kernel::Queue* family renamed to Channel / Sender / Receiver (Queue leaked crossbeam's data-structure name; the canonical vocabulary is the substrate's honest naming). File moved: wat/kernel/queue.wat → wat/kernel/channel.wat. Rename '{}' → '{}' at the offending site.",
                    new, old, new
                )
            }
            CheckErrorKind::DefRedefForbidden { name, original_def_span } => {
                // Arc 298.2: span is always real; always emit the location.
                let prior_loc = format!("{}", original_def_span);
                write!(
                    f,
                    "{}`:wat::core::def` redef of `{}`: name already bound at {}. Redef is forbidden by default; opt in via `(:wat::config::set-redef! true)` before this form. Use a different name, or enable redef explicitly.",
                    prefix, name, prior_loc
                )
            }
            CheckErrorKind::DefRedefTypeChange { name, prior_type, new_type, original_def_span } => {
                // Arc 298.2: span is always real; always emit the location.
                let prior_loc = format!("{}", original_def_span);
                write!(
                    f,
                    "{}`:wat::core::def` redef of `{}` changes type from `{}` to `{}` (prior binding at {}). Type-stability is mandatory on redef — the signature downstream callers depend on must stay intact. Only the expression's value may change; the type must not.",
                    prefix, name, prior_type, new_type, prior_loc
                )
            }
            CheckErrorKind::UnnamespacedName { name } => write!(
                f,
                "{}top-level name '{}' is not namespaced — only fn arguments and let-bindings \
                 may be bare; give it a namespace, e.g. ':my::{}'",
                prefix,
                name,
                name.trim_start_matches(':')
            ),
            CheckErrorKind::ReservedPrefix { name } => write!(
                f,
                "{}cannot define {} — reserved prefix ({}); user defines must use their own prefix",
                prefix,
                name,
                crate::resolve::reserved_prefix_list()
            ),
            CheckErrorKind::DottedName { name } => write!(
                f,
                "{}name '{}' contains a '.' in its name segment — reserved: a dot in a tag's \
                 NAME half means \"this is an enum variant\" (`#ns/Enum.Variant`), so a \
                 registered name may not contain one, or it could forge that tag; rename \
                 without the dot",
                prefix, name
            ),
            CheckErrorKind::DuplicateScheme { name } => write!(
                f,
                "{}'{}' is already registered with a different signature — the overlay \
                 scheme table does not allow a name to mean two different things",
                prefix, name
            ),
            CheckErrorKind::BareLegacyMainSignature => write!(
                f,
                "{}`:user::main` declared with a non-canonical signature is retired (arc 170 slice 1e — REALIZATIONS pass 7 + pass 10); canonical shape is `[] -> :wat::core::nil`. The four-arg shape (stdin/stdout/stderr/argv) retired: argv moves to the ambient `(:wat::runtime::argv)`; stdio access moves to the three substrate services (`:wat::kernel::StdInService` / `StdOutService` / `StdErrService` per slice 1f). `nil` IS the success exit code — clean nil-return maps to libc::exit(0); panic-cascade maps to libc::exit(N) via the StdErrService epilogue. User code never participates in exit-code arithmetic. Migrate the define to:\n  (:wat::core::defn :user::main [] -> :wat::core::nil \n    <body that does work and returns :wat::core::nil>)\nor with `defn`:\n  (:wat::core::defn :user::main [] -> :wat::core::nil\n    <body>)",
                prefix
            ),
            CheckErrorKind::BareLegacyConsolePath { path } => {
                write!(f, "{}`:wat::console::*`", prefix)?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " is retired (arc 109 § kill-std / arc 170 slice 1f-η). The :wat::console::* namespace (Console driver, spawn factory, handle plumbing, ConsoleLogger) has been fully annihilated. User code uses the ambient kernel-level stdio ops directly:\n  - For output:  (:wat::kernel::println v)         — EDN-encodes v, emits to stdout\n  - For error:   (:wat::kernel::eprintln v)        — EDN-encodes v, emits to stderr\n  - For input:   (:wat::kernel::readln)             — reads one EDN-decoded value (the self-describing wire types it)\nThese are EDN-only — any value EDN-encodes; no manual string formatting. See examples/console-demo/wat/main.wat for the canonical ambient-stdio shape. Offending token: '{}'.",
                    path
                )
            }
            CheckErrorKind::DefRestrictedCallerNotAllowed {
                callee,
                enclosing_fn,
                prefixes: allowed_prefixes,
            } => {
                write!(f, "{}`{}`", prefix, callee)?;
                if let Some(s) = shown {
                    write!(f, " at {}", s)?;
                }
                write!(
                    f,
                    " has a restricted caller whitelist [{}]; the enclosing fn `{}` does not match any entry (declared via `{{:restricted-to [...]}}` metadata-map). An entry ending in `::` is a namespace prefix (caller FQDN must start with it); an entry without trailing `::` is an exact-FQDN match. Either move the caller into one of the allowed namespaces, or add `{}` to the `:restricted-to` list at the binding site.",
                    allowed_prefixes.join(" "),
                    enclosing_fn,
                    enclosing_fn,
                )
            }
            CheckErrorKind::NoMatchingClauseAtCallSite {
                name,
                called_arity,
                called_arg_types,
                attempted_clauses,
            } => {
                let clause_summary: Vec<String> = attempted_clauses
                    .iter()
                    .map(|(arity, types)| format!("({}: [{}])", arity, types.join(", ")))
                    .collect();
                write!(
                    f,
                    "{}no clause of `{}` matches arity {} with types [{}]; \
                     clauses attempted: {}",
                    prefix,
                    name,
                    called_arity,
                    called_arg_types.join(", "),
                    if clause_summary.is_empty() { "(none)".into() } else { clause_summary.join("; ") },
                )
            }
            CheckErrorKind::AmbiguousClauseReturnAtCallSite { name, called_arg_types, candidate_returns } => {
                write!(
                    f,
                    "{}open-surface dispatch of `{}` on [{}] matches clauses with incompatible \
                     return types ({}); a discriminating defclause's matching clauses must share \
                     a return type",
                    prefix,
                    name,
                    called_arg_types.join(", "),
                    candidate_returns
                        .iter()
                        .map(|t| format!("`{}`", t))
                        .collect::<Vec<_>>()
                        .join(" vs "),
                )
            }
            CheckErrorKind::GuardExprNotBoolean { defclause_name, clause_index, got_type } => {
                write!(
                    f,
                    "{}defclause `{}` clause {}: `:guard` expression must return `:wat::core::bool`; got `{}`",
                    prefix,
                    defclause_name,
                    clause_index,
                    got_type,
                )
            }
            CheckErrorKind::EnsureFnInvalid { defclause_name, clause_index, reason } => {
                write!(
                    f,
                    "{}defclause `{}` clause {}: `:ensure` :fn is invalid — {}",
                    prefix,
                    defclause_name,
                    clause_index,
                    reason,
                )
            }
            CheckErrorKind::HygieneScopeDivergence { name, ref_key, binder_key } => {
                write!(
                    f,
                    "{}hygiene-scope divergence: reference `{}` (scope {{{}}}) is unbound, \
                     but a binder `{}` exists under a different scope {{{}}} — \
                     a macro rebuilt this binder from its name instead of reusing the node; \
                     reuse the original AST node.",
                    prefix,
                    name,
                    // ref_key is "name" for bare or "name\u{1}scopes" for scoped; extract scope part
                    ref_key.split_once('\u{1}').map(|x| x.1).unwrap_or(""),
                    name,
                    binder_key.split_once('\u{1}').map(|x| x.1).unwrap_or(""),
                )
            }
            CheckErrorKind::PublicOpInAlarm { variant, op_type } => {
                write!(
                    f,
                    "{}`:wat::service::Alarm`'s `op` is `{}`, a PUBLIC (client-facing) variant of \
                     `{}` — an alarm fires with no client, so only an INTERNAL op (its variant name \
                     begins with `-`) may be armed; a public op's reply would go nowhere with \
                     nothing reported. Declare an internal `-op` and arm that instead (a public op \
                     and a `-tick` may share one helper fn).",
                    prefix, variant, op_type,
                )
            }
            CheckErrorKind::HandleCreationEscape {
                function,
                service,
                created_at,
            } => write!(
                f,
                "{}function {}: a peer of {} escapes the scope that created its Handle at {} — \
                 the Handle dies when this scope ends, leaving a live channel to nothing",
                prefix,
                function,
                service,
                created_at,
            ),
            CheckErrorKind::HandleTailEscape {
                function,
                service,
                created_at,
                tail_call,
            } => write!(
                f,
                "{}function {}: a tail call at {} carries a peer of {} out of the scope that \
                 created its Handle at {} — the Handle dies before the call runs, leaving a live \
                 channel to nothing",
                prefix,
                function,
                tail_call,
                service,
                created_at,
            ),
        }
    }
}

impl fmt::Display for CheckErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Message text lives once in `fmt_with_span`; the span-free form
        // passes None so no prefix or mid-prose span is inserted.
        self.fmt_with_span(None, f)
    }
}

impl fmt::Debug for CheckError {
    // Stone B: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl std::error::Error for CheckError {}


/// Aggregated errors — `check_program` returns all findings together.
pub struct CheckErrors(pub Vec<CheckError>);

impl fmt::Debug for CheckErrors {
    // Stone B: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl fmt::Display for CheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl std::error::Error for CheckErrors {}
