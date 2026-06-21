//! `:wat::io::IOReader/read-frame` — multi-line EDN value accumulation probe.
//!
//! Verifies that `IOReader/read-frame` accumulates physical lines from an
//! IOReader until they form a complete EDN value (a single `Option<String>`
//! containing the frame), and returns `None` on clean EOF.
//!
//! This is the consumer-side proof that `read_framed_edn` is live: the
//! dispatch arm in runtime.rs routes to `eval_ioreader_read_frame`, which
//! calls `crate::edn_shim::read_framed_edn(|s| reader.read_line(s), span)`.
//!
//! Three tests:
//! 1. A MULTI-LINE EDN map written to a StringIoReader decodes to Some(frame).
//! 2. An empty reader (immediate EOF) returns None.
//! 3. Type-checker accepts IOReader/read-frame and startup succeeds.

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::io::{StringIoReader, WatReader, eval_ioreader_read_frame};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
use wat::scope::Identifier;
use wat::span::Span;
use wat::value::TrackedValue;

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Gate 1 — multi-line EDN map frame reads back as Some(frame-string).
///
/// Writes a 4-line pretty-printed map to a StringIoReader, then calls
/// `eval_ioreader_read_frame` directly via the Rust entry point.
/// Expects `Value::Option(Some(Value::String))` where the string contains
/// the map keys.
#[test]
fn read_frame_multiline_edn_map() {
    let world = freeze_ok(
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let sym = world.symbols();

    // A multi-line EDN map: 4 physical lines, 1 logical value.
    let frame_src = "{\n  :a 1\n  :b 2\n}\n";
    let reader_arc: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(frame_src.to_string()));
    let reader_val = Value::io__IOReader(reader_arc);

    // Inject the reader into a child env under the name "__reader__".
    let tv: TrackedValue = reader_val.into();
    let env = Environment::new()
        .child()
        .bind_unknown_span("__reader__", tv)
        .build();

    // Build a synthetic Symbol AST node pointing at "__reader__" in the env.
    let arg_ast = WatAST::Symbol(Identifier::bare("__reader__"), Span::unknown());

    let result = eval_ioreader_read_frame(&[arg_ast], &env, sym, &Span::unknown());
    match result {
        Ok(Value::Option(opt)) => match opt.as_ref() {
            Some(Value::String(s)) => {
                assert!(
                    s.contains(":a") && s.contains(":b"),
                    "read-frame must return the map frame string containing :a and :b; got: {:?}",
                    s
                );
            }
            other => panic!(
                "read-frame: expected Some(String); got: {:?}",
                other
            ),
        },
        Ok(other) => panic!("read-frame: expected Option(Some(String)); got: {:?}", other),
        Err(e) => panic!("read-frame: unexpected error: {}", e),
    }
}

/// Gate 2 — clean EOF on empty reader returns None.
#[test]
fn read_frame_eof_returns_none() {
    let world = freeze_ok(
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let sym = world.symbols();

    let reader_arc: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let reader_val = Value::io__IOReader(reader_arc);

    let tv: TrackedValue = reader_val.into();
    let env = Environment::new()
        .child()
        .bind_unknown_span("__reader__", tv)
        .build();

    let arg_ast = WatAST::Symbol(Identifier::bare("__reader__"), Span::unknown());

    let result = eval_ioreader_read_frame(&[arg_ast], &env, sym, &Span::unknown());
    match result {
        Ok(Value::Option(opt)) => match opt.as_ref() {
            None => {} // correct: EOF → None
            Some(v) => panic!("read-frame on empty reader must return None; got Some({:?})", v),
        },
        Ok(other) => panic!("read-frame on empty reader: expected Option(None); got: {:?}", other),
        Err(e) => panic!("read-frame on empty reader: unexpected error: {}", e),
    }
}

/// Gate 3 — type-checker accepts IOReader/read-frame and startup succeeds.
///
/// Freezes a minimal wat program that calls `(:wat::io::IOReader/read-frame r)`
/// and asserts the checker succeeds. Exercises the check.rs registration +
/// runtime.rs dispatch arm.
#[test]
fn read_frame_type_checks_and_startup_succeeds() {
    let src = r#"
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::let
            [r (:wat::io::IOReader/from-string "42\n")]
            (:wat::io::IOReader/read-frame r)
            nil))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => {}
        Err(e) => panic!(
            "(:wat::io::IOReader/read-frame r) must type-check and freeze without error; got: {}",
            e
        ),
    }
}
