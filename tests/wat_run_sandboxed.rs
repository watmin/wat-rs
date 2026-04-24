//! End-to-end tests for `:wat::kernel::run-sandboxed` — arc 007 slice 2a.
//!
//! The sandbox takes wat source + stdin lines + scope, freezes a fresh
//! inner world, invokes `:user::main` with StringIo-backed stdio, and
//! captures what the program wrote. Happy-path coverage only in slice
//! 2a; panic isolation / shutdown-wait / scope-enforcement tests land
//! in slice 2b.
//!
//! Covers:
//! - No-op main → empty stdout + stderr, failure: None.
//! - Main writes one line → stdout captured.
//! - Main writes multiple lines to both stdout and stderr.
//! - Main reads stdin and echoes to stdout.
//! - Main uses `print` (no newline) vs `println` (with newline).
//! - Scope `:None` isolates from disk.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

/// Unwrap a RunResult struct value into its three fields.
fn unwrap_run_result(v: Value) -> (Vec<String>, Vec<String>, bool) {
    match v {
        Value::Struct(sv) => {
            assert_eq!(sv.type_name, ":wat::kernel::RunResult");
            assert_eq!(sv.fields.len(), 3);
            let stdout = as_vec_string(&sv.fields[0]);
            let stderr = as_vec_string(&sv.fields[1]);
            let failure_is_some = match &sv.fields[2] {
                Value::Option(opt) => opt.is_some(),
                other => panic!("expected Option for failure; got {:?}", other),
            };
            (stdout, stderr, failure_is_some)
        }
        other => panic!("expected Struct; got {:?}", other),
    }
}

fn as_vec_string(v: &Value) -> Vec<String> {
    match v {
        Value::Vec(items) => items
            .iter()
            .map(|item| match item {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String; got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec; got {:?}", other),
    }
}

// ─── Happy path — no-op main ─────────────────────────────────────────────

#[test]
fn noop_main_yields_empty_stdout_and_stderr() {
    let src = r#"

        ;; Outer program: runs a sandboxed no-op main.
        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               ())"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert!(stdout.is_empty(), "expected empty stdout; got {:?}", stdout);
    assert!(stderr.is_empty(), "expected empty stderr; got {:?}", stderr);
    assert!(!failure, "expected failure: None; got Some");
}

// ─── Single stdout write ─────────────────────────────────────────────────

#[test]
fn main_writes_single_line_to_stdout() {
    let src = r#"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::io::IOWriter/println stdout \"hello\"))"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["hello".to_string()]);
    assert!(stderr.is_empty());
    assert!(!failure);
}

// ─── Multi-line + stderr ─────────────────────────────────────────────────

#[test]
fn main_writes_to_both_stdout_and_stderr() {
    let src = r#"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::core::let*
                 (((_ :()) (:wat::io::IOWriter/println stdout \"one\"))
                  ((_ :()) (:wat::io::IOWriter/println stdout \"two\"))
                  ((_ :()) (:wat::io::IOWriter/println stderr \"oops\")))
                 ()))"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["one".to_string(), "two".to_string()]);
    assert_eq!(stderr, vec!["oops".to_string()]);
    assert!(!failure);
}

// ─── Main echoes stdin to stdout ─────────────────────────────────────────

#[test]
fn main_echoes_stdin_to_stdout() {
    // r##"..."## delimiter so the outer vec :String "watmin" doesn't
    // need backslash-escaped quotes at the wat surface.
    let src = r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
                 ((Some line) (:wat::io::IOWriter/println stdout line))
                 (:None ())))"
            (:wat::core::vec :String "watmin")
            :None))
    "##;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["watmin".to_string()]);
    assert!(stderr.is_empty());
    assert!(!failure);
}

// ─── print (no newline) vs println ───────────────────────────────────────

#[test]
fn print_without_newline_does_not_split_into_lines() {
    // Three prints to stdout: "a" + "b" + "c". No newline.
    // Buffer: "abc". Split on \n: ["abc"]. No trailing \n to trim.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::core::let*
                 (((_ :()) (:wat::io::IOWriter/print stdout \"a\"))
                  ((_ :()) (:wat::io::IOWriter/print stdout \"b\"))
                  ((_ :()) (:wat::io::IOWriter/print stdout \"c\")))
                 ()))"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, _, _) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["abc".to_string()]);
}

// ─── Failure capture (slice 2b) ─────────────────────────────────────────

fn unwrap_run_result_with_failure(v: Value) -> (Vec<String>, Vec<String>, Option<String>) {
    match v {
        Value::Struct(sv) => {
            assert_eq!(sv.type_name, ":wat::kernel::RunResult");
            assert_eq!(sv.fields.len(), 3);
            let stdout = as_vec_string(&sv.fields[0]);
            let stderr = as_vec_string(&sv.fields[1]);
            let failure_msg = match &sv.fields[2] {
                Value::Option(opt) => match &**opt {
                    Some(Value::Struct(fs)) => {
                        assert_eq!(fs.type_name, ":wat::kernel::Failure");
                        // fields[0] is message :String
                        match &fs.fields[0] {
                            Value::String(s) => Some((**s).clone()),
                            _ => panic!("Failure.message not a String"),
                        }
                    }
                    Some(other) => panic!("Failure field not Struct: {:?}", other),
                    None => None,
                },
                _ => panic!("failure field not Option"),
            };
            (stdout, stderr, failure_msg)
        }
        other => panic!("expected Struct; got {:?}", other),
    }
}

#[test]
fn parse_error_in_source_surfaces_as_failure() {
    // Inner source is unterminated — the lexer's UnterminatedString
    // surfaces as a startup error, captured into Failure.
    let src = r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::core::define (:user::main (stdin :wat::io::IOReader) (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :()) \"unclosed"
            (:wat::core::vec :String)
            :None))
    "##;
    let (stdout, stderr, failure) = unwrap_run_result_with_failure(run(src));
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
    let msg = failure.expect("expected startup failure");
    assert!(
        msg.contains("startup") || msg.contains("Unterminated") || msg.contains("parse"),
        "unexpected failure message: {}",
        msg
    );
}

#[test]
fn main_signature_mismatch_surfaces_as_failure() {
    // Inner main takes no IO params — mismatch against the expected
    // three-IO contract. Captured as Failure.
    let src = r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main -> :()) ())"
            (:wat::core::vec :String)
            :None))
    "##;
    let (_, _, failure) = unwrap_run_result_with_failure(run(src));
    let msg = failure.expect("expected signature-mismatch failure");
    assert!(
        msg.contains(":user::main"),
        "failure should mention :user::main; got {}",
        msg
    );
}

#[test]
fn missing_user_main_surfaces_as_failure() {
    let src = r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)"
            (:wat::core::vec :String)
            :None))
    "##;
    let (_, _, failure) = unwrap_run_result_with_failure(run(src));
    let msg = failure.expect("expected missing-main failure");
    assert!(
        msg.contains(":user::main"),
        "failure should mention missing :user::main; got {}",
        msg
    );
}

#[test]
fn sandboxed_panic_caught_into_failure_and_partial_output_preserved() {
    // Inner main writes "before panic" to stdout, then triggers a
    // real Rust-level panic via :wat::holon::Bundle under :abort
    // mode with a list exceeding the capacity budget. Outer caller
    // sees RunResult with stdout=["before panic"] + Failure with
    // "panic" in the message.
    //
    // Arc 037 slice 1: ambient router picks dim from DEFAULT_TIERS
    // ([256, 4096, 10000, 100000]). Largest tier's sqrt ≈ 316. A
    // 400-element Bundle overflows every tier; :abort mode panics.
    let atoms = (0..400)
        .map(|i| format!(r#"(:wat::holon::Atom \"atom-{}\")"#, i))
        .collect::<Vec<_>>()
        .join(" ");
    let src = format!(r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :abort)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::core::let*
                 (((_ :()) (:wat::io::IOWriter/println stdout \"before panic\"))
                  ((_ :wat::holon::BundleResult)
                   (:wat::holon::Bundle
                     (:wat::core::list :wat::holon::HolonAST
                       {atoms}))))
                 ()))"
            (:wat::core::vec :String)
            :None))
    "##);
    let src = src.as_str();
    let (stdout, _, failure) = unwrap_run_result_with_failure(run(src));
    // Stdout captured BEFORE the panic should survive.
    assert_eq!(
        stdout,
        vec!["before panic".to_string()],
        "partial output before panic should be preserved"
    );
    let msg = failure.expect("expected panic failure");
    assert!(
        msg.contains("panic") || msg.contains("capacity") || msg.contains("Bundle"),
        "failure message should mention panic / capacity / Bundle; got {}",
        msg
    );
}

// ─── Scope enforcement (slice 2b) ───────────────────────────────────────

/// RAII test dir under std::env::temp_dir. Cleanup on drop.
struct ScopeDir {
    path: std::path::PathBuf,
}

impl ScopeDir {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "wat-sandbox-scope-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn write(&self, name: &str, contents: &str) -> std::path::PathBuf {
        let file_path = self.path.join(name);
        std::fs::write(&file_path, contents).unwrap();
        file_path
    }
}

impl Drop for ScopeDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn scoped_file_eval_inside_scope_succeeds() {
    // Write a wat source to a temp dir; point run-sandboxed's scope
    // at that dir; use inside the sandbox to
    // read it. The ScopedLoader allows the read because the target
    // is inside the canonical root.
    let scope = ScopeDir::new();
    let inner_source_path = scope.write("fortytwo.wat", "(:wat::core::i64::+ 40 2)");
    let inner_src = format!(
        r#"(:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :())
           (:wat::core::match
             (:wat::eval-file! "{path}")
             -> :()
             ((Ok h) (:wat::io::IOWriter/println stdout "ok"))
             ((Err _) (:wat::io::IOWriter/println stderr "err"))))"#,
        path = inner_source_path.display()
    );

    let scope_path = scope.path.canonicalize().unwrap();
    let src = format!(
        r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            {inner_src:?}
            (:wat::core::vec :String)
            (Some {scope:?})))
        "##,
        inner_src = inner_src,
        scope = scope_path.display().to_string(),
    );
    let (stdout, stderr, failure) = unwrap_run_result_with_failure(run(&src));
    assert_eq!(
        stdout,
        vec!["ok".to_string()],
        "in-scope file read should succeed; stderr was {:?}; failure={:?}",
        stderr,
        failure
    );
    assert!(
        failure.is_none(),
        "expected no failure; got {:?}",
        failure
    );
}

#[test]
fn scoped_file_eval_outside_scope_surfaces_as_err() {
    // Create a file OUTSIDE the scope; attempt to read it via
    // :wat::eval-file!. ScopedLoader's containment check
    // rejects; wat-rs surfaces this as an Err in the eval-file!
    // Result; the sandboxed program matches on Err and writes to
    // stderr. The sandbox itself succeeds — the "failure" here is
    // at the wat level (the Err arm), not a sandbox-caught failure.
    // This is the honest shape: the sandbox ran the program; the
    // program observed its own capability boundary.
    let scope = ScopeDir::new();
    let outside = ScopeDir::new();
    let outside_file = outside.write("leak.txt", "secrets");

    let inner_src = format!(
        r#"(:wat::config::set-capacity-mode! :error)
         (:wat::core::define (:user::main
                              (stdin  :wat::io::IOReader)
                              (stdout :wat::io::IOWriter)
                              (stderr :wat::io::IOWriter)
                              -> :())
           (:wat::core::match
             (:wat::eval-file! "{path}")
             -> :()
             ((Ok _) (:wat::io::IOWriter/println stdout "leaked"))
             ((Err _) (:wat::io::IOWriter/println stderr "blocked"))))"#,
        path = outside_file.display()
    );

    let scope_path = scope.path.canonicalize().unwrap();
    let src = format!(
        r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            {inner_src:?}
            (:wat::core::vec :String)
            (Some {scope:?})))
        "##,
        inner_src = inner_src,
        scope = scope_path.display().to_string(),
    );
    let (stdout, stderr, _failure) = unwrap_run_result_with_failure(run(&src));
    // The sandbox blocked the read — matched Err arm → stderr "blocked".
    assert_eq!(
        stderr,
        vec!["blocked".to_string()],
        "out-of-scope read should route to Err; stdout was {:?}",
        stdout
    );
    // stdout should NOT contain "leaked".
    assert!(
        !stdout.contains(&"leaked".to_string()),
        "out-of-scope read should not reach the Ok arm; stdout: {:?}",
        stdout
    );
}

// ─── Multiple stdin lines ────────────────────────────────────────────────

#[test]
fn main_reads_multiple_stdin_lines() {
    // Read and println each line until EOF.
    let src = r##"

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:my::echo-all
                                  (r :wat::io::IOReader)
                                  (w :wat::io::IOWriter)
                                  -> :())
               (:wat::core::match (:wat::io::IOReader/read-line r) -> :()
                 ((Some line)
                   (:wat::core::let*
                     (((_ :()) (:wat::io::IOWriter/println w line)))
                     (:my::echo-all r w)))
                 (:None ())))
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:my::echo-all stdin stdout))"
            (:wat::core::vec :String "alpha" "beta" "gamma")
            :None))
    "##;
    let (stdout, _, _) = unwrap_run_result(run(src));
    assert_eq!(
        stdout,
        vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
    );
}
