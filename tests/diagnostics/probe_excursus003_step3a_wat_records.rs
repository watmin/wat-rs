//! Excursus 003 envelope step 3a — gates G1/G2/G3 for `RuntimeError::to_record`
//! (`src/value/runtime_records.rs`) against `wat/runtime-errors.wat` /
//! `wat/kernel/diagnostics.wat`'s new `ClauseAttempt`/`ClauseFailureReason`.
//!
//! - **G1** — the declared record set equals the produced record-class set.
//! - **G2** — `to_record`'s EDN agrees with `WatError::error_edn()`'s wire, with
//!   only the two ruled exception classes (`G2_EXCEPTIONS` below), enumerated
//!   by (variant, field), so an unlisted difference goes RED.
//! - **G3** — every record's EDN round-trips through the typed decoder
//!   (`edn_to_value`) to a `Value` equal to the one written.
//!
//! Each gate's mutation is recorded in the strike report (broken, confirmed
//! RED, restored) — not re-encoded here as a second, permanently-mutated copy.

// rune:lint(no-inlined-wat) — this file constructs `RuntimeError`/`ClauseAttempt` Rust structs
// DIRECTLY (`all_variants`, no startup/eval pipeline) and drives them through `to_record`,
// `value_to_edn_with`, and `edn_to_value`. `PostconditionFailed`'s `ensure_expr_snapshot` field
// carries the literal `"(> result 0)"` as opaque snapshot text — the same field, same literal,
// same reason as `probe_arc237_stone4_rich_errors.rs`: it happens to look like a wat form but
// is never handed to wat's reader or evaluator here — Rust-level data, not wat-under-test.

use std::sync::Arc;

use wat::runtime::{
    ClauseAttempt, ClauseFailureReason, ReteCeiling, RuntimeError, RuntimeErrorKind,
    Value, ValueSnapshot,
};
use wat::hash::{HashError, HashErrorKind};
use wat::macros::{MacroError, MacroErrorKind};
use wat::span::Span;
use wat::edn::contract::WatError;
use wat::edn::render::{edn_to_value, value_to_edn_with};
use wat::types::TypeEnv;
use wat_edn::OwnedValue;

fn s() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn snap(v: Value) -> ValueSnapshot {
    ValueSnapshot::of(&v)
}

/// One instance of every `RuntimeErrorKind` variant, paired with the
/// `:wat::runtime::<Kind>` record name `to_record` must produce for it.
/// 40 entries — the measured count (`src/value/signal.rs:376`), not the
/// brief's first-draft 31.
fn all_variants() -> Vec<(&'static str, RuntimeError)> {
    let mk = |k: RuntimeErrorKind| RuntimeError::new(s(), k);
    vec![
        ("UnboundSymbol", mk(RuntimeErrorKind::UnboundSymbol("x".into()))),
        ("UnknownFunction", mk(RuntimeErrorKind::UnknownFunction(":user::f".into()))),
        ("NotValueDispatchable", mk(RuntimeErrorKind::NotValueDispatchable { name: ":wat::core::if".into() })),
        ("NotCallable", mk(RuntimeErrorKind::NotCallable { got: Box::new(snap(Value::i64(1))) })),
        ("TypeMismatch", mk(RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::+".into(), expected: "i64", got: Box::new(snap(Value::bool(true))),
        })),
        ("ArityMismatch", mk(RuntimeErrorKind::ArityMismatch { op: ":user::f".into(), expected: 2, got: 1 })),
        ("BadCondition", mk(RuntimeErrorKind::BadCondition { got: Box::new(snap(Value::i64(0))) })),
        ("MalformedForm", mk(RuntimeErrorKind::MalformedForm { head: "if".into(), reason: "missing branch".into() })),
        ("ParamShadowsBuiltin", mk(RuntimeErrorKind::ParamShadowsBuiltin("if".into()))),
        ("DivisionByZero", mk(RuntimeErrorKind::DivisionByZero)),
        ("IntegerOverflow", mk(RuntimeErrorKind::IntegerOverflow { op: "+".into(), a: i64::MAX, b: 1 })),
        ("DuplicateDefine", mk(RuntimeErrorKind::DuplicateDefine(":user::f".into()))),
        ("ReservedPrefix", mk(RuntimeErrorKind::ReservedPrefix(":wat::x".into()))),
        ("UnreachableClause", mk(RuntimeErrorKind::UnreachableClause {
            name: ":user::f".into(), clause_index: 1, subsumed_by: 0, declared_arg_types: vec!["i64".into()],
        })),
        ("UnnamespacedName", mk(RuntimeErrorKind::UnnamespacedName("bare".into()))),
        ("DottedName", mk(RuntimeErrorKind::DottedName(":user::a.b".into()))),
        ("DeclarationInExpressionPosition", mk(RuntimeErrorKind::DeclarationInExpressionPosition("define".into()))),
        ("EvalForbidsMutationForm", mk(RuntimeErrorKind::EvalForbidsMutationForm { head: "define".into() })),
        ("UserMainMissing", mk(RuntimeErrorKind::UserMainMissing)),
        ("EvalVerificationFailed", mk(RuntimeErrorKind::EvalVerificationFailed {
            err: HashError::new(s(), HashErrorKind::Mismatch { algo: "sha256".into(), expected: "aaa".into(), actual: "bbb".into() }),
        })),
        ("ChannelDisconnected", mk(RuntimeErrorKind::ChannelDisconnected { op: ":wat::kernel::join".into() })),
        ("ReteCeiling", mk(RuntimeErrorKind::ReteCeiling(ReteCeiling::FixpointRoundCapExceeded { cap: 50, still_deriving: 12 }))),
        ("NoEncodingCtx", mk(RuntimeErrorKind::NoEncodingCtx { op: ":wat::holon::cosine".into() })),
        ("NoSourceLoader", mk(RuntimeErrorKind::NoSourceLoader { op: ":wat::eval-file!".into() })),
        ("NoMacroRegistry", mk(RuntimeErrorKind::NoMacroRegistry { op: ":wat::core::macroexpand".into() })),
        ("MacroExpansionFailed", mk(RuntimeErrorKind::MacroExpansionFailed {
            op: ":wat::core::macroexpand".into(),
            cause: Box::new(MacroError { span: s(), kind: MacroErrorKind::DuplicateMacro(":user::m".into()) }),
        })),
        ("PatternMatchFailed", mk(RuntimeErrorKind::PatternMatchFailed { value_type: "i64" })),
        ("EffectfulInStep", mk(RuntimeErrorKind::EffectfulInStep { op: ":wat::kernel::println".into() })),
        ("NoStepRule", mk(RuntimeErrorKind::NoStepRule { op: ":wat::future::thing".into() })),
        ("AssertionFailed", mk(RuntimeErrorKind::AssertionFailed {
            message: "values differ".into(), actual: Some("42".into()), expected: Some("99".into()),
        })),
        ("SandboxScopeLeak", mk(RuntimeErrorKind::SandboxScopeLeak { offending_name: ":user::helper".into(), outer_define_span: s() })),
        ("ServiceNotRunning", mk(RuntimeErrorKind::ServiceNotRunning { op: ":wat::kernel::println".into() })),
        ("EdnCoerceMismatch", mk(RuntimeErrorKind::EdnCoerceMismatch {
            op: ":wat::kernel::readln".into(), expected: Box::new("i64".into()), got: Box::new("string".into()), path: "a.b".into(),
        })),
        ("UnknownField", mk(RuntimeErrorKind::UnknownField {
            record_class: "myapp::Voltage".into(), field: "bogus".into(), available: vec!["volts".into()],
        })),
        ("NoMatchingClause", mk(RuntimeErrorKind::NoMatchingClause {
            name: ":my::process".into(),
            called_arity: 1,
            called_args: vec![snap(Value::i64(42))],
            attempted_clauses: Box::new(vec![
                ClauseAttempt {
                    clause_index: 0, declared_arity: 2,
                    declared_arg_types: vec!["i64".into(), "i64".into()],
                    failure_reason: ClauseFailureReason::ArityMismatch { expected: 2, got: 1 },
                },
                ClauseAttempt {
                    clause_index: 1, declared_arity: 1,
                    declared_arg_types: vec!["string".into()],
                    failure_reason: ClauseFailureReason::ArgTypeMismatch { position: 0, expected: "string".into(), got: "i64".into() },
                },
                ClauseAttempt {
                    clause_index: 2, declared_arity: 1,
                    declared_arg_types: vec!["i64".into()],
                    failure_reason: ClauseFailureReason::GuardFalse,
                },
            ]),
        })),
        ("PostconditionFailed", mk(RuntimeErrorKind::PostconditionFailed {
            defclause_name: ":my::positive".into(), clause_index: 0,
            ensure_expr_snapshot: "(> result 0)".into(),
            returned_value: Box::new(snap(Value::i64(-5))),
            ensure_span: Box::new(s()),
        })),
        ("MacroAbort", mk(RuntimeErrorKind::MacroAbort { message: "boom".into() })),
        ("WriteStopped", mk(RuntimeErrorKind::WriteStopped)),
        ("ReteDefnAxisViolation", mk(RuntimeErrorKind::ReteDefnAxisViolation {
            name: ":rete::helper".into(), axis: "Pure", head: ":wat::kernel::println".into(),
        })),
        ("ReteDefnRecursive", mk(RuntimeErrorKind::ReteDefnRecursive { name: ":rete::helper".into(), head: ":rete::helper".into() })),
    ]
}

fn record_class_of(v: &Value) -> String {
    match v {
        Value::Aggregate(a) => a.class.to_string(),
        other => panic!("to_record must produce Value::Aggregate, got {other:?}"),
    }
}

// ─── G1 — the declaration is the list ─────────────────────────────────────────

/// The `:wat::runtime::*` `defrecord` names in `wat/runtime-errors.wat`,
/// excluding `ValueSnapshot` and the two `defenum`s (`Provenance`,
/// `ReteCeilingKind`) — G1's own scope per the brief.
fn declared_runtime_record_names() -> std::collections::BTreeSet<String> {
    let root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(root).join("wat/runtime-errors.wat");
    let src = std::fs::read_to_string(&path).expect("wat/runtime-errors.wat must be readable");
    let forms = wat_reader::parse_all_with_file(&src, "wat/runtime-errors.wat").expect("wat/runtime-errors.wat must parse");
    let mut names = std::collections::BTreeSet::new();
    for form in &forms {
        let wat_reader::WatAST::List(items, _) = form else { continue };
        let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
        if head.as_str() != ":wat::core::defrecord" {
            continue;
        }
        let Some(wat_reader::WatAST::Keyword(name, _)) = items.get(1) else { continue };
        let bare = name.as_str().trim_start_matches(":wat::runtime::");
        if bare == "ValueSnapshot" {
            continue;
        }
        names.insert(bare.to_string());
    }
    names
}

#[test]
fn g1_declaration_is_the_list() {
    let produced: std::collections::BTreeSet<String> = all_variants()
        .iter()
        .map(|(_, err)| record_class_of(&err.to_record()).trim_start_matches("wat::runtime::").to_string())
        .collect();
    let declared = declared_runtime_record_names();
    assert_eq!(
        produced, declared,
        "the set of record classes to_record() produces must equal the set of \
         `defrecord :wat::runtime::*` names in wat/runtime-errors.wat (excl. ValueSnapshot)"
    );
}

// ─── G2 — the record agrees with today's wire ─────────────────────────────────

/// Ruling 2026-09-26 (Retag), extended by excursus 003 strike B1's `cause` handling
/// (`is_nested_error_variant` below, not a G2_EXCEPTIONS class — the mismatch is a
/// FIELD NAME change too, `:error`/`:cause` on the wire vs. `:cause` in `to_record()`,
/// which a (variant, field) pair keyed on ONE name cannot express): exceptions between
/// `to_record`'s render and `WatError::error_edn()`'s wire, applied wherever that shape
/// occurs — enumerated here by (variant, field) so an unlisted difference still goes RED.
///
/// Strike B2, item 5: `NoMatchingClause.attempted-clauses` used to be listed here too
/// (`ClauseAttempt`'s nested `ClauseFailureReason` variant tag rode flat on the old
/// wire, dotted in `to_record()`) — RETIRED once `error_edn()`'s
/// `clause_failure_reason_to_edn` started emitting the same dotted tag `to_record()`
/// always did (`src/edn/error.rs`); the two writers agree now, so the field is caught
/// by the generic byte-equality check below, not an exception.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ExceptionClass {
    /// The old wire carried this payload untagged (`ValueSnapshot`), or `ReteCeiling.
    /// ceiling`'s own variant tag was already tagged (flat) and only its namespace-
    /// qualification changed (flat -> dotted); the record declares it, so it renders
    /// tagged/dotted now.
    Retag,
}

/// Excursus 003 strike B1, item 4: the two wrapping kinds whose wire field
/// (`EvalVerificationFailed`'s `:error`, `MacroExpansionFailed`'s `:cause`) carries the
/// wrapped error's OWN rich render, while `to_record()`'s `cause` field name differs on
/// the wire too (`:error` -> `:cause` for `EvalVerificationFailed`), which a (variant,
/// ONE field) pair cannot express — handled below as its own pass, not a G2_EXCEPTIONS
/// class.
///
/// Strike B2 item 3 CLOSED `EvalVerificationFailed`'s half of this gap:
/// `to_record()`'s `cause` now holds the REAL `:wat::kernel::HashError` record
/// (`hash_error_value`, `src/value/runtime_records.rs`) — byte-identical to the wire's
/// `:error` (both come from the SAME `HashError` value now that it carries its own
/// `message`/`location` floor), so `is_nested_error_variant` routes it to an EQUALITY
/// check, not the Fault-shape check. `MacroExpansionFailed` keeps the Fault gap
/// (`single_cause_fault`'s doc comment, `src/value/runtime_records.rs`) — strike B3's
/// scope fence names it explicitly as still open.
fn is_nested_error_variant(variant: &str) -> bool {
    variant == "EvalVerificationFailed" || variant == "MacroExpansionFailed"
}

/// `EvalVerificationFailed` closed its gap (see above): `to_record()`'s `cause` is now
/// REQUIRED to equal the wire's `:error`, byte-for-byte, not just Fault-shaped.
fn is_closed_nested_error_variant(variant: &str) -> bool {
    variant == "EvalVerificationFailed"
}

/// The wire's own field name for the nested error, per `is_nested_error_variant`.
fn nested_error_wire_field(variant: &str) -> &'static str {
    match variant {
        "EvalVerificationFailed" => "error",
        "MacroExpansionFailed" => "cause",
        other => panic!("nested_error_wire_field: not a nested-error variant: {other}"),
    }
}

const G2_EXCEPTIONS: &[(&str, &str, ExceptionClass)] = &[
    ("NotCallable", "got", ExceptionClass::Retag),
    ("TypeMismatch", "got", ExceptionClass::Retag),
    ("BadCondition", "got", ExceptionClass::Retag),
    ("NoMatchingClause", "called-args", ExceptionClass::Retag),
    ("PostconditionFailed", "returned-value", ExceptionClass::Retag),
    // `ReteCeiling.ceiling` was ALREADY tagged on the old wire (the derive's own
    // `#[to_edn(namespace = RUNTIME)]` on the nested `ReteCeiling` enum) — the
    // retag here is flat `#wat.runtime/<Variant>` -> dotted
    // `#wat.runtime/ReteCeilingKind.<Variant>`, not untagged -> tagged. Still
    // class 1 (a payload the record now declares under its own registered
    // type, `:wat::runtime::ReteCeilingKind`), so it is not a third class —
    // `assert_retagged` accepts either starting shape and only requires the
    // NEW side end up properly tagged and DIFFERENT from the old.
    ("ReteCeiling", "ceiling", ExceptionClass::Retag),
];

fn as_tagged_map(v: &OwnedValue) -> (&wat_edn::Tag, Vec<(&OwnedValue, &OwnedValue)>) {
    match v {
        OwnedValue::Tagged(tag, body) => match body.as_ref() {
            OwnedValue::Map(fields) => (tag, fields.iter().map(|(k, v)| (k, v)).collect()),
            OwnedValue::Nil => (tag, Vec::new()),
            other => panic!("expected a tagged map or bodyless tag, got {other:?}"),
        },
        other => panic!("expected a Tagged value, got {other:?}"),
    }
}

fn key_name(k: &OwnedValue) -> &str {
    match k {
        OwnedValue::Keyword(kw) => kw.name(),
        other => panic!("expected a Keyword map key, got {other:?}"),
    }
}

fn find_field<'a>(fields: &[(&'a OwnedValue, &'a OwnedValue)], name: &str) -> Option<&'a OwnedValue> {
    fields.iter().find(|(k, _)| key_name(k) == name).map(|(_, v)| *v)
}

#[test]
fn g2_record_agrees_with_wire() {
    for (variant, err) in all_variants() {
        let record = err.to_record();
        let actual = value_to_edn_with(&record, None).expect("to_record() must encode");
        let wire = err.error_edn();

        let (actual_tag, actual_fields) = as_tagged_map(&actual);
        let (wire_tag, wire_fields_raw) = as_tagged_map(&wire);
        // Drop :frames/:frames-elided — captured trace, out of scope for this step.
        let wire_fields: Vec<(&OwnedValue, &OwnedValue)> = wire_fields_raw
            .into_iter()
            .filter(|(k, _)| key_name(k) != "frames" && key_name(k) != "frames-elided")
            .collect();

        assert_eq!(actual_tag.name(), wire_tag.name(), "{variant}: tag name must match the wire's");

        let exceptions: Vec<&(&str, &str, ExceptionClass)> =
            G2_EXCEPTIONS.iter().filter(|(v, _, _)| *v == variant).collect();

        let nested_error = is_nested_error_variant(variant);

        // Every wire field not named as an exception must appear, byte-identical, in `actual`.
        for (k, v) in &wire_fields {
            let name = key_name(k);
            if nested_error && name == nested_error_wire_field(variant) {
                if is_closed_nested_error_variant(variant) {
                    // Strike B2 item 3: the gap is CLOSED for this variant — `to_record()`'s
                    // `cause` must equal the wire's value byte-for-byte (only the KEY name
                    // differs: `:error` on the wire, `:cause` in the record).
                    let actual_v = find_field(&actual_fields, "cause").unwrap_or_else(|| {
                        panic!("{variant}: to_record() must carry a `cause` field equal to the wire's `{name}`")
                    });
                    assert_eq!(
                        actual_v, *v,
                        "{variant}: to_record()'s `cause` must equal the wire's `{name}` byte-for-byte now that HashError carries its own floor"
                    );
                }
                continue; // MacroExpansionFailed: handled below — to_record()'s `cause` is a
                          // KNOWN, intentionally flattened Fault gap (strike B3), not equal
                          // to the wire's rich nested render.
            }
            if let Some((_, _, class)) = exceptions.iter().find(|(_, f, _)| *f == name) {
                match class {
                    ExceptionClass::Retag => {
                        // Checked in the retag pass below; here we only assert it did NOT
                        // vanish (a value must be present under this name in `actual`).
                        assert!(
                            find_field(&actual_fields, name).is_some(),
                            "{variant}: field `{name}` is a listed retag exception but to_record() has no such field at all"
                        );
                    }
                }
                continue;
            }
            let actual_v = find_field(&actual_fields, name);
            assert_eq!(
                actual_v, Some(*v),
                "{variant}: field `{name}` unexpectedly differs from the wire and has no listed G2 exception"
            );
        }

        // No field in `actual` beyond the floor may be absent from the wire's set
        // (a NEW field with no G2 exception would be an unnamed, undocumented addition).
        for (k, _) in &actual_fields {
            let name = key_name(k);
            if name == "message" || name == "location" {
                continue;
            }
            if nested_error && name == "cause" {
                continue; // handled below
            }
            let on_wire = find_field(&wire_fields, name).is_some();
            let is_exception = exceptions.iter().any(|(_, f, _)| *f == name);
            assert!(
                on_wire || is_exception,
                "{variant}: to_record() has field `{name}` the wire never had, and it is not a listed G2 exception"
            );
        }

        // Excursus 003 strike B1, item 4's KNOWN GAP: `MacroExpansionFailed` still carries
        // a `cause` field in `to_record()` that is a bare `:wat::core::Fault` (structurally
        // a `:wat::core::Error`, `{message location}`, nothing more), not the wrapped
        // type's own declared shape — strike B3's scope (the equality check above already
        // handled `EvalVerificationFailed`, whose gap strike B2 item 3 closed). Every other
        // variant carries no `cause` field at all.
        if nested_error && !is_closed_nested_error_variant(variant) {
            let actual_cause = find_field(&actual_fields, "cause")
                .unwrap_or_else(|| panic!("{variant}: to_record() must carry a `cause` field"));
            let (fault_tag, fault_fields) = as_tagged_map(actual_cause);
            assert_eq!(fault_tag.name(), "Fault", "{variant}: cause must be a Fault");
            assert!(find_field(&fault_fields, "message").is_some(), "{variant}: the cause Fault must carry a message");
            assert!(find_field(&fault_fields, "location").is_some(), "{variant}: the cause Fault must carry a location");
            assert_eq!(fault_fields.len(), 2, "{variant}: :wat::core::Fault is {{message location}} now (excursus 003 strike B1) — no third field");
        } else if !nested_error {
            assert!(find_field(&actual_fields, "cause").is_none(), "{variant}: only the two wrapping kinds carry a `cause` field");
        }

        // Retag pass: for every listed Retag exception, the OLD wire value must be
        // UNTAGGED (a bare Map, or a Vector of bare Maps) and the NEW value must be
        // TAGGED (a Tagged value, or a Vector of Tagged values) — proving the retag
        // actually happened, not merely that the field survived.
        for (_, field, class) in &exceptions {
            if *class != ExceptionClass::Retag {
                continue;
            }
            let old_v = find_field(&wire_fields, field).unwrap_or_else(|| panic!("{variant}: wire has no field `{field}` to retag"));
            let new_v = find_field(&actual_fields, field).unwrap_or_else(|| panic!("{variant}: to_record() has no field `{field}` to retag"));
            assert_retagged(variant, field, old_v, new_v);
        }
    }
}

// ─── Excursus 003 strike B2, GB2a — ClauseFailureReason is dotted on the WIRE ─────
//
// `g2_record_agrees_with_wire` (above) proves `error_edn()` and `to_record()` now
// agree byte-for-byte on `attempted-clauses` (the stale G2_EXCEPTIONS entry for it is
// gone). This gate is the independent, direct proof item 5's target asks for: drive
// `error_edn()` (the WIRE writer, `clause_failure_reason_to_edn`) for each of
// `ClauseFailureReason`'s three variants, and assert — without reference to
// `to_record()` at all — that the tag is `#wat.kernel/ClauseFailureReason.<Variant>`
// and decodes, through the GENERAL tag-driven decoder (`edn_to_value`), typed AS THE
// ENUM (`Value::Enum` with `type_path == ":wat::kernel::ClauseFailureReason"`), not a
// generic untyped map.
//
// Mutation (strike report): revert `clause_failure_reason_to_edn`'s `ArityMismatch` arm
// from `edn_tag_dotted("ClauseFailureReason", "ArityMismatch", ...)` back to the old
// `tagged("ArityMismatch", ...)` (flat) — this gate goes RED for that one variant
// (`split_variant_tag_name` finds no `.` and `edn_to_value` falls back to a generic
// untagged-map-shaped decode, so the `Value::Enum` assertion fails), while the other
// two variants stay green: proof the gate is reading the ACTUAL writer, not a
// structural accident.
#[test]
fn gate_gb2a_clause_failure_reason_wire_is_dotted() {
    let types = TypeEnv::with_builtins();

    let cases: [(&str, ClauseFailureReason); 3] = [
        ("ArityMismatch", ClauseFailureReason::ArityMismatch { expected: 2, got: 1 }),
        (
            "ArgTypeMismatch",
            ClauseFailureReason::ArgTypeMismatch {
                position: 0,
                expected: "string".into(),
                got: "i64".into(),
            },
        ),
        ("GuardFalse", ClauseFailureReason::GuardFalse),
    ];

    for (variant_name, reason) in cases {
        let err = RuntimeError::new(
            s(),
            RuntimeErrorKind::NoMatchingClause {
                name: ":user::f".into(),
                called_arity: 1,
                called_args: vec![snap(Value::i64(1))],
                attempted_clauses: Box::new(vec![ClauseAttempt {
                    clause_index: 0,
                    declared_arity: 1,
                    declared_arg_types: vec!["i64".into()],
                    failure_reason: reason,
                }]),
            },
        );
        let wire = err.error_edn();
        let (_, wire_fields) = as_tagged_map(&wire);
        let attempted = find_field(&wire_fields, "attempted-clauses")
            .unwrap_or_else(|| panic!("{variant_name}: wire has no attempted-clauses"));
        let OwnedValue::Vector(attempts) = attempted else {
            panic!("{variant_name}: attempted-clauses must be a vector, got {attempted:?}");
        };
        let (_, attempt_fields) = as_tagged_map(&attempts[0]);
        let reason_v = find_field(&attempt_fields, "failure-reason")
            .unwrap_or_else(|| panic!("{variant_name}: ClauseAttempt has no failure-reason"));

        let (reason_tag, _) = as_tagged_map(reason_v);
        assert_eq!(reason_tag.namespace(), "wat.kernel", "{variant_name}: namespace must stay wat.kernel");
        assert_eq!(
            reason_tag.name(),
            format!("ClauseFailureReason.{variant_name}"),
            "{variant_name}: wire tag must be dotted #wat.kernel/ClauseFailureReason.{variant_name}, \
             not the flat #wat.kernel/{variant_name}"
        );

        let decoded = edn_to_value(reason_v, Some(&types), None)
            .unwrap_or_else(|e| panic!("{variant_name}: dotted wire tag must decode: {e:?}"));
        match decoded {
            Value::Enum(ev) => {
                assert_eq!(ev.type_path, ":wat::kernel::ClauseFailureReason", "{variant_name}: must decode AS THE ENUM");
                assert_eq!(ev.variant_name, variant_name);
            }
            other => panic!("{variant_name}: expected Value::Enum, got {other:?}"),
        }
    }
}

// ─── Excursus 003 strike B2, item 2 — LoadFetchError is dotted on the WIRE ────
//
// Mirrors `gate_gb2a_clause_failure_reason_wire_is_dotted` above, for the other
// data (non-Error) sum type item 2 converts: drives `LoadErrorKind::Fetch`'s
// hand-written `ToEdn` (`LoadFetchError`, `src/load/loader.rs` — kept
// hand-written because `Other`'s wire tag renames to `LoadOther`, which the
// derive's `qualified` directive cannot express) for all 3 variants, and
// asserts the tag is `#wat.kernel/LoadFetchError.<Variant>` and decodes, via
// `LoadErrorKind::Fetch.cause`'s now-concrete field type
// (`:wat::kernel::LoadFetchError`, not `:wat::core::Value`), typed AS THE
// ENUM.
//
// Mutation (this strike): revert `LoadFetchError::NotFound`'s arm from
// `edn_tag_dotted("LoadFetchError", "NotFound", ...)` back to the old
// `edn_tag("NotFound", ...)` (flat) — RED for that one variant, green for the
// other two, proving the gate reads the ACTUAL writer.
#[test]
fn gate_gb2a_load_fetch_error_wire_is_dotted() {
    use wat::load::loader::{LoadError, LoadErrorKind, LoadFetchError};

    let types = TypeEnv::with_builtins();

    let cases: [(&str, LoadFetchError); 3] = [
        ("NotFound", LoadFetchError::NotFound("missing.wat".into())),
        ("LoadOther", LoadFetchError::Other { path: "x.wat".into(), reason: "boom".into() }),
        ("OutOfScope", LoadFetchError::OutOfScope { path: "../x.wat".into(), scope: "/root".into() }),
    ];

    for (variant_name, fetch_err) in cases {
        let err = LoadError::new(s(), LoadErrorKind::Fetch(fetch_err));
        let wire = err.error_edn();
        let (_, wire_fields) = as_tagged_map(&wire);
        let cause_v = find_field(&wire_fields, "cause")
            .unwrap_or_else(|| panic!("{variant_name}: wire has no cause field"));

        let (cause_tag, _) = as_tagged_map(cause_v);
        assert_eq!(cause_tag.namespace(), "wat.kernel", "{variant_name}: namespace must stay wat.kernel");
        assert_eq!(
            cause_tag.name(),
            format!("LoadFetchError.{variant_name}"),
            "{variant_name}: wire tag must be dotted #wat.kernel/LoadFetchError.{variant_name}, \
             not the flat #wat.kernel/{variant_name}"
        );

        let decoded = edn_to_value(cause_v, Some(&types), None)
            .unwrap_or_else(|e| panic!("{variant_name}: dotted wire tag must decode: {e:?}"));
        match decoded {
            Value::Enum(ev) => {
                assert_eq!(ev.type_path, ":wat::kernel::LoadFetchError", "{variant_name}: must decode AS THE ENUM");
                assert_eq!(ev.variant_name, variant_name);
            }
            other => panic!("{variant_name}: expected Value::Enum, got {other:?}"),
        }
    }
}

/// Excursus 003 strike B2, item 2 GB2b — `LoadErrorKind::Fetch.cause` is typed
/// (`:wat::kernel::LoadFetchError`, not `:wat::core::Value`): a `Fetch` record
/// whose `:cause` is shaped for the WRONG enum (`ClauseFailureReason`, a
/// sibling `defenum` with its own dotted tags) must be REFUSED, whole, at
/// typed decode — never silently accepted the way an untyped `Value`-typed
/// field would (tag-driven decode consults only the TAG, not the declared
/// field type, so this is the one gate that proves the declared type is
/// actually consulted).
///
/// Mutation (recorded for the builder, small enough to state rather than drive
/// twice in CI): retype `Fetch.cause` back to `:wat::core::Value` in
/// `wat/load-errors.wat` — this assertion would go RED (decode would succeed
/// where it must fail) since an untyped field accepts any tagged value.
#[test]
fn gate_gb2b_load_fetch_error_field_is_typed_not_value() {
    use wat::edn::render::EdnReadErrorKind;
    use wat::load::loader::{LoadError, LoadErrorKind, LoadFetchError};

    let types = TypeEnv::with_builtins();
    let err = LoadError::new(
        s(),
        LoadErrorKind::Fetch(LoadFetchError::NotFound("missing.wat".into())),
    );
    let wire = err.error_edn();
    let (fetch_tag, fetch_fields) = as_tagged_map(&wire);

    // Swap the real (correctly-typed) `:cause` value for a WRONG enum's — a
    // STRUCTURALLY built dotted tag (`ClauseFailureReason.GuardFalse`, a
    // sibling `defenum`), never an inlined EDN string literal.
    let wrong_enum_value = OwnedValue::Tagged(
        wat_edn::Tag::ns("wat.kernel", "ClauseFailureReason.GuardFalse"),
        Box::new(OwnedValue::Map(Vec::new())),
    );
    let mut mutated_fields: Vec<(OwnedValue, OwnedValue)> = Vec::new();
    let mut swapped = false;
    for (k, v) in fetch_fields {
        if key_name(k) == "cause" {
            mutated_fields.push((k.clone(), wrong_enum_value.clone()));
            swapped = true;
        } else {
            mutated_fields.push((k.clone(), v.clone()));
        }
    }
    assert!(swapped, "fixture assumption broken: `Fetch` wire has no `:cause` field to swap");
    let mutated = OwnedValue::Tagged(fetch_tag.clone(), Box::new(OwnedValue::Map(mutated_fields)));

    let decoded = edn_to_value(&mutated, Some(&types), None);
    match decoded {
        Err(e) => match e.kind {
            EdnReadErrorKind::FieldTypeMismatch { ref field, .. } => {
                assert_eq!(
                    field, "cause",
                    "must name `cause` as the mismatched field; got {:?}", e.kind
                );
            }
            other => panic!(
                "a `Fetch` record whose `:cause` is a ClauseFailureReason value must be \
                 refused as FieldTypeMismatch on `cause` — got a different EdnReadErrorKind: {other:?}"
            ),
        },
        Ok(v) => panic!(
            "a `Fetch` record whose `:cause` is a ClauseFailureReason value must be REFUSED — \
             `:wat::kernel::LoadFetchError` is the declared field type; decoded as {v:?}"
        ),
    }
}

// ─── Excursus 003 strike B2, item 2 — EnsureFnInvalidReason is dotted ────────
//
// Mirrors `gate_gb2a_load_fetch_error_wire_is_dotted` above, for the third
// item-2 sum type. `CheckErrorKind::EnsureFnInvalid.reason` is a genuine
// `defenum` now (`:wat::check::EnsureFnInvalidReason`, moved to the
// `wat.check` namespace), its 5 variants dot-joined by the DERIVE's own
// `qualified` directive (`#[to_edn(namespace = crate::error_ns::CHECK,
// qualified)]`, `src/check/error.rs`) — no hand-written writer at all.
//
// Mutation (this strike): drop `qualified` from `EnsureFnInvalidReason`'s
// derive attribute — every variant's tag reverts to flat
// (`#wat.check/<Variant>`), RED for all 5 (unlike the hand-written sum
// types above, the derive has no per-variant granularity to revert just one).
#[test]
fn gate_gb2a_ensure_fn_invalid_reason_wire_is_dotted() {
    use wat::check::error::{CheckError, CheckErrorKind, EnsureFnInvalidReason};

    let types = TypeEnv::with_builtins();
    let cases: [(&str, EnsureFnInvalidReason); 5] = [
        ("NotFnForm", EnsureFnInvalidReason::NotFnForm),
        ("ArityNotOne", EnsureFnInvalidReason::ArityNotOne { got: 2 }),
        (
            "ArgTypeMismatch",
            EnsureFnInvalidReason::ArgTypeMismatch {
                arg_type: ":wat::core::bool".into(),
                clause_return_type: ":wat::core::i64".into(),
            },
        ),
        ("ReturnTypeNotBool", EnsureFnInvalidReason::ReturnTypeNotBool { got: ":wat::core::i64".into() }),
        ("MalformedSignature", EnsureFnInvalidReason::MalformedSignature),
    ];

    for (variant_name, reason) in cases {
        let err = CheckError {
            span: s(),
            kind: CheckErrorKind::EnsureFnInvalid {
                defclause_name: ":user::f".into(),
                clause_index: 0,
                reason,
            },
        };
        let wire = err.error_edn();
        let (_, wire_fields) = as_tagged_map(&wire);
        let reason_v = find_field(&wire_fields, "reason")
            .unwrap_or_else(|| panic!("{variant_name}: wire has no reason field"));

        let (reason_tag, _) = as_tagged_map(reason_v);
        assert_eq!(reason_tag.namespace(), "wat.check", "{variant_name}: namespace must be wat.check, not wat.kernel");
        assert_eq!(
            reason_tag.name(),
            format!("EnsureFnInvalidReason.{variant_name}"),
            "{variant_name}: wire tag must be dotted #wat.check/EnsureFnInvalidReason.{variant_name}, \
             not the flat #wat.kernel/{variant_name}"
        );

        let decoded = edn_to_value(reason_v, Some(&types), None)
            .unwrap_or_else(|e| panic!("{variant_name}: dotted wire tag must decode: {e:?}"));
        match decoded {
            Value::Enum(ev) => {
                assert_eq!(ev.type_path, ":wat::check::EnsureFnInvalidReason", "{variant_name}: must decode AS THE ENUM");
                assert_eq!(ev.variant_name, variant_name);
            }
            other => panic!("{variant_name}: expected Value::Enum, got {other:?}"),
        }
    }
}

/// Excursus 003 strike B2, item 2 GB2b — `CheckErrorKind::EnsureFnInvalid.reason`
/// is typed (`:wat::check::EnsureFnInvalidReason`, not `:wat::core::Value`): a
/// `EnsureFnInvalid` record whose `:reason` is shaped for the WRONG enum
/// (`LoadFetchError`, a sibling `defenum` with its own dotted tags, built
/// structurally — never an inlined EDN string) must be REFUSED, whole.
///
/// Mutation (recorded for the builder): retype `EnsureFnInvalid.reason` back
/// to `:wat::core::Value` in `wat/check-errors.wat` — this assertion would go
/// RED (decode would succeed where it must fail).
#[test]
fn gate_gb2b_ensure_fn_invalid_reason_field_is_typed_not_value() {
    use wat::check::error::{CheckError, CheckErrorKind, EnsureFnInvalidReason};
    use wat::edn::render::EdnReadErrorKind;

    let types = TypeEnv::with_builtins();
    let err = CheckError {
        span: s(),
        kind: CheckErrorKind::EnsureFnInvalid {
            defclause_name: ":user::f".into(),
            clause_index: 0,
            reason: EnsureFnInvalidReason::NotFnForm,
        },
    };
    let wire = err.error_edn();
    let (ensure_tag, ensure_fields) = as_tagged_map(&wire);

    let wrong_enum_value = OwnedValue::Tagged(
        wat_edn::Tag::ns("wat.kernel", "LoadFetchError.NotFound"),
        Box::new(OwnedValue::Map(vec![(
            OwnedValue::Keyword(wat_edn::Keyword::new("path")),
            OwnedValue::String("x.wat".into()),
        )])),
    );
    let mut mutated_fields: Vec<(OwnedValue, OwnedValue)> = Vec::new();
    let mut swapped = false;
    for (k, v) in ensure_fields {
        if key_name(k) == "reason" {
            mutated_fields.push((k.clone(), wrong_enum_value.clone()));
            swapped = true;
        } else {
            mutated_fields.push((k.clone(), v.clone()));
        }
    }
    assert!(swapped, "fixture assumption broken: `EnsureFnInvalid` wire has no `:reason` field to swap");
    let mutated = OwnedValue::Tagged(ensure_tag.clone(), Box::new(OwnedValue::Map(mutated_fields)));

    let decoded = edn_to_value(&mutated, Some(&types), None);
    match decoded {
        Err(e) => match e.kind {
            EdnReadErrorKind::FieldTypeMismatch { ref field, .. } => {
                assert_eq!(
                    field, "reason",
                    "must name `reason` as the mismatched field; got {:?}", e.kind
                );
            }
            other => panic!(
                "an `EnsureFnInvalid` record whose `:reason` is a LoadFetchError value must be \
                 refused as FieldTypeMismatch on `reason` — got a different EdnReadErrorKind: {other:?}"
            ),
        },
        Ok(v) => panic!(
            "an `EnsureFnInvalid` record whose `:reason` is a LoadFetchError value must be \
             REFUSED — `:wat::check::EnsureFnInvalidReason` is the declared field type; decoded as {v:?}"
        ),
    }
}

fn assert_tagged(variant: &str, field: &str, v: &OwnedValue) {
    match v {
        OwnedValue::Tagged(_, _) => {}
        OwnedValue::Vector(items) => {
            for item in items {
                assert!(matches!(item, OwnedValue::Tagged(_, _)), "{variant}.{field}: expected the NEW to_record() element to be tagged, got {item:?}");
            }
        }
        other => panic!("{variant}.{field}: expected the NEW to_record() value to be tagged (or a vector of tagged), got {other:?}"),
    }
}

/// A `Retag` exception covers two starting shapes: `ValueSnapshot`/
/// `ClauseAttempt`'s nested reason were UNTAGGED or flat-namespaced on the old
/// wire; `ReteCeiling.ceiling` was ALREADY tagged (flat) on the old wire. Both
/// converge on the same proof obligation: the NEW side is tagged (or a vector
/// of tagged values) under the record's own declared type, and it is not
/// byte-identical to the old side (the retag actually happened).
fn assert_retagged(variant: &str, field: &str, old_v: &OwnedValue, new_v: &OwnedValue) {
    assert_tagged(variant, field, new_v);
    assert_ne!(
        old_v, new_v,
        "{variant}.{field}: listed as a G2 retag exception, but the old wire and to_record() render IDENTICALLY — the exception is stale"
    );
}

// ─── Excursus 003 step 3c, Gate B — the wire carries no standalone frames ───────
//
// `RuntimeError::error_edn()` (`WatError`, `src/edn/error.rs`) is THE writer on the
// process wire — `to_wire_edn`/`Debug`/`Display` all resolve to it, and it is what
// `LociDiedError`/`Failure`/`StartupError` embed a `RuntimeError` cause through
// (`error_edn_of`/`error_edn_of_boxed`). `impl ToEdn for RuntimeError::to_edn()` (the
// derive-generated `:span`/`:frames`/`:frames-elided` shape) is a SEPARATE,
// deliberately-kept representation — not retired, because ~30 golden-backed
// regression tests (`probe_arc298_3_runtime_derive_identical`,
// `probe_stone_233_3_runtime_error_edn`, `probe_arc237_stone4_rich_errors`,
// `probe_arc296_typed_causes`, `probe_arc296_macro_error_is_structured_edn`, …) pin it
// as the derive's byte-identical output (arc 298.3's own migration proof), and
// `StartupError::to_edn_values`'s `--check-output edn|json` reads it for a
// `StartupError::Runtime` (a registration-time failure, not a mid-execution crash —
// see the strike report's production-caller census). A SECOND writer therefore still
// exists on paper, but it is unreachable from the crash-reporting wire: nothing that
// serves `to_wire_edn`/`error_edn`/a `Failure`'s embedded cause calls `.to_edn()` on a
// bare `RuntimeError` — every site above calls it directly on a purpose-built,
// hand-constructed error and asserts against ITS OWN named golden, so a lint pinning
// "unreachable from the wire" would duplicate what this gate already proves for the
// wire's own shape. This gate is the standing proof for the ONE writer's own contract:
// no `:span`, no `:frames`, no `:frames-elided` — those live on `:wat::kernel::Failure`
// alone (step 3b).
#[test]
fn gate_b_wire_carries_no_standalone_frames_or_span() {
    for (variant, err) in all_variants() {
        let wire = err.error_edn();
        let (_, fields) = as_tagged_map(&wire);
        assert!(
            find_field(&fields, "span").is_none(),
            "{variant}: error_edn() (the wire) must not carry :span — :location is the \
             floor's only location key"
        );
        assert!(
            find_field(&fields, "frames").is_none(),
            "{variant}: error_edn() (the wire) must not carry :frames — frames live on \
             :wat::kernel::Failure alone (excursus 003 step 3c)"
        );
        assert!(
            find_field(&fields, "frames-elided").is_none(),
            "{variant}: error_edn() (the wire) must not carry :frames-elided — it lives on \
             :wat::kernel::Failure alone (excursus 003 step 3c)"
        );
        assert!(
            find_field(&fields, "location").is_some(),
            "{variant}: error_edn() (the wire) must always carry :location"
        );
    }
}

// ─── G3 — round trip ───────────────────────────────────────────────────────────

#[test]
fn g3_round_trip() {
    let types = TypeEnv::with_builtins();
    for (variant, err) in all_variants() {
        let original = err.to_record();
        let edn = value_to_edn_with(&original, None).unwrap_or_else(|e| panic!("{variant}: to_record() must encode: {e:?}"));
        let text = wat_edn::write(&edn);
        let reparsed = wat_edn::parse_owned(&text).unwrap_or_else(|e| panic!("{variant}: EDN must re-parse: {e:?}"));
        let decoded = edn_to_value(&reparsed, Some(&types), None)
            .unwrap_or_else(|e| panic!("{variant}: typed decode of its :wat::runtime record must resolve: {e:?}"));
        assert_eq!(decoded, original, "{variant}: round-tripped record must equal the one written");
    }
}
