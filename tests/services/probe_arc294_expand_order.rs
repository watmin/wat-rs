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
use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

/// Fire a fixture-defined nullary fn by NAME through the Rust API.
///
/// Deliberately NOT `parse_one!("(:user::compute-…)")`: the no-inlined-wat gate bans a
/// DRIVER expression built from an inline string just as it bans an inline world — "a test
/// must get its world from a co-located `.wat` fixture". The fixture owns every wat form;
/// the harness only names the entry point.
fn call_fixture_fn(world: &wat::freeze::FrozenWorld, name: &str) -> Value {
    let f = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} must be defined by the co-located fixture"))
        .clone();
    apply_function(f, Vec::new(), world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("{name} raised: {e:?}"))
}

/// CONTROL — a handler that constructs nothing minted. Green before the fix too; a red
/// here means the sequential-registration walk broke the ordinary expansion path.
#[test]
fn control_ping_handler_without_minted_construction() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        call_fixture_fn(&world, ":user::compute-ping"),
        Value::i64(1),
        "ping should round-trip its PingResponse value"
    );
}

/// THE REGRESSION — a handler constructing the defservice's OWN minted `::State`/`::Record`.
/// Red before sequential registration (`unknown function: :probe::echo::State`).
#[test]
fn bump_handler_constructs_its_own_minted_state() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        call_fixture_fn(&world, ":user::compute-bump"),
        Value::i64(7),
        "bump's handler must construct its own minted ::State/::Record and reply"
    );
}
