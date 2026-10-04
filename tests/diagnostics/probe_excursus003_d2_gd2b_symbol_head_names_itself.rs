//! Excursus 003 strike D2, GD2b — a symbol-headed call names its OWN activation, not
//! whatever special form happens to be enclosing it.
//!
//! `AUDIT-the-shape-of-an-error.md` § "Strike D landed", finding 2: `(f 1)`, with `f` a
//! local bound to a non-callable, used to raise `NotCallable` with its Rust frame named
//! `:wat::core::let` — the raise has nothing to do with `let`; it is the application
//! `(f 1)` itself that failed. `BRIEF-shape-strike-D2-every-raise-names-its-activation.md`
//! item 1's rule: name the activation by the callee's resolved function name when the
//! head resolves to a function, otherwise the head symbol AS WRITTEN (`f`) — the one
//! honest fact in hand before resolution, and exactly what this raise is about.
//!
//! WAT fixture: tests/diagnostics/probe_excursus003_d2_gd2b_symbol_head_names_itself.wat
//!
//! Mutation (recorded in the strike report, not re-encoded here as a permanently
//! mutated copy): drop the `ActivationGuard::enter(ident.as_str())` call at the
//! `eval_list`/`eval_tail` `WatAST::Symbol` application-path call sites in
//! `src/runtime.rs` — RED, `frame_fn(&frames[0])` reads `:wat::core::let` again.

use wat::edn::contract::ToEdn;
use wat::freeze::call_beside_value;
use wat_edn::OwnedValue;

fn get_field<'a>(pairs: &'a [(OwnedValue, OwnedValue)], key: &str) -> &'a OwnedValue {
    for (k, v) in pairs {
        if let Some(kw) = k.as_keyword() {
            if kw.name() == key && kw.namespace().is_none() {
                return v;
            }
        }
    }
    panic!("key :{key} not found in map {pairs:?}");
}

fn frame_fn(frame: &OwnedValue) -> String {
    let (tag, body) = frame.as_tagged().expect("frame is a tagged Frame");
    assert_eq!(tag.namespace(), "wat.kernel", "frame tag namespace: {tag}");
    assert_eq!(tag.name(), "Frame", "frame tag name: {tag}");
    let pairs = body.as_map().expect("Frame body is a map");
    get_field(pairs, "fn").as_str().expect(":fn is a String").to_string()
}

#[test]
fn gd2b_symbol_headed_call_names_itself_not_the_enclosing_let() {
    let err = call_beside_value(file!(), ":my::test::probe-notcallable")
        .expect_err("f is bound to the i64 5, not a function; applying it must raise NotCallable");

    let edn = err.to_edn();
    let (tag, body) = edn.as_tagged().expect("RuntimeError EDN is tagged");
    assert_eq!(tag.namespace(), "wat.runtime");
    assert_eq!(tag.name(), "NotCallable", "the fixture must raise NotCallable, not something else");
    let pairs = body.as_map().expect("RuntimeError EDN body is a map");
    let frames = get_field(pairs, "frames").as_vector().expect(":frames is a vector");
    assert!(!frames.is_empty(), "NotCallable's real producer must carry a Rust frame");

    // frames[0] is the INNERMOST frame — the one Rust frame `RuntimeError::new` always
    // carries (strike D: Rust frame first, true innermost position).
    assert_eq!(
        frame_fn(&frames[0]),
        "f",
        "the Rust frame must name the head AS WRITTEN (`f`) — never the enclosing \
         `:wat::core::let`, which is what this gate exists to catch a regression of: {frames:?}"
    );
}
