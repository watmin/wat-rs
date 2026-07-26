//! FM 2-bis probe — arc 251 Stone 251.5a-ii: `write-forms`, the round-trip closed.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_write_forms`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn contract_01_homoiconic_roundtrip_dirty_in_clean_out() {
    // just-eval (rubric): `:user::c01` lives in the co-located fixture.
    let result = call_beside_value(file!(), ":user::c01").map_err(|e| format!("eval: {e:?}"));
    match result {
        Ok(Value::bool(true)) => {}
        other => panic!(
            "read-string(::source) → write-forms(clean EDN) → read-string → a List: \
             the round-trip closes in wat; got {other:?}"
        ),
    }
}
