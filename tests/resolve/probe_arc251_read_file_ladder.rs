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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// A fixture with comments, unicode, and a trailing newline — byte-faithfulness must hold.
const CONTENT: &str = ";; a comment — ünïcode ✓\n(:wat::core::defn :x [] -> :wat::core::nil nil)\n";

fn eval_string_call(call: &str) -> String {
    let world = startup_beside(file!()).expect("startup: read-file-ladder fixture must load");
    let ast = wat::parse_one!(call).expect("parse call");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected String; got {other:?}"),
    }
}

fn escape_wat_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[test]
fn read_file_round_trips_byte_faithfully() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("wat_read_file_ladder_{}.wat", std::process::id()));
    let path_str = path.to_str().expect("utf8 path").to_string();

    let escaped_path = escape_wat_string(&path_str);
    let escaped_content = escape_wat_string(CONTENT);

    // write-file (existing) → read-file (new): the round-trip must be byte-identical.
    let call = format!("(:user::run \"{}\" \"{}\")", escaped_path, escaped_content);
    let got = eval_string_call(&call);
    assert_eq!(got, CONTENT, "read-file must return write-file's bytes exactly");

    // The explicit-handle rung agrees: open-file → read-all-string.
    let call2 = format!("(:user::run2 \"{}\")", escaped_path);
    let got2 = eval_string_call(&call2);
    assert_eq!(got2, CONTENT, "open-file → read-all-string must agree with read-file");

    let _ = std::fs::remove_file(&path);
}
