//! Arc 232 Stone 232.2 — `assignable(T, :P)`: a `:P`-typed param accepts an extender.
//!
//! 232.1 registered defprotocol/extend-type (the registries exist). 232.2 makes the protocol a
//! usable BOUND: a fn parameter typed `:P` accepts a value whose type `extend-type`s `:P`, and
//! rejects one that doesn't. The grounded plan: `extend-type :T :P` registers a subtype-parent
//! edge `T → P` (mirroring recordtype's `:wat::Record` parent at types.rs:416); `assignable`
//! already consults `is_subtype` for Path→Path (check.rs:13566), so the edge is all that's needed —
//! UNLESS `:P` must also be a registered TypeEnv type for the annotation to be accepted (this probe
//! at HEAD-after-232.1 will reveal which: "unknown type :P" → need a TypeDef; "not assignable" →
//! need only the edge).
//!
//! RED at HEAD (232.1 shipped): a `:t::Robot` is not assignable to `:t::Greeter` (no edge yet).
//! GREEN once 232.2 wires the satisfaction edge.
//!
//! Run: cargo test --release -p wat --test probe_arc232_2_protocol_assignable

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn p_typed_param_accepts_an_extender() {
    let world = startup_beside(file!())
        .expect("startup should succeed (232.2: a :P-typed param accepts an extender)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(99)),
        "expected 99: a :t::Robot (extend-types :t::Greeter) passed where :t::Greeter is required \
         must type-check via the satisfaction edge; got {got:?}"
    );
}

// Anti-over-reach: a record that does NOT extend the protocol must STILL be rejected where :P is
// required. Proves the edge is precise (only registered extenders satisfy), not a blanket accept.
// This errors at HEAD and after 232.2 alike — a regression guard, not a RED→GREEN gate.
#[test]
fn p_typed_param_rejects_a_non_extender() {
    let result = startup_from_file(
        "tests/types/probe_arc232_2_protocol_assignable_non_extender_bad.wat",
    );
    assert!(
        result.is_err(),
        ":t::Rock does NOT extend-type :t::Greeter, so passing it where :t::Greeter is required \
         must remain a check error (the satisfaction edge is precise, not a blanket accept)"
    );
}
