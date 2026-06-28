//! Arc 232 Stone 232.1 — `defprotocol` + `extend-type` parse + register (the foundation).
//!
//! 232.1 is registry ONLY: a program may DECLARE a protocol and EXTEND a type to it. It cannot yet
//! type a param as `:P` (232.2 — the `assignable` edge) or CALL a protocol method (232.3 —
//! dispatch). So this probe declares a protocol + extends a record to it, with NO `:P`-typed param
//! and NO method call — and asserts the world builds. That isolates exactly "the forms parse +
//! register" without depending on the later stones.
//!
//! RED at HEAD: `:wat::core::defprotocol` is an unknown call head → `startup_from_source` fails to
//! resolve/register the top-level form, so the world never builds. GREEN once 232.1 ships the two
//! special forms + their registries.
//!
//! (The anti-fake Rust registry assertion — that `protocol_registrations` / `extend_registrations`
//! are actually populated — lives as a unit test next to `CheckEnv::from_symbols`, per the BRIEF.)
//!
//! Run: cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn defprotocol_and_extend_type_parse_and_register() {
    let world = startup_beside(file!())
        .expect("startup should succeed (232.1: defprotocol + extend-type parse + register)");
    let ast = wat::parse_one!("(:user::ok)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("ok raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: a program declaring a protocol + extending a record to it must build \
         (registry only — no dispatch yet); got {got:?}"
    );
}
