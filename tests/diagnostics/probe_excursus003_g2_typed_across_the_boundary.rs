//! Excursus 003 step 3b, gate G2 — "typed across the boundary": a runtime crash crosses the
//! peer wire as a real record, never a masked string.
//!
//! See the co-located `.wat` fixture for the EDN-over-stdio rationale (`docs/CONVENTIONS.md`
//! § Test idioms — a crash / dying declaration is a real-process claim).
//!
//! **Mutation** (recorded here, not left to memory; run in RELEASE):
//! `src/runtime.rs`'s `runtime_error_failure` — change
//! `let error_field = re.to_record();` to
//! `let error_field = Value::String(Arc::new(crate::edn::contract::to_wire_edn(re)));`
//! (reintroducing the double-quoting mask this excursus exists to kill). Confirmed RED: the
//! child's own `(:wat::kernel::LociDiedError/message cause)` call refuses loudly at the WAT
//! level (`#wat.runtime/TypeMismatch … "expected :wat::kernel::Failure inside *DiedError
//! variant, got non-Failure payload"`), which `call_beside_value` surfaces as `Err`, failing
//! this test's own `.expect(...)` — a loud structural refusal, not a silent pass.
//!
//! **Measured, not used** — the brief's own suggested mutation ("remove the `DivisionByZero`
//! registration", i.e. comment out `src/types.rs`'s
//! `wat_record_from!(env, "wat/runtime-errors.wat", ":wat::runtime::DivisionByZero")` inside
//! `register_builtin_types`) does NOT redden this gate: `runtime_error_failure` builds
//! `failure.error` via `re.to_record()`, which resolves its field names through
//! `division_by_zero_names()` — a COMPILE-TIME constant baked in by the `wat_field_names_from!`
//! proc macro reading `wat/runtime-errors.wat` directly — never through the runtime `TypeEnv`
//! `register_builtin_types` populates. Driven: with that one registration line commented out,
//! this test still passed, `cause.variant_name` still read `"RuntimeError"`, and
//! `error.class` still read `"wat::runtime::DivisionByZero"` (verified with a temporary debug
//! print of the decoded `Value`, then reverted). Item 4's "must not silently become Panic"
//! guard (`loci_died_error_from_reason`, `src/kernel/error.rs`) is real and exercised by other
//! tests in this suite, but THIS specific registration is not its trigger — filed as an open
//! question rather than forced through with an unrepresentative mutation.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn a_child_division_by_zero_crosses_the_boundary_as_a_typed_runtime_error() {
    let v = call_beside_value(file!(), ":g2::divzero-report")
        .expect(":g2::divzero-report should run and return its Result value, never raise");

    let result = match &v {
        Value::Aggregate(a) => a,
        other => panic!("expected a :g2::Result record; got {other:?}"),
    };
    // :g2::Result field order: [cause, message].
    let cause = match &result.fields[0] {
        Value::Enum(ev) => ev.as_ref(),
        other => panic!("expected :g2::Result.cause to be an enum; got {other:?}"),
    };
    let wat_message = match &result.fields[1] {
        Value::String(s) => s.to_string(),
        other => panic!("expected :g2::Result.message to be a String; got {other:?}"),
    };

    assert_eq!(
        cause.type_path, ":wat::kernel::LociDiedError",
        "the recv' Lost cause must be a LociDiedError"
    );
    // THE GATE: the death must be reported as RuntimeError, not silently masked as an opaque
    // Panic (or any other variant) by a decode failure — item 4 of the brief.
    assert_eq!(
        cause.variant_name, "RuntimeError",
        "a child's integer division by zero must surface as LociDiedError::RuntimeError, not \
         {} — got message {wat_message:?}",
        cause.variant_name
    );

    // RuntimeError.fields = [failure :Failure] (excursus 003 step 3b — one mandatory Failure).
    let failure = match &cause.fields[0] {
        Value::Aggregate(a) if a.class.as_ref() == "wat::kernel::Failure" => a,
        other => panic!("expected RuntimeError.failure to be a Failure record; got {other:?}"),
    };
    // Failure.fields = [error, frames, actual, expected, frames-elided].
    let error = match &failure.fields[0] {
        Value::Aggregate(a) => a,
        other => panic!("expected Failure.error to be a typed record; got {other:?}"),
    };
    // THE GATE: the error is the CLASS `:wat::runtime::DivisionByZero` — a record, not a string
    // rendering of one. A `Value::String` here would be exactly the double-quoting mask G1 hunts
    // for at rest; this is the same claim, driven live.
    assert_eq!(
        error.class.as_ref(),
        "wat::runtime::DivisionByZero",
        "Failure.error must be the DivisionByZero record itself (its class, not its text)"
    );
    // DivisionByZero.fields = [message, location] (excursus 003 strike B1: `causes` left
    // the floor, F3).
    let struct_message = match &error.fields[0] {
        Value::String(s) => s.to_string(),
        other => panic!("expected DivisionByZero.message to be a String; got {other:?}"),
    };

    let frames = match &failure.fields[1] {
        Value::Vec(items) => items,
        other => panic!("expected Failure.frames to be a Vector; got {other:?}"),
    };
    assert!(
        !frames.is_empty(),
        "Failure.frames must be non-empty for a runtime death (the capped wat call-stack \
         snapshot, step 2) — got an empty vector"
    );

    // THE GATE: the DERIVED `LociDiedError/message` (a wat-side call, not a Rust re-projection)
    // must equal the error record's own structural `.message` field exactly — the "one-line
    // headline", not `to_wire_edn` of anything.
    assert_eq!(
        wat_message, struct_message,
        "(:wat::kernel::LociDiedError/message cause) must equal failure.error.message exactly"
    );
}
