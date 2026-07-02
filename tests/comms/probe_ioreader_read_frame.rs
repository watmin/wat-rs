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
use wat::freeze::{startup_bare, startup_beside};
use wat::io::{StringIoReader, WatReader, eval_ioreader_read_frame};
use wat::runtime::{Environment, Value};
use wat::scope::Identifier;
use wat::value::TrackedValue;

/// Gate 1 — multi-line EDN map frame reads back as Some(frame-string).
///
/// Writes a 4-line pretty-printed map to a StringIoReader, then calls
/// `eval_ioreader_read_frame` directly via the Rust entry point.
/// Expects `Value::Option(Some(Value::String))` where the string contains
/// the map keys.
#[test]
fn read_frame_multiline_edn_map() {
    let world = startup_bare().expect("startup_bare should succeed");
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
    let arg_ast = WatAST::Symbol(Identifier::bare("__reader__"), wat::rust_caller_span!());

    let result = eval_ioreader_read_frame(&[arg_ast], &env, sym, &wat::rust_caller_span!());
    match result {
        Ok(Value::Option(opt)) => match opt.as_ref() {
            Some(Value::String(s)) => {
                assert_eq!(
                    s.as_str(),
                    "{\n  :a 1\n  :b 2\n}",
                    "read-frame must return the exact EDN map frame golden"
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
    let world = startup_bare().expect("startup_bare should succeed");
    let sym = world.symbols();

    let reader_arc: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let reader_val = Value::io__IOReader(reader_arc);

    let tv: TrackedValue = reader_val.into();
    let env = Environment::new()
        .child()
        .bind_unknown_span("__reader__", tv)
        .build();

    let arg_ast = WatAST::Symbol(Identifier::bare("__reader__"), wat::rust_caller_span!());

    let result = eval_ioreader_read_frame(&[arg_ast], &env, sym, &wat::rust_caller_span!());
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
/// Loads the co-located `.wat` fixture (which calls `IOReader/read-frame` in main)
/// and asserts the checker succeeds. Exercises the check.rs registration +
/// runtime.rs dispatch arm.
#[test]
fn read_frame_type_checks_and_startup_succeeds() {
    match startup_beside(file!()) {
        Ok(_) => {}
        Err(e) => panic!(
            "(:wat::io::IOReader/read-frame r) must type-check and freeze without error; got: {}",
            e
        ),
    }
}
