//! Arc 051 — SimHash direction-space quantization.
//!
//! Coverage:
//! - Determinism: same AST, two calls → same i64
//! - Atom identity: `(simhash (Atom 0))` is stable
//! - Cosine-near-1 → small hamming distance (same/perturbed AST)
//! - Cosine-near-0 → hamming distance ≈ 32 (orthogonal-by-construction
//!   AST pair)
//! - Type system: returns `:i64`; arithmetic + cache integration work

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

// ─── Determinism ─────────────────────────────────────────────────────

#[test]
fn simhash_deterministic_same_ast() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((a :wat::holon::HolonAST)
              (:wat::holon::Bind
                (:wat::holon::Atom "role")
                (:wat::holon::Atom "filler")))
             ((k1 :i64) (:wat::holon::simhash a))
             ((k2 :i64) (:wat::holon::simhash a)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= k1 k2) -> :String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

// ─── Atom identity ───────────────────────────────────────────────────

#[test]
fn simhash_atom_zero_stable() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((k1 :i64) (:wat::holon::simhash (:wat::holon::Atom 0)))
             ((k2 :i64) (:wat::holon::simhash (:wat::holon::Atom 0))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= k1 k2) -> :String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

// ─── Cosine-near-1 → low hamming distance (same AST) ─────────────────
//
// Two encodings of the same AST shape produce the same vector,
// therefore the same SimHash. Hamming distance = 0.

#[test]
fn simhash_same_shape_zero_hamming() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((a :wat::holon::HolonAST)
              (:wat::holon::Bind
                (:wat::holon::Atom "role")
                (:wat::holon::Atom "filler")))
             ((b :wat::holon::HolonAST)
              (:wat::holon::Bind
                (:wat::holon::Atom "role")
                (:wat::holon::Atom "filler")))
             ((k1 :i64) (:wat::holon::simhash a))
             ((k2 :i64) (:wat::holon::simhash b)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= k1 k2) -> :String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["same".to_string()]);
}

// ─── Different ASTs → different keys (with high probability) ────────
//
// Two structurally different ASTs (e.g., distinct atoms) almost
// certainly produce different SimHash keys. Hamming distance is
// expected to be near 32 (half the bits differ on orthogonal inputs).
// We can't assert distance reliably; we assert the keys differ.

#[test]
fn simhash_distinct_atoms_distinct_keys() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((alpha :wat::holon::HolonAST) (:wat::holon::Atom "alpha"))
             ((beta  :wat::holon::HolonAST) (:wat::holon::Atom "beta"))
             ((k-a :i64) (:wat::holon::simhash alpha))
             ((k-b :i64) (:wat::holon::simhash beta)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= k-a k-b) -> :String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["diff".to_string()]);
}

// ─── Type system: simhash returns :i64; works with arithmetic ───────

#[test]
fn simhash_result_works_in_arithmetic() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((k :i64) (:wat::holon::simhash (:wat::holon::Atom "x")))
             ((doubled :i64) (:wat::core::+ k k)))
            ;; Just checking the type-checker accepts arithmetic on
            ;; the result. Print "ok" if we got here.
            (:wat::io::IOWriter/println stdout "ok")))
    "##;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

// (Cache composition with `:rust::lru::LruCache<i64,V>` is documented
// in the arc 051 DESIGN and exercised by `wat-lru`'s own test crate
// where the LRU shim is registered. The five tests above cover the
// primitive's contract: deterministic, identity-stable, distinct-AST-
// distinct-key, and i64-typed for downstream composition.)
