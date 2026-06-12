//! A2 RED probe — program::Env as a typed extensible recordtype base.
//!
//! At HEAD `:wat::program::Env` is a `typealias = HashMap<keyword, HolonAST>` (the dynamic store
//! whose cast-accessors A1 deleted). A2 replaces it with a recordtype base carrying the first
//! system field `wat.started-at : Instant`, defined in blessed stdlib `wat/program.wat` via
//! `Record::def`, and swaps the spawn arg[1] check `unify`→`assignable` so an *extended* env
//! (a child recordtype) satisfies the base.
//!
//! RED at HEAD on both counts:
//!   C01 — program::Env has no record constructor/accessor (it's a HashMap) → no `wat.started-at`.
//!   C02 — program::Env can't be a recordtype PARENT (it's a typealias, not a record).
//!
//! Run: `cargo test --release --test probe_arc258_program_env_record`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_i64(decls: &str, body: &str) -> Result<i64, String> {
    let src = format!(
        "{decls}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

#[test]
fn c01_base_record_started_at() {
    // Construct the base program::Env with started-at = at-millis(5000), read it back.
    let got = eval_i64(
        "",
        "(:wat::time::epoch-millis \
           (:wat::program::Env/wat.started-at \
             (:wat::program::Env (:wat::time::at-millis 5000) (:wat::time::at-millis 0) 0 0 :wat::program::PeerKind::process (:wat::program::EmptyEnv))))",
    );
    assert_eq!(got, Ok(5000),
        "program::Env is a record with a wat.started-at : Instant field, constructed + read");
}

#[test]
fn c02_user_extends_program_env() {
    // A program EXTENDS program::Env with its own typed field; the extension is a subtype.
    // Construct it, read the inherited wat.started-at AND the user field.
    let got = eval_i64(
        "(:wat::core::recordtype :user::MyEnv :wat::program::Env [port <- :wat::core::i64])",
        "(:user::MyEnv/port \
           (:user::MyEnv (:wat::time::at-millis 1) (:wat::time::at-millis 0) 0 0 :wat::program::PeerKind::process (:wat::program::EmptyEnv) 8080))",
    );
    assert_eq!(got, Ok(8080),
        "a user recordtype can extend :wat::program::Env (it is a record base, not a HashMap typealias)");
}
