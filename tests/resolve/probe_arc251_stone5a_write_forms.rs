//! FM 2-bis probe — arc 251 Stone 251.5a-ii: `write-forms`, the round-trip closed.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_write_forms`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn contract_01_homoiconic_roundtrip_dirty_in_clean_out() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::c01)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"));
    match result {
        Ok(Value::bool(true)) => {}
        other => panic!(
            "read-string(::source) → write-forms(clean EDN) → read-string → a List: \
             the round-trip closes in wat; got {other:?}"
        ),
    }
}
