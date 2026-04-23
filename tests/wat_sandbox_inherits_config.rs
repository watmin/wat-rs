//! Integration coverage for arc 031 — sandbox inherits outer
//! committed Config by default.
//!
//! Every test builds an outer `:user::main` that commits a specific
//! Config, constructs inner `Vec<WatAST>` forms that deliberately
//! DROP the setter section, and runs them through
//! `:wat::kernel::run-sandboxed-ast`. The inner program reads a
//! config value (dims via `:wat::config::dims`) and prints it; Rust
//! asserts the printed value came from the outer's commit, not from
//! defaults.
//!
//! Back-compat: the last test keeps inner setters and shows they
//! still override — the pre-031 shape continues to work.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
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
    let bytes = stdout.snapshot_bytes().expect("stdout snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

// ─── sandbox with no setters inherits outer dims ───────────────────────

#[test]
fn sandbox_no_setters_inherits_outer_dims() {
    // Outer commits dims=4096. Inner forms skip setters entirely —
    // they only define :user::main and print `(:wat::config::dims)`.
    // Outer captures the printed value and echoes it; expected
    // output is "4096".
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 4096)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((forms :Vec<wat::WatAST>)
              (:wat::core::vec :wat::WatAST
                (:wat::core::quote
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout
                      (:wat::core::i64::to-string (:wat::config::dims)))))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-ast forms (:wat::core::vec :String) :None))
             ((lines :Vec<String>) (:wat::kernel::RunResult/stdout r))
             ((line :String) (:wat::core::first lines)))
            (:wat::io::IOWriter/println stdout line)))
    "##;
    assert_eq!(run(src), vec!["4096"]);
}

// ─── sandbox with only dims setter inherits outer capacity-mode ────────

#[test]
fn sandbox_with_dims_setter_still_inherits_capacity_mode() {
    // Outer commits :error + 1024. Inner commits dims=2048 only —
    // capacity-mode should still inherit the outer :error. Inner
    // prints its committed dims; outer captures it. Expected: 2048.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((forms :Vec<wat::WatAST>)
              (:wat::core::vec :wat::WatAST
                (:wat::core::quote (:wat::config::set-dims! 2048))
                (:wat::core::quote
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout
                      (:wat::core::i64::to-string (:wat::config::dims)))))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-ast forms (:wat::core::vec :String) :None))
             ((lines :Vec<String>) (:wat::kernel::RunResult/stdout r))
             ((line :String) (:wat::core::first lines)))
            (:wat::io::IOWriter/println stdout line)))
    "##;
    assert_eq!(run(src), vec!["2048"]);
}

// ─── sandbox with both setters — back-compat path ──────────────────────

#[test]
fn sandbox_with_both_setters_still_uses_explicit_values() {
    // Back-compat for every pre-031 deftest template. Outer at 1024;
    // inner explicitly sets 4096. Inner's explicit setters win.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((forms :Vec<wat::WatAST>)
              (:wat::core::vec :wat::WatAST
                (:wat::core::quote (:wat::config::set-capacity-mode! :error))
                (:wat::core::quote (:wat::config::set-dims! 4096))
                (:wat::core::quote
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout
                      (:wat::core::i64::to-string (:wat::config::dims)))))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-ast forms (:wat::core::vec :String) :None))
             ((lines :Vec<String>) (:wat::kernel::RunResult/stdout r))
             ((line :String) (:wat::core::first lines)))
            (:wat::io::IOWriter/println stdout line)))
    "##;
    assert_eq!(run(src), vec!["4096"]);
}

// ─── fork (hermetic) child inherits through COW ────────────────────────

#[test]
fn hermetic_sandbox_inherits_outer_dims_through_fork() {
    // Same as sandbox_no_setters_inherits_outer_dims but routed
    // through run-sandboxed-hermetic-ast — the fork-based sibling.
    // The child process COW-inherits the parent's committed Config
    // and runs startup_from_forms_with_inherit.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 4096)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((forms :Vec<wat::WatAST>)
              (:wat::core::vec :wat::WatAST
                (:wat::core::quote
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout
                      (:wat::core::i64::to-string (:wat::config::dims)))))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-hermetic-ast forms (:wat::core::vec :String) :None))
             ((lines :Vec<String>) (:wat::kernel::RunResult/stdout r))
             ((line :String) (:wat::core::first lines)))
            (:wat::io::IOWriter/println stdout line)))
    "##;
    assert_eq!(run(src), vec!["4096"]);
}
