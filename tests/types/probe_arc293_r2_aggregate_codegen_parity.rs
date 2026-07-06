//! Arc 293.R2 — the aggregate codegen annihilation: ONE toolkit, nature is the only variance.
//!
//! THE PARITY BREAK: ctor + accessor synthesis is still TWO Rust functions split by nature —
//! `register_struct_methods` (runtime.rs:924, `nature == Struct`) and `register_record_methods`
//! (runtime.rs:1315, `nature != Struct`) — and they drifted. The struct path carries `type_params` and
//! uses the bare name for the accessor key; the record path hardcodes `type_params: vec![]` and builds the
//! key from `entry.name` (which carries `<T>`), so a GENERIC record/holon-record's field accessor lands at
//! the mangled key `:R<T>/v` and `:R/v` is never registered.
//!
//! RED at HEAD: startup fails — `:r2::CR/v` / `:r2::HR/v` are unresolved. GREEN after 293.R2a — one
//! `register_aggregate_methods` mints accessors for all three natures, generic-aware, bare key.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// Generic core-record + holon-record field accessors resolve, at parity with the generic struct;
/// and a holon record is accepted where a core `:wat::core::Record` is wanted (policy c).
#[test]
fn aggregate_codegen_parity_generic_record_accessors() {
    let world = startup_beside(file!())
        .expect("293.R2: a generic record + holon-record must expose their field accessor (one aggregate toolkit)");

    let got = eval_in_frozen(
        &wat::parse_one!("(:r2::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:r2::probe) must read :r2::CR/v + :r2::HR/v + :r2::ST/v")
    .value_owned();

    match got {
        Value::i64(n) => assert_eq!(
            n, 60,
            "10 (core-record) + 20 (holon-record) + 30 (struct) across the three natures; got {n}"
        ),
        other => panic!("expected i64 60 from the three-nature accessor parity; got {other:?}"),
    }
}
