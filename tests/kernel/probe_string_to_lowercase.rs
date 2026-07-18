//! `:wat::core::string::to-lowercase` — the string lowercase primitive.
//!
//! Surfaced as a genuine substrate gap during arc 209 C.3: `defservice` derives lowercase
//! kebab fn names (`:Get` → `get`) from PascalCase op keywords, which needs a lowercase verb.
//! `to-lowercase` did not exist at HEAD (only the Rust `to_lowercase` method) — this is its
//! direct test. Pure + total (Rust `String::to_lowercase`, no IO), so it is also admitted to the
//! macro purity fence (`is_pure_total`) — exercised transitively by the defservice macro.
//!
//! Run: cargo test --release -p wat --test probe_string_to_lowercase

use std::sync::Arc;

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn lower(input: &str) -> String {
    let world = startup_beside(file!()).expect("startup");
    let func = world.symbols().get(":user::lower").expect(":user::lower").clone();
    let arg = Value::String(Arc::new(input.to_string()));
    match apply_function(func, vec![arg], world.symbols(), wat::rust_caller_span!()).expect("lower")
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
