//! A service severed by its owner must not read to a client as a clean close.
//!
//! `:Shutdown`'s own declaration (`wat/spawn.wat`) names the cause — "owner dropped the handle
//! (self-peer drained)" — and the serve loop used to answer `nil`, so every connected client's
//! next `recv'` read a bare EOF and reported `RecvOutcome::Closed`. A clean-close label on a
//! service that did not close cleanly. Arc 278's RST stone killed exactly that mute for the CRASH
//! kinds; the ordinary return, where the owner simply lets go, was the one path it never reached.
//!
//! The two cases below differ in ONE thing — whether the drive call sits in the `let`'s body
//! (tail position, so the scope holding the handle is released before the call runs) or in a
//! binding. Keeping the control in the same gate is deliberate: without it, a fixture that had
//! stopped discriminating would still pass on the subject alone.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn drive(entry: &str) -> String {
    let world = startup_beside(file!()).expect("startup should succeed (echo service + kernel vocab)");
    let func = world
        .symbols()
        .get(entry)
        .unwrap_or_else(|| panic!("{entry} not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("{entry} raised: {e:?}"));
    match got {
        Value::String(s) => s.to_string(),
        other => panic!("{entry} must return the outcome's NAME as a String; got {other:?}"),
    }
}

/// The owner released the handle while it was still lexically in scope (the drive is a tail call).
/// The client must be told the service was SEVERED — not handed a mute `Closed`.
#[test]
fn an_owner_drop_reaches_the_client_as_severed() {
    let got = drive(":user::owner-drop-is-named");
    assert_eq!(
        got, "SEVERED",
        "an owner-dropped handle must reach the client as LociDiedError::Severed. \
         \"CLOSED:MUTE\" is the regression this gate exists for — a clean-close label on a service \
         that did not close cleanly. \"LOST:Disconnected\" means the sever is being collapsed into \
         the catch-all (check the trust-boundary scrub still passes Severed through, beside Stopped). \
         \"REPLIED\" means the handle was not released and the fixture has stopped discriminating."
    );
}

/// The one-variable control: same code, drive moved into a binding so the handle outlives it.
/// If this ever reports SEVERED, the subject above is measuring nothing.
#[test]
fn a_held_handle_still_replies() {
    let got = drive(":user::held-handle-still-replies");
    assert_eq!(
        got, "REPLIED",
        "with the handle held for the whole drive the service must simply answer; anything else \
         means this control has stopped being a control and the sever test above proves nothing"
    );
}
