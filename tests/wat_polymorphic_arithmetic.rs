//! Arc 050 — polymorphic numerics with int → float promotion.
//!
//! Coverage matrix:
//! - Polymorphic arithmetic `+ - * /`: i64×i64, f64×f64, i64×f64, f64×i64
//! - Polymorphic comparison `= < > <= >=`: cross-numeric pairs typecheck
//!   and execute correctly
//! - Typed strict `:i64::*` and `:f64::*` variants reject cross-type
//! - Division-by-zero error preserved across all forms
//! - Non-numeric arithmetic args rejected at check-time

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
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

fn run_expecting_check_error(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("startup should fail with check error");
    format!("{:?}", err)
}

fn run_expecting_runtime_error(src: &str) -> String {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should pass type-check");
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
    let err = invoke_user_main(&world, args).expect_err("main should error");
    format!("{:?}", err)
}

// ─── Polymorphic arithmetic — homogeneous types ──────────────────────

#[test]
fn poly_add_i64_i64_returns_i64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((sum :wat::core::i64) (:wat::core::+ 2 3)))
            (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string sum))))
    "##;
    assert_eq!(run(src), vec!["5".to_string()]);
}

#[test]
fn poly_add_f64_f64_returns_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((sum :wat::core::f64) (:wat::core::+ 2.0 3.5)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string sum))))
    "##;
    assert_eq!(run(src), vec!["5.5".to_string()]);
}

// ─── Polymorphic arithmetic — cross-numeric promotion ────────────────

#[test]
fn poly_add_i64_f64_promotes_to_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((sum :wat::core::f64) (:wat::core::+ 1 2.5)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string sum))))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

#[test]
fn poly_add_f64_i64_promotes_to_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((sum :wat::core::f64) (:wat::core::+ 2.5 1)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string sum))))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

#[test]
fn poly_sub_mixed_promotes() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((d :wat::core::f64) (:wat::core::- 5 1.5)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string d))))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

#[test]
fn poly_mul_mixed_promotes() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((p :wat::core::f64) (:wat::core::* 3 1.5)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string p))))
    "##;
    assert_eq!(run(src), vec!["4.5".to_string()]);
}

#[test]
fn poly_div_i64_i64_returns_i64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((q :wat::core::i64) (:wat::core::/ 7 2)))
            (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string q))))
    "##;
    assert_eq!(run(src), vec!["3".to_string()]);
}

#[test]
fn poly_div_mixed_returns_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((q :wat::core::f64) (:wat::core::/ 7 2.0)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string q))))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

// ─── Division by zero — typed and polymorphic forms ──────────────────

#[test]
fn poly_div_i64_zero_errors() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((q :wat::core::i64) (:wat::core::/ 5 0)))
            (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string q))))
    "##;
    let err = run_expecting_runtime_error(src);
    assert!(err.to_lowercase().contains("division") || err.to_lowercase().contains("zero"),
        "expected DivisionByZero; got {}", err);
}

#[test]
fn poly_div_f64_zero_errors() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((q :wat::core::f64) (:wat::core::/ 5.0 0.0)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string q))))
    "##;
    let err = run_expecting_runtime_error(src);
    assert!(err.to_lowercase().contains("division") || err.to_lowercase().contains("zero"),
        "expected DivisionByZero; got {}", err);
}

#[test]
fn poly_div_mixed_zero_errors() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((q :wat::core::f64) (:wat::core::/ 5 0.0)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string q))))
    "##;
    let err = run_expecting_runtime_error(src);
    assert!(err.to_lowercase().contains("division") || err.to_lowercase().contains("zero"),
        "expected DivisionByZero; got {}", err);
}

// ─── Polymorphic comparison — cross-numeric ──────────────────────────

#[test]
fn poly_lt_mixed_i64_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:wat::core::< 1 2.5) -> :wat::core::unit
            (:wat::io::IOWriter/println stdout "less")
            (:wat::io::IOWriter/println stdout "not less")))
    "##;
    assert_eq!(run(src), vec!["less".to_string()]);
}

#[test]
fn poly_eq_mixed_promotes() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:wat::core::= 3 3.0) -> :wat::core::unit
            (:wat::io::IOWriter/println stdout "equal")
            (:wat::io::IOWriter/println stdout "not equal")))
    "##;
    assert_eq!(run(src), vec!["equal".to_string()]);
}

#[test]
fn poly_eq_strings_still_works() {
    // Non-numeric same-type still works as before.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:wat::core::= "a" "a") -> :wat::core::unit
            (:wat::io::IOWriter/println stdout "yes")
            (:wat::io::IOWriter/println stdout "no")))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

// ─── Typed strict comparison/equality variants ───────────────────────

// Arc 148 slice 5 — per-Type comparison leaves
// (`:wat::core::{i64,f64}::{=,<,>,<=,>=}`) retired. Strict
// type-locking is now expressed via param types at the call
// site's enclosing function: a helper with `(a :i64) (b :i64)`
// params calling the polymorphic `:wat::core::=` enforces the
// same constraint at the binding site that the per-Type leaf
// used to enforce in-line.

#[test]
fn typed_strict_i64_eq_homogeneous_works() {
    let src = r##"
        (:wat::core::define
          (:my::test::eq-i64 (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::bool)
          (:wat::core::= a b))
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:my::test::eq-i64 3 3) -> :wat::core::unit
            (:wat::io::IOWriter/println stdout "yes")
            (:wat::io::IOWriter/println stdout "no")))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

#[test]
fn typed_strict_i64_eq_rejects_f64_arg() {
    let src = r##"
        (:wat::core::define
          (:my::test::eq-i64 (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::bool)
          (:wat::core::= a b))
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:my::test::eq-i64 3 3.0) -> :wat::core::unit
            (:wat::io::IOWriter/println stdout "yes")
            (:wat::io::IOWriter/println stdout "no")))
    "##;
    let err = run_expecting_check_error(src);
    assert!(err.contains("i64") || err.contains("f64"),
        "expected type-mismatch; got {}", err);
}

#[test]
fn typed_strict_f64_lt_homogeneous_works() {
    let src = r##"
        (:wat::core::define
          (:my::test::lt-f64 (a :wat::core::f64) (b :wat::core::f64) -> :wat::core::bool)
          (:wat::core::< a b))
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:my::test::lt-f64 1.5 2.5) -> :wat::core::unit
            (:wat::io::IOWriter/println stdout "less")
            (:wat::io::IOWriter/println stdout "not less")))
    "##;
    assert_eq!(run(src), vec!["less".to_string()]);
}

#[test]
fn typed_strict_f64_lt_rejects_i64_arg() {
    let src = r##"
        (:wat::core::define
          (:my::test::lt-f64 (a :wat::core::f64) (b :wat::core::f64) -> :wat::core::bool)
          (:wat::core::< a b))
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:my::test::lt-f64 1 2.5) -> :wat::core::unit
            (:wat::io::IOWriter/println stdout "less")
            (:wat::io::IOWriter/println stdout "not less")))
    "##;
    let err = run_expecting_check_error(src);
    assert!(err.contains("f64") || err.contains("i64"),
        "expected type-mismatch; got {}", err);
}

// ─── Polymorphic arithmetic rejects non-numeric ──────────────────────

#[test]
fn poly_add_string_rejected_at_check() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let* (((bad :i64) (:wat::core::+ "hello" 1)))
            (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string bad))))
    "##;
    let err = run_expecting_check_error(src);
    assert!(err.contains("String") || err.contains("i64") || err.contains("f64"),
        "expected type-mismatch on string arg; got {}", err);
}

// ─── Typed strict arithmetic still works alongside polymorphic ───────

#[test]
fn typed_strict_arithmetic_coexists() {
    // Existing :wat::core::i64::+,2 / :wat::core::f64::+,2 still work
    // and reject cross-type. Polymorphic + works alongside.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((a :wat::core::i64) (:wat::core::i64::+,2 1 2))
             ((b :wat::core::f64) (:wat::core::f64::+,2 1.0 2.0))
             ((c :wat::core::f64) (:wat::core::+ 1 2.5)))
            (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string a))))
    "##;
    assert_eq!(run(src), vec!["3".to_string()]);
}
