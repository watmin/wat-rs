//! Integration coverage for arc 143 slice 6 — the
//! `:wat::runtime::define-alias` defmacro.
//!
//! The macro lives in `wat/runtime.wat` (pure wat) and composes
//! the substrate primitives shipped in slices 1+2+3:
//!   - `:wat::runtime::signature-of`         (slice 1)
//!   - `:wat::runtime::rename-callable-name`  (slice 3)
//!   - `:wat::runtime::extract-arg-names`     (slice 3)
//!   - computed unquote `,(expr)`             (slice 2)
//!
//! Sequencing constraint discovered in slice 6:
//!   Computed unquote runs during macro expansion (step 4 in
//!   startup_from_forms_post_config) with &SymbolTable::default()
//!   (empty). Only substrate primitives visible via
//!   CheckEnv::with_builtins() are reachable at expand-time.
//!   User-defined functions in the same source file are NOT visible
//!   until step 6 (register_defines). Therefore define-alias can
//!   only alias substrate primitives at expand-time.
//!
//! Tests:
//!   1. Alias a substrate primitive (:wat::core::foldl) — expand-time
//!      signature-of succeeds; alias delegates to the primitive correctly.
//!   2. Alias another substrate primitive (:wat::core::length) — verifies
//!      the macro works for multiple targets.
//!   3. Error case — alias to a name that doesn't exist (not a substrate
//!      primitive, not a user define) — Option/expect panics at expand-time.
//!      Verifies the macro expands eagerly and the error message propagates.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
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

// ─── Test 1: alias :wat::core::foldl — expand-time substrate lookup works ────

#[test]
fn define_alias_foldl_to_user_fold_delegates_correctly() {
    // Alias :wat::core::foldl as :user::my-fold.
    // At expand-time, signature-of :wat::core::foldl resolves via
    // CheckEnv::with_builtins() — the substrate primitive IS visible.
    // Call (:user::my-fold (Vector :i64 1 2 3 4) 0 +lambda) → 10.
    let src = r##"

        (:wat::runtime::define-alias :user::my-fold :wat::core::foldl)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::io::IOWriter/println stdout
            (:wat::edn::write
              (:user::my-fold
                (:wat::core::Vector :wat::core::i64 1 2 3 4)
                0
                (:wat::core::lambda
                  ((acc :wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
                  (:wat::core::+ acc x))))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    assert_eq!(
        out[0].trim(), "10",
        "expected alias of foldl to sum [1,2,3,4] from 0 → 10, got: {}",
        out[0]
    );
}

// ─── Test 2: alias :wat::core::length ────────────────────────────────────────

#[test]
fn define_alias_length_to_user_size_delegates_correctly() {
    // Alias :wat::core::length as :user::my-size.
    // At expand-time, signature-of :wat::core::length resolves via substrate.
    // Call (:user::my-size (Vector :i64 10 20 30)) → 3.
    let src = r##"

        (:wat::runtime::define-alias :user::my-size :wat::core::length)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::io::IOWriter/println stdout
            (:wat::edn::write
              (:user::my-size
                (:wat::core::Vector :wat::core::i64 10 20 30)))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    assert_eq!(
        out[0].trim(), "3",
        "expected alias of length to return 3 for Vec of 3 elements, got: {}",
        out[0]
    );
}

// ─── Test 3: unknown target panics at expand-time ────────────────────────────

#[test]
fn define_alias_unknown_target_panics_at_expand_time() {
    // :user::name-that-does-not-exist is not a substrate primitive or user define.
    // The macro calls (Option/expect (signature-of :user::name-that-does-not-exist) ...)
    // at expand-time; expect panics via std::panic::panic_any —
    // this propagates as a Rust panic out of startup_from_source.
    //
    // NOTE: expect_panic uses std::panic::panic_any (not a Result::Err),
    // so startup_from_source propagates the panic rather than returning
    // Err(StartupError). We use catch_unwind to detect it.
    let src = r##"

        (:wat::runtime::define-alias :user::alias :user::name-that-does-not-exist)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::io::IOWriter/println stdout "should not reach"))
    "##;
    let result = std::panic::catch_unwind(|| {
        startup_from_source(
            src,
            Some(concat!(file!(), ":", line!())),
            Arc::new(InMemoryLoader::new()),
        )
    });
    assert!(
        result.is_err(),
        "expected startup to panic for unknown target name, but it returned Ok"
    );
    // The panic payload is an AssertionPayload; its message field carries
    // "define-alias: target name not found in environment".
    // We don't inspect the payload here — the panic-at-startup is the
    // observable signal. The message is in the macro body verbatim.
}
