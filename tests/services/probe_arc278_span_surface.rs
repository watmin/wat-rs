//! arc 278 stone Span.1 — the `:wat::telemetry'::Span` PRODUCER surface acceptance gate. A toy
//! satisfier (`:probe::toy-span'`, NOT the real `span'`) proves the surface freezes, is satisfiable
//! via `:satisfies`, and all four ops (`incr`/`timed`/`log`/`close`) reply through the wire. Mirrors
//! probe_arc278_journal_surface. The real `span'` (holds a `journal'` peer, accumulates, emits on
//! close) is stone Span.2.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn span_surface_freezes_and_all_four_ops_reply() {
    let world = startup_beside(file!())
        .expect("startup should succeed (the Span surface must freeze from the baked telemetry.wat)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("toy-span' driving incr/timed/log/close raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(1)),
        "expected the toy Span satisfier to reply to all four ops and return CloseResponse::Done \
         (encoded as 1); got {got:?}"
    );
}
