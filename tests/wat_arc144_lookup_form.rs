//! Integration coverage for arc 144 slice 1 — uniform `lookup_form`
//! reflection across the five form-kinds.
//!
//! The arc-143 reflection primitives (`:wat::runtime::lookup-define`,
//! `:wat::runtime::signature-of`, `:wat::runtime::body-of`) now
//! dispatch on a uniform `Binding` enum (5 variants). Slice 1 ships
//! UserFunction, Macro, Primitive, and Type coverage; SpecialForm
//! arrives in slice 2 (registry not yet populated; lookup_form's
//! SpecialForm path returns None today).
//!
//! These tests verify:
//!   1. Macro lookup — defmacro is reflected; lookup-define returns
//!      Some + emission carries `:wat::core::defmacro`; signature-of
//!      returns Some; body-of returns the template.
//!   2. Type lookup — struct decl is reflected; lookup-define returns
//!      Some + emission carries `:wat::core::struct`; signature-of
//!      returns Some + emission carries the type's name; body-of
//!      returns :None (types are body-less in the wat sense).
//!   3. User-function lookup — no regression vs arc 143's existing
//!      coverage; the refactor preserves UserFunction behavior exactly.
//!   4. Substrate-primitive lookup — same regression-guard for
//!      `:wat::core::foldl` post-Binding-refactor.
//!   5. Unknown name — all three primitives return :None for an
//!      unregistered name.

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

// ─── Macro lookup (NEW kind for arc 144) ────────────────────────────────────

#[test]
fn lookup_define_macro_returns_some_and_emits_defmacro_head() {
    // A registered defmacro now reflects through lookup-define. The
    // emission must carry the `:wat::core::defmacro` head keyword so
    // readers can distinguish a macro-decl from a function-decl in the
    // returned AST.
    let src = r##"
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `,x)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((def-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::lookup-define :my::ident))
             ((rendered :wat::core::String)
              (:wat::edn::write def-opt)))
            (:wat::io::IOWriter/println stdout rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one rendered line, got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains("defmacro"),
        "expected 'defmacro' head in rendered macro define-ast, got: {}",
        line
    );
    assert!(
        line.contains("my::ident"),
        "expected macro name 'my::ident' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_macro_returns_some() {
    let src = r##"
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `,x)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::signature-of :my::ident)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn body_of_macro_returns_some_with_template() {
    // body-of on a macro returns the stored template (the macro's
    // body field) — matches the FAQ "macros have bodies (their
    // template); primitives don't (Rust impl)".
    let src = r##"
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `,x)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::body-of :my::ident)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── Type lookup (NEW kind for arc 144) ─────────────────────────────────────

#[test]
fn lookup_define_struct_returns_some_and_emits_struct_head() {
    // A struct decl reflects through lookup-define. The emission's
    // head keyword is `:wat::core::struct` (slice 1's honest sentinel
    // shape — head + name + sentinel body slot; field rendering is a
    // future arc).
    let src = r##"
        (:wat::core::struct :my::Bar
          (open  :wat::core::f64)
          (close :wat::core::f64))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((def-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::lookup-define :my::Bar))
             ((rendered :wat::core::String)
              (:wat::edn::write def-opt)))
            (:wat::io::IOWriter/println stdout rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one rendered line, got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains("struct"),
        "expected 'struct' head in rendered type define-ast, got: {}",
        line
    );
    assert!(
        line.contains("my::Bar"),
        "expected type name 'my::Bar' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_struct_returns_some() {
    let src = r##"
        (:wat::core::struct :my::Point
          (x :wat::core::f64)
          (y :wat::core::f64))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::signature-of :my::Point)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn body_of_struct_returns_none() {
    // Types declare shapes; they don't have wat bodies. body-of
    // returns :None — honest about absence (the declaration is the
    // lookup-define output).
    let src = r##"
        (:wat::core::struct :my::Tick
          (price :wat::core::f64))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::body-of :my::Tick)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "pass"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── Regression guards: UserFunction + Primitive behavior preserved ─────────

#[test]
fn lookup_define_user_function_still_returns_some_post_refactor() {
    // Regression guard: arc 143's UserFunction emission behavior
    // (function_to_define_ast) must be unchanged after the Binding
    // refactor.
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
            (:wat::runtime::lookup-define :user::my-add)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn signature_of_substrate_primitive_still_returns_some_post_refactor() {
    // Regression guard: arc 143's Primitive emission behavior
    // (type_scheme_to_signature_ast) must be unchanged after refactor.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::signature-of :wat::core::foldl)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── Unknown name returns None across all three primitives ──────────────────

#[test]
fn all_three_primitives_return_none_on_unknown_name() {
    // For a name no registry knows, lookup-define / signature-of /
    // body-of all return :None. lookup_form's SpecialForm path is
    // currently a no-op (slice 2 territory); it does not produce
    // false-positive Some(...) for arbitrary names.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((d-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::lookup-define :no::such::thing))
             ((s-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::signature-of :no::such::thing))
             ((b-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::body-of    :no::such::thing)))
            (:wat::core::match d-opt
              -> :wat::core::unit
              ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail-d"))
              (:wat::core::None
                (:wat::core::match s-opt
                  -> :wat::core::unit
                  ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail-s"))
                  (:wat::core::None
                    (:wat::core::match b-opt
                      -> :wat::core::unit
                      ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail-b"))
                      (:wat::core::None    (:wat::io::IOWriter/println stdout "pass")))))))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}
