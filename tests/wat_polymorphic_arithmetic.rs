//! Arc 050 — polymorphic numerics with int → float promotion.
//!
//! Coverage matrix:
//! - Polymorphic arithmetic `+ - * /`: i64×i64, f64×f64, i64×f64, f64×i64
//! - Polymorphic comparison `= < > <= >=`: cross-numeric pairs typecheck
//!   and execute correctly
//! - Typed strict `:wat::core::i64::*` and `:wat::core::f64::*` variants reject cross-type
//! - Division-by-zero error preserved across all forms
//! - Non-numeric arithmetic args rejected at check-time

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::thread_io::{install_ambient_stdio, uninstall_ambient_stdio, AmbientStdio};

fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(read_fd));
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));
    (reader, writer)
}

fn drain_lines(reader: &Arc<dyn WatReader>) -> Vec<String> {
    let bytes = reader
        .read_all(wat::span::Span::unknown())
        .expect("read-all");
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

fn run(src: &str) -> Vec<String> {
    let _ = uninstall_ambient_stdio();
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    invoke_user_main(&world, Vec::new()).expect("main");
    let _ = uninstall_ambient_stdio();
    drain_lines(&stdout_capture)
}

fn run_expecting_check_error(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("startup should fail with check error");
    format!("{:?}", err)
}

fn run_expecting_runtime_error(src: &str) -> String {
    let _ = uninstall_ambient_stdio();
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should pass type-check");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (stderr_capture, stderr_service) = pipe_pair();
    let _ = stdout_capture;
    let _ = stderr_capture;
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    let err = invoke_user_main(&world, Vec::new()).expect_err("main should error");
    let _ = uninstall_ambient_stdio();
    format!("{:?}", err)
}

// ─── Polymorphic arithmetic — homogeneous types ──────────────────────

#[test]
fn poly_add_i64_i64_returns_i64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [sum (:wat::core::+ 2 3)]
            (:wat::kernel::println sum)))
    "##;
    assert_eq!(run(src), vec!["5".to_string()]);
}

#[test]
fn poly_add_f64_f64_returns_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [sum (:wat::core::+ 2.0 3.5)]
            (:wat::kernel::println sum)))
    "##;
    assert_eq!(run(src), vec!["5.5".to_string()]);
}

// ─── Polymorphic arithmetic — cross-numeric promotion ────────────────

#[test]
fn poly_add_i64_f64_promotes_to_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [sum (:wat::core::+ 1 2.5)]
            (:wat::kernel::println sum)))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

#[test]
fn poly_add_f64_i64_promotes_to_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [sum (:wat::core::+ 2.5 1)]
            (:wat::kernel::println sum)))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

#[test]
fn poly_sub_mixed_promotes() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [d (:wat::core::- 5 1.5)]
            (:wat::kernel::println d)))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

#[test]
fn poly_mul_mixed_promotes() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [p (:wat::core::* 3 1.5)]
            (:wat::kernel::println p)))
    "##;
    assert_eq!(run(src), vec!["4.5".to_string()]);
}

#[test]
fn poly_div_i64_i64_returns_i64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [q (:wat::core::/ 7 2)]
            (:wat::kernel::println q)))
    "##;
    assert_eq!(run(src), vec!["3".to_string()]);
}

#[test]
fn poly_div_mixed_returns_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [q (:wat::core::/ 7 2.0)]
            (:wat::kernel::println q)))
    "##;
    assert_eq!(run(src), vec!["3.5".to_string()]);
}

// ─── Division by zero — typed and polymorphic forms ──────────────────

#[test]
fn poly_div_i64_zero_errors() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [q (:wat::core::/ 5 0)]
            (:wat::kernel::println q)))
    "##;
    let err = run_expecting_runtime_error(src);
    assert!(err.to_lowercase().contains("division") || err.to_lowercase().contains("zero"),
        "expected DivisionByZero; got {}", err);
}

#[test]
fn poly_div_f64_zero_errors() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [q (:wat::core::/ 5.0 0.0)]
            (:wat::kernel::println q)))
    "##;
    let err = run_expecting_runtime_error(src);
    assert!(err.to_lowercase().contains("division") || err.to_lowercase().contains("zero"),
        "expected DivisionByZero; got {}", err);
}

#[test]
fn poly_div_mixed_zero_errors() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [q (:wat::core::/ 5 0.0)]
            (:wat::kernel::println q)))
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
          (:user::main -> :wat::core::nil)
          (:wat::core::if (:wat::core::< 1 2.5) -> :wat::core::nil
            (:wat::kernel::println "less")
            (:wat::kernel::println "not less")))
    "##;
    assert_eq!(run(src), vec!["\"less\"".to_string()]);
}

#[test]
fn poly_eq_mixed_promotes() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::if (:wat::core::= 3 3.0) -> :wat::core::nil
            (:wat::kernel::println "equal")
            (:wat::kernel::println "not equal")))
    "##;
    assert_eq!(run(src), vec!["\"equal\"".to_string()]);
}

#[test]
fn poly_eq_strings_still_works() {
    // Non-numeric same-type still works as before.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::if (:wat::core::= "a" "a") -> :wat::core::nil
            (:wat::kernel::println "yes")
            (:wat::kernel::println "no")))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

// ─── Typed strict comparison/equality variants ───────────────────────

// Arc 148 slice 5 — per-Type comparison leaves
// (`:wat::core::{i64,f64}::{=,<,>,<=,>=}`) retired. Strict
// type-locking is now expressed via param types at the call
// site's enclosing function: a helper with `(a :wat::core::i64) (b :wat::core::i64)`
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
          (:user::main -> :wat::core::nil)
          (:wat::core::if (:my::test::eq-i64 3 3) -> :wat::core::nil
            (:wat::kernel::println "yes")
            (:wat::kernel::println "no")))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

#[test]
fn typed_strict_i64_eq_rejects_f64_arg() {
    let src = r##"
        (:wat::core::define
          (:my::test::eq-i64 (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::bool)
          (:wat::core::= a b))
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::if (:my::test::eq-i64 3 3.0) -> :wat::core::nil
            (:wat::kernel::println "yes")
            (:wat::kernel::println "no")))
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
          (:user::main -> :wat::core::nil)
          (:wat::core::if (:my::test::lt-f64 1.5 2.5) -> :wat::core::nil
            (:wat::kernel::println "less")
            (:wat::kernel::println "not less")))
    "##;
    assert_eq!(run(src), vec!["\"less\"".to_string()]);
}

#[test]
fn typed_strict_f64_lt_rejects_i64_arg() {
    let src = r##"
        (:wat::core::define
          (:my::test::lt-f64 (a :wat::core::f64) (b :wat::core::f64) -> :wat::core::bool)
          (:wat::core::< a b))
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::if (:my::test::lt-f64 1 2.5) -> :wat::core::nil
            (:wat::kernel::println "less")
            (:wat::kernel::println "not less")))
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
          (:user::main -> :wat::core::nil)
          (:wat::core::let [bad (:wat::core::+ "hello" 1)]
            (:wat::kernel::println bad)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(err.contains("String") || err.contains("i64") || err.contains("f64"),
        "expected type-mismatch on string arg; got {}", err);
}

// ─── Typed strict arithmetic still works alongside polymorphic ───────

#[test]
fn typed_strict_arithmetic_coexists() {
    // Existing :wat::core::i64::+'2 / :wat::core::f64::+'2 still work
    // and reject cross-type. Polymorphic + works alongside.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a (:wat::core::i64::+'2 1 2)
             b (:wat::core::f64::+'2 1.0 2.0)
             c (:wat::core::+ 1 2.5)]
            (:wat::kernel::println a)))
    "##;
    assert_eq!(run(src), vec!["3".to_string()]);
}

// ─── Arc 148 slice 4 — variadic polymorphic arithmetic ───────────────
//
// Per the locked DESIGN: the polymorphic surface at `:wat::core::<v>`
// is variadic; reduces left-to-right via per-pair routing. Lisp/
// Clojure arity rules: `+`/`*` 0-ary returns identity; `-`/`/` 0-ary
// errors. 1-ary `+`/`*` returns arg unchanged; `-`/`/` insert
// identity-on-left (negation/reciprocal). 2+-ary folds.

#[test]
fn slice4_variadic_add_three_i64_args_folds() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [sum (:wat::core::+ 1 2 3 4 5)]
            (:wat::kernel::println sum)))
    "##;
    assert_eq!(run(src), vec!["15".to_string()]);
}

#[test]
fn slice4_variadic_add_mixed_numerics_design_worked_example() {
    // The DESIGN's worked example: (:wat::core::+ 0 40.0 2) => :wat::core::f64 42.0
    // Mixed-numeric variadic via dispatch + per-pair routing.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [sum (:wat::core::+ 0 40.0 2)]
            (:wat::kernel::println sum)))
    "##;
    assert_eq!(run(src), vec!["42.0".to_string()]);
}

#[test]
fn slice4_variadic_add_zero_ary_returns_i64_zero() {
    // `+` 0-ary returns identity 0:wat::core::i64 per Lisp/Clojure tradition.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [zero (:wat::core::+)]
            (:wat::kernel::println zero)))
    "##;
    assert_eq!(run(src), vec!["0".to_string()]);
}

#[test]
fn slice4_variadic_mul_zero_ary_returns_i64_one() {
    // `*` 0-ary returns identity 1:wat::core::i64.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [one (:wat::core::*)]
            (:wat::kernel::println one)))
    "##;
    assert_eq!(run(src), vec!["1".to_string()]);
}

#[test]
fn slice4_variadic_sub_one_ary_negates_i64() {
    // `(- x)` inserts identity-on-left = -x.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [neg (:wat::core::- 5)]
            (:wat::kernel::println neg)))
    "##;
    assert_eq!(run(src), vec!["-5".to_string()]);
}

#[test]
fn slice4_variadic_sub_one_ary_negates_f64() {
    // 1-ary `-` preserves type (DESIGN § "Type preservation").
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [neg (:wat::core::- 5.5)]
            (:wat::kernel::println neg)))
    "##;
    assert_eq!(run(src), vec!["-5.5".to_string()]);
}

#[test]
fn slice4_variadic_div_one_ary_reciprocal_i64_truncates() {
    // `(/ 5)` = `(/ 1 5)` = 0 (i64 truncation; honest).
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [r (:wat::core::/ 5)]
            (:wat::kernel::println r)))
    "##;
    assert_eq!(run(src), vec!["0".to_string()]);
}

#[test]
fn slice4_variadic_sub_zero_ary_errors() {
    // `(:-)` is ARITY ERROR — `-` has no identity.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [bad (:wat::core::-)]
            (:wat::kernel::println bad)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(err.to_lowercase().contains("arity") || err.contains("ArityMismatch"),
        "expected ArityMismatch on 0-ary `-`; got {}", err);
}

#[test]
fn slice4_variadic_div_zero_ary_errors() {
    // `(:/)` is ARITY ERROR — `/` has no identity.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [bad (:wat::core::/)]
            (:wat::kernel::println bad)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(err.to_lowercase().contains("arity") || err.contains("ArityMismatch"),
        "expected ArityMismatch on 0-ary `/`; got {}", err);
}

#[test]
fn slice4_same_type_variadic_i64_add_works() {
    // The wat-defined :wat::core::i64::+ variadic wrapper.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [sum (:wat::core::i64::+ 1 2 3 4 5)]
            (:wat::kernel::println sum)))
    "##;
    assert_eq!(run(src), vec!["15".to_string()]);
}

#[test]
fn slice4_same_type_variadic_f64_mul_works() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [p (:wat::core::f64::* 1.0 2.0 3.0)]
            (:wat::kernel::println p)))
    "##;
    assert_eq!(run(src), vec!["6.0".to_string()]);
}

#[test]
fn slice4_mixed_type_leaf_directly_callable() {
    // The mixed-type leaf is reachable per no-privacy.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [s (:wat::core::+'i64'f64 1 2.0)]
            (:wat::kernel::println s)))
    "##;
    assert_eq!(run(src), vec!["3.0".to_string()]);
}

#[test]
fn slice4_binary_dispatch_directly_callable() {
    // The binary Dispatch entity at :wat::core::+'2 routes by type.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [s (:wat::core::+'2 1 2.0)]
            (:wat::kernel::println s)))
    "##;
    assert_eq!(run(src), vec!["3.0".to_string()]);
}
