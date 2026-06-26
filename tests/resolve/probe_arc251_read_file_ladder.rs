//! Arc 251/261 — the read-file ladder: `IOReader/open-file` + `read-all-string` + the
//! `read-file` one-shot. The read mirror of the write ladder (`IOWriter/open-file` +
//! `to-string` + `write-file`); the rungs the self-hosted fix-wat migration runner rides.
//!
//! Proves byte-faithful, round-trip read (write-file → read-file → identical bytes),
//! and that the explicit-handle form (`open-file` → `read-all-string`) agrees.
//!
//! Run: cargo test --release -p wat --test probe_arc251_read_file_ladder

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A fixture with comments, unicode, and a trailing newline — byte-faithfulness must hold.
const CONTENT: &str = ";; a comment — ünïcode ✓\n(:wat::core::defn :x [] -> :wat::core::nil nil)\n";

fn eval_string(program: &str, call: &str) -> String {
    let world = startup_from_source(program, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!(call).expect("parse call");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected String; got {other:?}"),
    }
}

#[test]
fn read_file_round_trips_byte_faithfully() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("wat_read_file_ladder_{}.wat", std::process::id()));
    let path_str = path.to_str().expect("utf8 path").to_string();
    let escaped = path_str.replace('\\', "\\\\").replace('"', "\\\"");

    // write-file (existing) → read-file (new): the round-trip must be byte-identical.
    let prog = format!(
        "(:wat::core::defn :user::run [c <- :wat::core::String] -> :wat::core::String\n\
         \x20 (:wat::core::do\n\
         \x20   (:wat::io::write-file \"{escaped}\" c)\n\
         \x20   (:wat::io::read-file \"{escaped}\")))\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let cescaped = CONTENT.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    let got = eval_string(&prog, &format!("(:user::run \"{cescaped}\")"));
    assert_eq!(got, CONTENT, "read-file must return write-file's bytes exactly");

    // The explicit-handle rung agrees: open-file → read-all-string.
    let prog2 = format!(
        "(:wat::core::defn :user::run2 [] -> :wat::core::String\n\
         \x20 (:wat::io::IOReader/read-all-string (:wat::io::IOReader/open-file \"{escaped}\")))\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let got2 = eval_string(&prog2, "(:user::run2)");
    assert_eq!(got2, CONTENT, "open-file → read-all-string must agree with read-file");

    let _ = std::fs::remove_file(&path);
}
