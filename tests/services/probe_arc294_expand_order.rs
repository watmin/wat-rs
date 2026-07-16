//! Arc 294 item 9a — REGRESSION: sequential macro registration during expansion.
//!
//! See the co-located `.wat` for the full root. In one line: a `defservice` handler that
//! constructs the service's OWN minted `::State`/`::Record` used to die at eval with
//! `#wat.runtime/UnknownFunction: unknown function: :probe::echo::State`, because
//! `expand_form` held the registry immutably and expanded the `serve` body before the
//! companion `defmacro`s in its own emitted `do` had registered.
//!
//! A/B: `ping` (constructs nothing minted) is the CONTROL — green even at the broken HEAD;
//! `bump` (constructs its own minted types) is the regression — red at the broken HEAD.
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

/// CONTROL — a handler that constructs nothing minted. Green before the fix too; a red
/// here means the sequential-registration walk broke the ordinary expansion path.
#[test]
fn control_ping_handler_without_minted_construction() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-ping)").expect("parse");
    let tv = eval_in_frozen(&ast, &world, &Environment::new()).expect("ping round-trip");
    assert_eq!(
        tv.value_owned(),
        wat::runtime::Value::i64(1),
        "ping should round-trip its PingResponse value"
    );
}

/// THE REGRESSION — a handler constructing the defservice's OWN minted `::State`/`::Record`.
/// Red before sequential registration (`unknown function: :probe::echo::State`).
#[test]
fn bump_handler_constructs_its_own_minted_state() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-bump)").expect("parse");
    let tv = eval_in_frozen(&ast, &world, &Environment::new()).expect("bump round-trip");
    assert_eq!(
        tv.value_owned(),
        wat::runtime::Value::i64(7),
        "bump's handler must construct its own minted ::State/::Record and reply"
    );
}
