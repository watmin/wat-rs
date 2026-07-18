//! Arc 251/261 — the read-file ladder: `IOReader/open-file` + `read-all-string` + the
//! `read-file` one-shot. The read mirror of the write ladder (`IOWriter/open-file` +
//! `to-string` + `write-file`); the rungs the self-hosted fix-wat migration runner rides.
//!
//! Proves byte-faithful, round-trip read (write-file → read-file → identical bytes),
//! and that the explicit-handle form (`open-file` → `read-all-string`) agrees.
//!
//! The fixture accepts `path` and `content` as parameters so this test can supply
//! a runtime-computed temp path (unique per process ID) without inlining WAT source.
//!
//! Run: cargo test --release -p wat --test probe_arc251_read_file_ladder

use std::sync::Arc;

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// A fixture with comments, unicode, and a trailing newline — byte-faithfulness must hold.
const CONTENT: &str = include_str!("probe_arc251_read_file_ladder__content.wat");

// just-eval (rubric): `:user::run`/`:user::run2` take real args (path/content), so the fixture's
// fns are fetched and `apply_function`-invoked directly with runtime-computed Values — no inline
// wat driver, no string-escaping needed now that the path/content never round-trip through wat
// source text at all.
fn call_string(fn_name: &str, args: Vec<Value>) -> String {
    let world = startup_beside(file!()).expect("startup: read-file-ladder fixture must load");
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no {fn_name} in fixture")).clone();
    match apply_function(func, args, world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected String; got {other:?}"),
    }
}

#[test]
fn read_file_round_trips_byte_faithfully() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("wat_read_file_ladder_{}.wat", std::process::id()));
    let path_str = path.to_str().expect("utf8 path").to_string();

    // write-file (existing) → read-file (new): the round-trip must be byte-identical.
    let got = call_string(
        ":user::run",
        vec![Value::String(Arc::new(path_str.clone())), Value::String(Arc::new(CONTENT.to_string()))],
    );
    assert_eq!(got, CONTENT, "read-file must return write-file's bytes exactly");

    // The explicit-handle rung agrees: open-file → read-all-string.
    let got2 = call_string(":user::run2", vec![Value::String(Arc::new(path_str))]);
    assert_eq!(got2, CONTENT, "open-file → read-all-string must agree with read-file");

    let _ = std::fs::remove_file(&path);
}
