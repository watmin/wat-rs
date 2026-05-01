//! Arc 052 — Vector as a first-class wat-tier value.
//!
//! Coverage:
//! - Construct via `(:wat::holon::encode <ast>)`
//! - Equality (bit-exact, dim-aware)
//! - Vector as struct field (round-trip through field access)
//! - Polymorphic cosine: AST-AST, Vector-Vector, mixed
//! - Polymorphic dot: same surface as cosine
//! - Polymorphic simhash: AST input vs Vector input agree
//! - Type system: rejects non-AST, non-Vector inputs
//! - Cross-dim guards (deferred — single-d test fixtures only)

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

// ─── Construct + equality ────────────────────────────────────────────

#[test]
fn vector_construct_via_encode() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((v1 :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((v2 :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x"))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= v1 v2) -> :wat::core::String "equal" "diff"))))
    "##;
    assert_eq!(run(src), vec!["equal".to_string()]);
}

#[test]
fn vector_distinct_atoms_distinct_vectors() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "alpha")))
             ((vb :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "beta"))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= va vb) -> :wat::core::String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["diff".to_string()]);
}

// ─── Vector as struct field ─────────────────────────────────────────

#[test]
fn vector_as_struct_field_roundtrip() {
    let src = r##"
        (:wat::core::struct :my::Engram
          (label :wat::core::String)
          (vec :wat::holon::Vector))

        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((e :my::Engram) (:my::Engram/new "alpha" v))
             ((retrieved :wat::holon::Vector) (:my::Engram/vec e)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= v retrieved) -> :wat::core::String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

// ─── Polymorphic cosine — all four argument shapes ──────────────────

#[test]
fn polymorphic_cosine_ast_ast() {
    // Existing behavior preserved.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((a :wat::holon::HolonAST) (:wat::holon::Atom "x"))
             ((b :wat::holon::HolonAST) (:wat::holon::Atom "x"))
             ((c :wat::core::f64) (:wat::holon::cosine a b)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["near-1".to_string()]);
}

#[test]
fn polymorphic_cosine_vector_vector() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((vb :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((c :wat::core::f64) (:wat::holon::cosine va vb)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["near-1".to_string()]);
}

#[test]
fn polymorphic_cosine_ast_vector_mixed() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((a :wat::holon::HolonAST) (:wat::holon::Atom "x"))
             ((vb :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((c :wat::core::f64) (:wat::holon::cosine a vb)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["near-1".to_string()]);
}

#[test]
fn polymorphic_cosine_vector_ast_mixed() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((b :wat::holon::HolonAST) (:wat::holon::Atom "x"))
             ((c :wat::core::f64) (:wat::holon::cosine va b)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["near-1".to_string()]);
}

// ─── Polymorphic dot — Vector pair ──────────────────────────────────

#[test]
fn polymorphic_dot_vector_vector() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((vb :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((d :wat::core::f64) (:wat::holon::dot va vb)))
            ;; dot on the SAME vector should be sizeable (positive, bounded).
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> d 0.0) -> :wat::core::String "positive" "non-positive"))))
    "##;
    assert_eq!(run(src), vec!["positive".to_string()]);
}

// ─── Polymorphic simhash — AST and Vector inputs agree ──────────────

#[test]
fn polymorphic_simhash_ast_and_vector_agree() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((ast :wat::holon::HolonAST) (:wat::holon::Atom "alpha"))
             ((vec :wat::holon::Vector) (:wat::holon::encode ast))
             ((k-ast :wat::core::i64) (:wat::holon::simhash ast))
             ((k-vec :wat::core::i64) (:wat::holon::simhash vec)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= k-ast k-vec) -> :wat::core::String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["same".to_string()]);
}

// ─── Type system: cosine rejects non-holon-non-vector ───────────────

#[test]
fn polymorphic_cosine_rejects_string() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let* (((bad :f64) (:wat::holon::cosine "hello" "world")))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string bad))))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("HolonAST")
            || err.contains("Vector")
            || err.to_lowercase().contains("type"),
        "expected type-mismatch on string args; got {}",
        err
    );
}

// ─── Determinism: encode is reproducible ────────────────────────────

#[test]
fn vector_encode_deterministic_across_calls() {
    // Two encodes of an identical compound AST → equal Vectors.
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
             ((va :wat::holon::Vector) (:wat::holon::encode a))
             ((vb :wat::holon::Vector) (:wat::holon::encode b)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= va vb) -> :wat::core::String "deterministic" "drift"))))
    "##;
    assert_eq!(run(src), vec!["deterministic".to_string()]);
}
