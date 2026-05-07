//! Integration tests for `:wat::core::do` — Clojure-faithful sequential
//! evaluation form. Arc 136 slice 1a.
//!
//! Shape: `(:wat::core::do f1 f2 ... fN)`.
//!
//! Semantics:
//!   - Variadic; one or more forms.
//!   - Empty `(do)` → MalformedForm parse error.
//!   - Each non-final form is evaluated for side effect; its result is
//!     DISCARDED. Non-finals' types are unconstrained.
//!   - The FINAL form is evaluated; its value is returned.
//!   - The do form's inferred type IS the final form's inferred type.
//!     Recipient unification at the consuming site (binding slot,
//!     function declared return, argument position) is the static check.
//!
//! No `-> :T` slot — per the FOURTH amendment to the arc 136 DESIGN
//! (and the arc 145 back-out realization), the substrate's existing
//! inference + recipient unification provides the static check.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

fn run_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

fn unwrap_i64(v: Value) -> i64 {
    match v {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// Run a program whose `:user::main` writes to stdout via
/// `IOWriter/println`. Returns captured stdout split by `\n`.
fn run_with_stdout(src: &str) -> Vec<String> {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args).expect("main");
    let bytes = stdout.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

// ─── 1. Empty: (:wat::core::do) → MalformedForm parse error ─────────────

#[test]
fn do_empty_form_is_malformed() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::do))
    "#;
    let err = run_err(src);
    assert!(
        err.contains("do") && (err.contains("MalformedForm") || err.contains("at least one")),
        "expected MalformedForm naming the do form; got: {}",
        err
    );
}

// ─── 2. Single form: (do x) ≡ x ─────────────────────────────────────────

#[test]
fn do_single_form_returns_its_value() {
    // Degenerate single-form do — accepts (matches Clojure's `(do x) => x`).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::do 42))
    "#;
    assert_eq!(unwrap_i64(run(src)), 42);
}

// ─── 3. Multi form: side effects in order; final value returned ─────────

#[test]
fn do_multi_form_evaluates_left_to_right_returns_final() {
    // Three printlns plus a final i64 — the printlns are non-final
    // (results discarded; do permits any non-final type because the
    // value is dropped). Final form returns 99.
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::do
            (:wat::io::IOWriter/println stdout "log-1")
            (:wat::io::IOWriter/println stdout "log-2")
            (:wat::io::IOWriter/println stdout "log-3")))
    "#;
    let out = run_with_stdout(src);
    assert_eq!(out, vec!["log-1", "log-2", "log-3"]);
}

// ─── 4. Type flow at recipient (clean unification) ──────────────────────

#[test]
fn do_recipient_unifies_with_final_form_type() {
    // The probe declares -> :i64; its body is a do form whose final
    // form is 42 (i64). Substrate infers do's type from final = :i64;
    // recipient unification (probe's body slot expects :i64) succeeds.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::i64)
          (:wat::core::do
            (:wat::core::i64::+,2 1 1)
            42))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:my::probe))
    "#;
    assert_eq!(unwrap_i64(run(src)), 42);
}

// ─── 5. Recipient mismatch fires TypeMismatch ───────────────────────────

#[test]
fn do_recipient_mismatch_fires_type_mismatch() {
    // The probe declares -> :wat::core::String; its body is a do form
    // whose final form is 42 (i64). Substrate infers do's type from
    // final = :i64; recipient unification (probe's declared :String)
    // fails → TypeMismatch fires at the recipient.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::String)
          (:wat::core::do
            (:wat::core::i64::+,2 1 1)
            42))

        (:wat::core::define (:user::main -> :wat::core::String)
          (:my::probe))
    "#;
    let err = run_err(src);
    assert!(
        err.contains("TypeMismatch"),
        "expected TypeMismatch at probe's body; got: {}",
        err
    );
}

// ─── 6. Non-final type unconstrained ────────────────────────────────────

#[test]
fn do_non_final_type_is_unconstrained() {
    // The non-final form is a String literal (NOT :unit) — under
    // let*-with-unit-bindings this would have been rejected because
    // each `((_ :unit) form)` slot REQUIRED form to be :unit. The do
    // form is MORE permissive: non-final's value is intentionally
    // discarded; its type is unconstrained. Final form's i64 is the do's type.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::do
            "string-not-unit"
            (:wat::core::i64::+,2 1 1)
            42))
    "#;
    assert_eq!(unwrap_i64(run(src)), 42);
}

// ─── 7. Reflection round-trip via signature-of ──────────────────────────

#[test]
fn do_reflection_round_trip_emits_variadic_sketch() {
    // `(:wat::runtime::signature-of :wat::core::do)` should return
    // Some(<HolonAST>) carrying the registered sketch. The sketch's
    // bundle head is `:wat::core::do` and the slot is `<form>+` (the
    // variadic placeholder).
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((sig-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::signature-of :wat::core::do))
             ((rendered :wat::core::String)
              (:wat::edn::write sig-opt)))
            (:wat::io::IOWriter/println stdout rendered)))
    "##;
    let out = run_with_stdout(src);
    assert_eq!(out.len(), 1, "expected one rendered line, got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":wat::core::do"),
        "expected do keyword as signature head; got: {}",
        line
    );
    assert!(
        line.contains("<form>+"),
        "expected variadic <form>+ slot in signature; got: {}",
        line
    );
}

// ─── 8. Tail-call sanity: do in tail position preserves TCO ─────────────

#[test]
fn do_in_tail_position_preserves_tail_call() {
    // Tail-recursive countdown whose recursive call is the final form
    // of a do. Without TCO threading through eval_do_tail, this would
    // overflow the stack at this depth.
    let src = r#"
        (:wat::core::define (:my::countdown (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::if (:wat::core::<= n 0)
            -> :wat::core::i64
            n
            (:wat::core::do
              (:wat::core::i64::+,2 n 0)
              (:my::countdown (:wat::core::i64::-,2 n 1)))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:my::countdown 100000))
    "#;
    assert_eq!(unwrap_i64(run(src)), 0);
}

// ─── 9. Nested do forms compose ─────────────────────────────────────────

#[test]
fn do_nested_compose_cleanly() {
    // Inner do evaluates its non-final and returns 1; outer do evaluates
    // the inner-do (result 1, discarded) and returns 2. Both stdout lines
    // appear in order.
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::do
            (:wat::core::do
              (:wat::io::IOWriter/println stdout "inner")
              (:wat::io::IOWriter/println stdout "inner-final"))
            (:wat::io::IOWriter/println stdout "outer-final")))
    "#;
    let out = run_with_stdout(src);
    assert_eq!(out, vec!["inner", "inner-final", "outer-final"]);
}

// ─── 10. Mixed with let*: types compose ─────────────────────────────────

#[test]
fn do_inside_let_star_body_composes_types_cleanly() {
    // A let* whose body is a do form — types compose: let*'s body slot
    // expects whatever the recipient (here :user::main's -> :i64) wants;
    // body is a do form whose final form returns the bound x = 7. The
    // first non-final of the do uses the binding too (proves do sees the
    // surrounding let*'s scope).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((x :wat::core::i64) 7))
            (:wat::core::do
              (:wat::core::i64::+,2 x 1)
              x)))
    "#;
    assert_eq!(unwrap_i64(run(src)), 7);
}
