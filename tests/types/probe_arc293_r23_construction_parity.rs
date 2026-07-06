//! Arc 293.R2.3 — construction-form parity: every type-name is its own constructor (bare `:T`), `/new` dropped.
//!
//! Records already construct via the bare type name (`(:Pt 1 2)` — the defrecord macro emits a bare-name `defn`
//! ctor). Structs and newtypes are the holdouts: they construct only via `:T/new` (`register_struct_methods` /
//! `register_newtype_methods` mint at `{T}/new`). This breaks "the only variance is the nature" — construction
//! FORM differs by nature for no reason. 293.R2.3 mints the ctor at the BARE name for structs + newtypes too, and
//! annihilates `/new` (builder's decided call, NOTE-base-struct-horizon: "every type-name is its own constructor").
//!
//! RED at HEAD: `(:b::Pt 3 4)` / `(:b::Price 38)` are unresolved. GREEN after R2.3 — `(:b::probe)` = 41.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// A struct and a newtype construct via their bare type name, at parity with records.
#[test]
fn construction_form_parity_bare_ctor_for_struct_and_newtype() {
    let world = startup_beside(file!())
        .expect("293.R2.3: a struct + newtype must construct via the bare type name (parity with records)");

    let got = eval_in_frozen(
        &wat::parse_one!("(:b::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:b::probe) must construct (:b::Pt 3 4) + (:b::Price 38) via bare ctors")
    .value_owned();

    match got {
        Value::i64(n) => assert_eq!(n, 41, "(:b::Pt/x (:b::Pt 3 4)) + (:b::Price/0 (:b::Price 38)) = 3 + 38; got {n}"),
        other => panic!("expected i64 41 from the bare-ctor parity; got {other:?}"),
    }
}
