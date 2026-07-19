//! Arc 278 — Option A RED gate: a macro that GENERATES a service.
//!
//! A `defmacro` emits `(do (defsurface … :messages […]) (defservice :satisfies …))`
//! — the shape the sift Rules-form UX needs (splice user defs into a generated
//! surface). The nested defsurface's `:messages` accessors + `::Variant` ctors
//! must mint exactly as a hand-written top-level defsurface's do.
//!
//! RED at HEAD: `hoist_surface_messages` (src/macros/expand.rs:53) fires only for
//! a DIRECT top-level defsurface; a do-nested one is spliced up raw, its
//! `:messages` never hoisted → `:probe::Echo::Req/c` is UnresolvedReference at
//! freeze → the `startup_beside` below panics.
//!
//! GREEN when Option A makes the do/let-splice re-enter the top-level per-form
//! dispatch (so a do-nested defsurface gets the same hoist).
//!
//! Run: cargo test --release -p wat --test probe_arc278_macro_generates_service

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn macro_generated_service_mints_its_surface_messages_accessors() {
    let world = startup_beside(file!()).expect(
        "startup: a macro emitting (do (defsurface …)(defservice :satisfies …)) must hoist the \
         nested surface's :messages so :probe::Echo::EchoRequest/c mints — at HEAD this is a \
         StartupError: UnresolvedReference (the do-nested defsurface bypasses hoist_surface_messages; \
         expand.rs:53). GREEN when Option A lands.",
    );
    let func = world
        .symbols()
        .get(":user::check")
        .expect(":user::check in fixture")
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!(":user::check raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "the macro-generated surface's field accessor :probe::Echo::EchoRequest/c must mint + \
         return 42; got {got:?}",
    );
}
