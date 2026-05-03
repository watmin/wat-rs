//! Integration coverage for arc 143 slice 1 — three substrate
//! introspection primitives:
//!   `:wat::core::lookup-define`
//!   `:wat::core::signature-of`
//!   `:wat::core::body-of`
//!
//! Each primitive takes a keyword name and returns
//! `:Option<wat::holon::HolonAST>`. Test coverage:
//!   1. User-define lookup — define a wat function, call the primitive,
//!      assert the returned Option is Some.
//!   2. Substrate-primitive lookup — call the primitive on
//!      `:wat::core::foldl`, assert Some.
//!   3. Unknown name — call on a non-existent name, assert None.
//!   4. `body-of` for substrate primitive returns None (not the sentinel).
//!
//! Tests use stdout pass/fail convention consistent with the rest of
//! the test suite. `edn::write` renders the HolonAST inside the Option
//! (Option(Some(v)) is transparent in EDN) so we can inspect its shape.

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

// ─── :wat::core::lookup-define ───────────────────────────────────────────

#[test]
fn lookup_define_user_define_returns_some() {
    // Define a user function and verify lookup-define returns Some.
    let src = r##"

        (:wat::core::define
          (:user::my-add (x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
          (:wat::core::+ x y))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::lookup-define :user::my-add)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn lookup_define_substrate_primitive_returns_some() {
    // :wat::core::foldl is a substrate primitive; lookup-define must
    // return Some (synthesised from its TypeScheme).
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::lookup-define :wat::core::foldl)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn lookup_define_unknown_name_returns_none() {
    // A completely made-up name returns :None.
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::lookup-define :user::this-does-not-exist)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "pass"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── :wat::core::signature-of ────────────────────────────────────────────

#[test]
fn signature_of_user_define_returns_some() {
    // User-defined function → signature-of returns Some.
    let src = r##"

        (:wat::core::define
          (:user::my-mul (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::i64)
          (:wat::core::* a b))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::signature-of :user::my-mul)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn signature_of_substrate_primitive_returns_some() {
    // :wat::core::foldl → synthesised head; must be Some.
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::signature-of :wat::core::foldl)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn signature_of_unknown_name_returns_none() {
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::signature-of :no::such::function)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "pass"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── :wat::core::body-of ─────────────────────────────────────────────────

#[test]
fn body_of_user_define_returns_some() {
    // User-defined function → body-of returns Some (the wat body).
    let src = r##"

        (:wat::core::define
          (:user::my-neg (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::- 0 n))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::body-of :user::my-neg)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn body_of_substrate_primitive_returns_none() {
    // Substrate primitives have no wat body — body-of must return :None.
    // (lookup-define returns the sentinel; body-of is honest about absence.)
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::body-of :wat::core::foldl)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "pass"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn body_of_unknown_name_returns_none() {
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::core::body-of :totally::unknown)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "pass"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── Shape verification via edn::write ───────────────────────────────────

#[test]
fn signature_of_foldl_renders_synthesised_shape() {
    // Verify the actual synthesised AST for :wat::core::foldl.
    // Expected: a Bundle whose first element is a Symbol for
    // ":wat::core::foldl<T,Acc>", followed by param-pair Bundles and
    // the return type. We render via edn::write and check key substrings.
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((sig-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::core::signature-of :wat::core::foldl))
             ((rendered :wat::core::String)
              (:wat::edn::write sig-opt)))
            (:wat::io::IOWriter/println stdout rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    let line = &out[0];
    // The rendered form is the EDN encoding of the HolonAST Bundle.
    // Option(Some(v)) is transparent in edn::write, so we get the
    // HolonAST directly. A Bundle renders as #wat-edn.holon/Bundle [...].
    // Key checks: the name includes "foldl", and "_a0" / "_a1" / "_a2"
    // synthetic param names appear (synthesised from TypeScheme which
    // has no real param names).
    assert!(
        line.contains("foldl"),
        "expected 'foldl' in rendered signature, got: {}",
        line
    );
    assert!(
        line.contains("_a0") && line.contains("_a1") && line.contains("_a2"),
        "expected synthesised param names _a0/_a1/_a2, got: {}",
        line
    );
    assert!(
        line.contains("Acc") && line.contains("Vec"),
        "expected type-param names T/Acc and Vec in signature, got: {}",
        line
    );
    // Verbatim EDN output of `(:wat::core::signature-of :wat::core::foldl)`:
    //   #wat-edn.holon/Bundle [
    //     #wat-edn.holon/Symbol ":wat::core::foldl<T,Acc>"
    //     #wat-edn.holon/Bundle [#wat-edn.holon/Symbol ":_a0" #wat-edn.holon/Symbol ":Vec<T>"]
    //     #wat-edn.holon/Bundle [#wat-edn.holon/Symbol ":_a1" #wat-edn.holon/Symbol ":Acc"]
    //     #wat-edn.holon/Bundle [#wat-edn.holon/Symbol ":_a2" #wat-edn.holon/Symbol ":fn(Acc,T)->Acc"]
    //     #wat-edn.holon/Symbol "->"
    //     #wat-edn.holon/Symbol ":Acc"]
    let _ = line; // asserted above
}

#[test]
fn lookup_define_user_function_contains_define_keyword() {
    // Verify lookup-define for a user function renders a structure that
    // includes the "define" form marker.
    let src = r##"

        (:wat::core::define
          (:user::my-square (x :wat::core::i64) -> :wat::core::i64)
          (:wat::core::* x x))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((def-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::core::lookup-define :user::my-square))
             ((rendered :wat::core::String)
              (:wat::edn::write def-opt)))
            (:wat::io::IOWriter/println stdout rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line");
    let line = &out[0];
    // The rendered HolonAST should contain both "define" and "my-square".
    assert!(
        line.contains("define"),
        "expected 'define' in rendered define-ast, got: {}",
        line
    );
    assert!(
        line.contains("my-square"),
        "expected 'my-square' in rendered define-ast, got: {}",
        line
    );
}
