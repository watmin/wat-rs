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
use wat::hash::HashError;
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
            err: HashError::Mismatch { algo: "sha256".into(), expected: "aaa".into(), actual: "bbb".into() },
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

/// Ruling 2026-09-26: exactly two CLASSES of exception between `to_record`'s
/// render and `WatError::error_edn()`'s wire, applied wherever that shape
/// occurs — enumerated here by (variant, field) so an unlisted difference
/// still goes RED.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ExceptionClass {
    /// The old wire carried this payload untagged (`ValueSnapshot`) or under
    /// the wrong namespace (`ClauseAttempt`'s nested `ClauseFailureReason`
    /// variant tag, flat `wat.kernel/<Variant>` -> dotted
    /// `wat.kernel/ClauseFailureReason.<Variant>`); the record declares it,
    /// so it renders tagged/dotted now.
    Retag,
    /// The old wire kept this as a kind field; the record moves it into
    /// `causes` as one `:wat::core::Fault`, dropping the field entirely.
    NestedErrorMoved,
}

const G2_EXCEPTIONS: &[(&str, &str, ExceptionClass)] = &[
    ("NotCallable", "got", ExceptionClass::Retag),
    ("TypeMismatch", "got", ExceptionClass::Retag),
    ("BadCondition", "got", ExceptionClass::Retag),
    ("NoMatchingClause", "called-args", ExceptionClass::Retag),
    ("NoMatchingClause", "attempted-clauses", ExceptionClass::Retag),
    ("PostconditionFailed", "returned-value", ExceptionClass::Retag),
    ("EvalVerificationFailed", "error", ExceptionClass::NestedErrorMoved),
    ("MacroExpansionFailed", "cause", ExceptionClass::NestedErrorMoved),
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

        // Every wire field not named as an exception must appear, byte-identical, in `actual`.
        for (k, v) in &wire_fields {
            let name = key_name(k);
            if name == "causes" {
                continue; // causes handled below, as a unit (empty vs. one Fault)
            }
            if let Some((_, _, class)) = exceptions.iter().find(|(_, f, _)| *f == name) {
                match class {
                    ExceptionClass::NestedErrorMoved => {
                        assert!(
                            find_field(&actual_fields, name).is_none(),
                            "{variant}: field `{name}` should have moved into `causes`, but to_record() still carries it"
                        );
                    }
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
            if name == "message" || name == "location" || name == "causes" {
                continue;
            }
            let on_wire = find_field(&wire_fields, name).is_some();
            let is_exception = exceptions.iter().any(|(_, f, _)| *f == name);
            assert!(
                on_wire || is_exception,
                "{variant}: to_record() has field `{name}` the wire never had, and it is not a listed G2 exception"
            );
        }

        // `causes`: empty on both sides, unless this variant has a NestedErrorMoved
        // exception, in which case the wire's is empty and to_record()'s carries
        // exactly one `:wat::core::Fault`.
        let has_nested_error = exceptions.iter().any(|(_, _, c)| *c == ExceptionClass::NestedErrorMoved);
        let actual_causes = find_field(&actual_fields, "causes").expect("floor `causes` must always be present");
        if has_nested_error {
            match actual_causes {
                OwnedValue::Vector(items) if items.len() == 1 => {
                    let (fault_tag, fault_fields) = as_tagged_map(&items[0]);
                    assert_eq!(fault_tag.name(), "Fault", "{variant}: the nested error's cause must be a Fault");
                    assert!(find_field(&fault_fields, "message").is_some(), "{variant}: the cause Fault must carry a message");
                    assert!(find_field(&fault_fields, "location").is_some(), "{variant}: the cause Fault must carry a location");
                    assert!(find_field(&fault_fields, "causes").is_some(), "{variant}: the cause Fault must carry (empty) causes");
                }
                other => panic!("{variant}: expected causes = [one Fault], got {other:?}"),
            }
        } else {
            assert_eq!(actual_causes, &OwnedValue::Vector(Vec::new()), "{variant}: causes must be empty");
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
