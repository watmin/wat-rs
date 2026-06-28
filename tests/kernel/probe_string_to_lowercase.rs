//! `:wat::core::string::to-lowercase` — the string lowercase primitive.
//!
//! Surfaced as a genuine substrate gap during arc 209 C.3: `defservice` derives lowercase
//! kebab fn names (`:Get` → `get`) from PascalCase op keywords, which needs a lowercase verb.
//! `to-lowercase` did not exist at HEAD (only the Rust `to_lowercase` method) — this is its
//! direct test. Pure + total (Rust `String::to_lowercase`, no IO), so it is also admitted to the
//! macro purity fence (`is_pure_total`) — exercised transitively by the defservice macro.
//!
//! Run: cargo test --release -p wat --test probe_string_to_lowercase

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn lower(input: &str) -> String {
    let world = startup_beside(file!()).expect("startup");
    let call = format!("(:user::lower {input:?})");
    let ast = wat::parse_one!(&call).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("lower")
    {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn to_lowercase_downcases() {
    assert_eq!(lower("GetObject"), "getobject");
    assert_eq!(lower("Increment"), "increment");
    assert_eq!(lower("already-lower"), "already-lower");
    assert_eq!(lower("MiXeD123"), "mixed123");
    assert_eq!(lower(""), "");
}
