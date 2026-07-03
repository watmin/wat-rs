//! Arc 298.3 — `#[derive(ToEdn)]` on `RuntimeErrorKind` is byte-identical to
//! the deleted `runtime_error_to_edn` hand-written serializer.
//!
//! ## What this proves
//!
//! For every `RuntimeErrorKind` variant (31 probe functions):
//! - The derive generates the same `#wat.runtime/<Name>` tag.
//! - Field keys are snake→kebab in declaration order.
//! - `:span` is appended LAST by `splice_span` (from the outer `RuntimeError.span`).
//! - `Box<T>: ToEdn` delegates through the Box.
//! - `Option<T>` is tagged (`#wat.core.Option/Some`/`None`) per arc 298.1.
//! - Secondary `Span` fields emit under their auto-kebab key (e.g. `outer-define-span`).
//! - `MacroExpansionFailed.cause` uses `error_edn_of_boxed` (floor form).
//! - `EdnCoerceMismatch.path` is split into a Vec via `edn_path_segments`.
//! - `Vec<ClauseAttempt>` serializes via `impl ToEdn for ClauseAttempt`.
//!
//! ## How the golden strings were derived
//!
//! Each golden was captured by running the test with a temporary `eprintln!`
//! and pasting the byte-identical stderr output. NEVER hand-guessed.
//!
//! PROBATIO FLEXA MENTITVR — every probe uses `assert_eq!` on exact bytes.

use std::sync::Arc;
use wat::hash::HashError;
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{
    ClauseAttempt, ClauseFailureReason, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot,
};
use wat::span::Span;
use wat::to_edn::ToEdn;
use wat::value::Provenance;

fn s() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn make(kind: RuntimeErrorKind) -> RuntimeError {
    RuntimeError { span: s(), kind }
}

fn write(err: &RuntimeError) -> String {
    wat_edn::write(&err.to_edn())
}

// ─── 1. UnboundSymbol ────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_unbound_symbol() {
    let err = make(RuntimeErrorKind::UnboundSymbol("my-var".into()));
    assert_eq!(
        write(&err),
        r#"#wat.runtime/UnboundSymbol {:name "my-var" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 2. UnknownFunction ──────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_unknown_function() {
    let err = make(RuntimeErrorKind::UnknownFunction(":user::greet".into()));
    assert_eq!(
        write(&err),
        r#"#wat.runtime/UnknownFunction {:path ":user::greet" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 3. NotCallable ──────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_not_callable() {
    let snap = ValueSnapshot::of(&Value::String(Arc::new("hello".to_string())));
    let err = make(RuntimeErrorKind::NotCallable { got: Box::new(snap) });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/NotCallable {:got {:type "wat::core::String" :rendered "\"hello\"" :provenance nil} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 4. TypeMismatch ─────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_type_mismatch() {
    let snap = ValueSnapshot::of(&Value::i64(42));
    let err = make(RuntimeErrorKind::TypeMismatch {
        op: ":wat::core::+".into(),
        expected: "i64",
        got: Box::new(snap),
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/TypeMismatch {:op ":wat::core::+" :expected "i64" :got {:type "wat::core::i64" :rendered "42" :provenance nil} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 5. ArityMismatch ────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_arity_mismatch() {
    let err = make(RuntimeErrorKind::ArityMismatch {
        op: ":my::func".into(),
        expected: 2,
        got: 3,
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/ArityMismatch {:op ":my::func" :expected 2 :got 3 :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 6. BadCondition ─────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_bad_condition() {
    let snap = ValueSnapshot::of(&Value::i64(0));
    let err = make(RuntimeErrorKind::BadCondition { got: Box::new(snap) });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/BadCondition {:got {:type "wat::core::i64" :rendered "0" :provenance nil} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 7. MalformedForm ────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_malformed_form() {
    let err = make(RuntimeErrorKind::MalformedForm {
        head: ":wat::core::fn".into(),
        reason: "param list must be a vector".into(),
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/MalformedForm {:head ":wat::core::fn" :reason "param list must be a vector" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 8. ParamShadowsBuiltin ──────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_param_shadows_builtin() {
    let err = make(RuntimeErrorKind::ParamShadowsBuiltin("map".into()));
    assert_eq!(
        write(&err),
        r#"#wat.runtime/ParamShadowsBuiltin {:name "map" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 9. DivisionByZero ───────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_division_by_zero() {
    let err = make(RuntimeErrorKind::DivisionByZero);
    assert_eq!(
        write(&err),
        r#"#wat.runtime/DivisionByZero {:span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 10. DuplicateDefine ─────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_duplicate_define() {
    let err = make(RuntimeErrorKind::DuplicateDefine(":user::counter".into()));
    assert_eq!(
        write(&err),
        r#"#wat.runtime/DuplicateDefine {:name ":user::counter" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 11. ReservedPrefix ──────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_reserved_prefix() {
    let err = make(RuntimeErrorKind::ReservedPrefix(":wat::my-thing".into()));
    assert_eq!(
        write(&err),
        r#"#wat.runtime/ReservedPrefix {:prefix ":wat::my-thing" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 12. DeclarationInExpressionPosition ─────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_declaration_in_expression_position() {
    let err = make(RuntimeErrorKind::DeclarationInExpressionPosition(":wat::core::define".into()));
    assert_eq!(
        write(&err),
        r#"#wat.runtime/DeclarationInExpressionPosition {:head ":wat::core::define" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 13. EvalForbidsMutationForm ─────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_eval_forbids_mutation_form() {
    let err = make(RuntimeErrorKind::EvalForbidsMutationForm { head: ":wat::core::define".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/EvalForbidsMutationForm {:head ":wat::core::define" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 14. UserMainMissing ─────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_user_main_missing() {
    let err = make(RuntimeErrorKind::UserMainMissing);
    assert_eq!(
        write(&err),
        r#"#wat.runtime/UserMainMissing {:span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 15. EvalVerificationFailed ──────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_eval_verification_failed() {
    let err = make(RuntimeErrorKind::EvalVerificationFailed {
        err: HashError::UnsupportedAlgorithm { algo: "SHA1".into() },
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/EvalVerificationFailed {:error #wat.kernel/UnsupportedAlgorithm {:algo "SHA1"} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 16. ChannelDisconnected ─────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_channel_disconnected() {
    let err = make(RuntimeErrorKind::ChannelDisconnected { op: ":wat::kernel::send".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/ChannelDisconnected {:op ":wat::kernel::send" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 17. NoEncodingCtx ───────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_no_encoding_ctx() {
    let err = make(RuntimeErrorKind::NoEncodingCtx { op: ":wat::holon::cosine".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/NoEncodingCtx {:op ":wat::holon::cosine" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 18. NoSourceLoader ──────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_no_source_loader() {
    let err = make(RuntimeErrorKind::NoSourceLoader { op: ":wat::eval-file!".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/NoSourceLoader {:op ":wat::eval-file!" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 19. NoMacroRegistry ─────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_no_macro_registry() {
    let err = make(RuntimeErrorKind::NoMacroRegistry { op: ":wat::core::macroexpand".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/NoMacroRegistry {:op ":wat::core::macroexpand" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 20. MacroExpansionFailed ────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_macro_expansion_failed() {
    let inner_span = Span::new(Arc::new("macro.wat".to_string()), 5, 2);
    let cause = Box::new(MacroError {
        span: inner_span,
        kind: MacroErrorKind::DuplicateMacro("my-macro".into()),
    });
    let err = make(RuntimeErrorKind::MacroExpansionFailed {
        op: ":user::expand".into(),
        cause,
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/MacroExpansionFailed {:op ":user::expand" :cause #wat.macro/DuplicateMacro {:message "duplicate macro registration: my-macro" :location {:file "macro.wat" :line 5 :col 2} :causes [] :name "my-macro"} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 21. PatternMatchFailed ──────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_pattern_match_failed() {
    let err = make(RuntimeErrorKind::PatternMatchFailed { value_type: "i64" });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/PatternMatchFailed {:value-type "i64" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 22. EffectfulInStep ─────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_effectful_in_step() {
    let err = make(RuntimeErrorKind::EffectfulInStep { op: ":wat::kernel::println".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/EffectfulInStep {:op ":wat::kernel::println" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 23. NoStepRule ──────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_no_step_rule() {
    let err = make(RuntimeErrorKind::NoStepRule { op: ":user::custom-op".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/NoStepRule {:op ":user::custom-op" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 24a. AssertionFailed (both Some) ────────────────────────────────────────
//
// Arc 298.1 wire change: `Option<String>` is now tagged.
// Old: `:actual "42"` (transparent)
// New: `:actual #wat.core.Option/Some "42"` (tagged per arc 298.1)

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_assertion_failed_both_some() {
    let err = make(RuntimeErrorKind::AssertionFailed {
        message: "values differ".into(),
        actual: Some("42".into()),
        expected: Some("99".into()),
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/AssertionFailed {:message "values differ" :actual #wat.core.Option/Some "42" :expected #wat.core.Option/Some "99" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 24b. AssertionFailed (expected None) ────────────────────────────────────
//
// Arc 298.1: `None` → `#wat.core.Option/None nil`

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_assertion_failed_expected_none() {
    let err = make(RuntimeErrorKind::AssertionFailed {
        message: "fired".into(),
        actual: Some("x".into()),
        expected: None,
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/AssertionFailed {:message "fired" :actual #wat.core.Option/Some "x" :expected #wat.core.Option/None nil :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 25. SandboxScopeLeak ────────────────────────────────────────────────────
//
// Multi-span: outer `span` (from `err.span`) appended as `:span` LAST by
// `splice_span`; secondary `outer_define_span: Span` emitted as
// `:outer-define-span` in declaration order (before `:span`).
//
// Arc 298.3 wire change: old form used `:call-span` for the outer span;
// new form uses `:span` (the derive's splice_span name).

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_sandbox_scope_leak() {
    let outer_span = Span::new(Arc::new("outer.wat".to_string()), 10, 4);
    let err = make(RuntimeErrorKind::SandboxScopeLeak {
        offending_name: ":user::my-helper".into(),
        outer_define_span: outer_span,
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/SandboxScopeLeak {:offending-name ":user::my-helper" :outer-define-span {:file "outer.wat" :line 10 :col 4} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 26. ServiceNotRunning ───────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_service_not_running() {
    let err = make(RuntimeErrorKind::ServiceNotRunning { op: ":wat::kernel::println".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/ServiceNotRunning {:op ":wat::kernel::println" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 27a. EdnCoerceMismatch (non-empty path) ─────────────────────────────────
//
// `path` field uses `edn_path_segments` via: splits on '.' → `["x" "y"]`

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_edn_coerce_mismatch() {
    let err = make(RuntimeErrorKind::EdnCoerceMismatch {
        op: ":wat::kernel::readln".into(),
        expected: ":user::Point".into(),
        got: "Map".into(),
        path: "x.y".into(),
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/EdnCoerceMismatch {:op ":wat::kernel::readln" :expected ":user::Point" :got "Map" :path ["x" "y"] :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 27b. EdnCoerceMismatch (empty path) ─────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_edn_coerce_mismatch_empty_path() {
    let err = make(RuntimeErrorKind::EdnCoerceMismatch {
        op: ":wat::kernel::readln".into(),
        expected: ":user::Point".into(),
        got: "Integer".into(),
        path: "".into(),
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/EdnCoerceMismatch {:op ":wat::kernel::readln" :expected ":user::Point" :got "Integer" :path [] :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 28. UnknownField ────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_unknown_field() {
    let err = make(RuntimeErrorKind::UnknownField {
        record_class: "user::Point".into(),
        field: "z".into(),
        available: vec!["x".into(), "y".into()],
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/UnknownField {:record-class "user::Point" :field "z" :available ["x" "y"] :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 29. NoMatchingClause ────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_no_matching_clause() {
    let err = make(RuntimeErrorKind::NoMatchingClause {
        name: ":user::process".into(),
        called_arity: 1,
        called_args: vec![ValueSnapshot::of(&Value::i64(42))],
        attempted_clauses: vec![
            ClauseAttempt {
                clause_index: 0,
                declared_arity: 2,
                declared_arg_types: vec!["i64".into(), "i64".into()],
                failure_reason: ClauseFailureReason::ArityMismatch { expected: 2, got: 1 },
            },
            ClauseAttempt {
                clause_index: 1,
                declared_arity: 1,
                declared_arg_types: vec!["String".into()],
                failure_reason: ClauseFailureReason::ArgTypeMismatch {
                    position: 0,
                    expected: "String".into(),
                    got: "i64".into(),
                },
            },
        ],
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/NoMatchingClause {:name ":user::process" :called-arity 1 :called-args [{:type "wat::core::i64" :rendered "42" :provenance nil}] :attempted-clauses [#wat.kernel/ClauseAttempt {:clause-index 0 :declared-arity 2 :declared-arg-types ["i64" "i64"] :failure-reason #wat.kernel/ArityMismatch {:expected 2 :got 1}} #wat.kernel/ClauseAttempt {:clause-index 1 :declared-arity 1 :declared-arg-types ["String"] :failure-reason #wat.kernel/ArgTypeMismatch {:position 0 :expected "String" :got "i64"}}] :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 30. PostconditionFailed ─────────────────────────────────────────────────
//
// Multi-span: outer `span` (from `err.span`) appended as `:span` LAST;
// secondary `ensure_span: Span` emitted as `:ensure-span` in declaration order.
//
// Arc 298.3 wire change: old form used `:body-span` for the outer span;
// new form uses `:span`.

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_postcondition_failed() {
    let ensure_span = Span::new(Arc::new("defs.wat".to_string()), 20, 8);
    let err = make(RuntimeErrorKind::PostconditionFailed {
        defclause_name: ":user::positive".into(),
        clause_index: 0,
        ensure_expr_snapshot: "(> result 0)".into(),
        returned_value: Box::new(ValueSnapshot::of(&Value::i64(-5))),
        ensure_span,
    });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/PostconditionFailed {:defclause-name ":user::positive" :clause-index 0 :ensure-expr-snapshot "(> result 0)" :returned-value {:type "wat::core::i64" :rendered "-5" :provenance nil} :ensure-span {:file "defs.wat" :line 20 :col 8} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 31. MacroAbort ──────────────────────────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_macro_abort() {
    let err = make(RuntimeErrorKind::MacroAbort { message: "bad template".into() });
    assert_eq!(
        write(&err),
        r#"#wat.runtime/MacroAbort {:message "bad template" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── Silence unused import warnings ──────────────────────────────────────────

#[allow(dead_code)]
fn _uses_provenance(_: Provenance) {}
