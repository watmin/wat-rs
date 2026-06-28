//! Arc 170 slice 3 Gap F-1 — regression probes for top-level `do` splicing of `enum` forms.
//!
//! Two probes confirm that `(:wat::core::defenum ...)` forms inside a top-level
//! `(:wat::core::do ...)` pre-register their tagged-variant constructor stubs.
//!
//! Both probes FAIL before Gap F-1 ships; both PASS after.
//!
//! Probe 1: enum declaration + define calling tagged-variant constructor, both in top-level `do`.
//! Probe 2: `defmacro` that emits `do` wrapping enum + define — the Phase E use case directly.

use wat::freeze::startup_from_file;

/// Probe 1 — enum + define calling tagged-variant constructor, both in a top-level `do`.
#[test]
fn probe_do_enum_constructor_visible() {
    let world = startup_from_file("tests/macros/probe_do_splice_enum_constructor.wat").expect("freeze");
    assert!(world.symbols().get(":my::Request::Push").is_some(), ":my::Request::Push not registered");
    assert!(world.symbols().get(":my::make-push").is_some(), ":my::make-push not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `do` wrapping enum + define.
#[test]
fn probe_do_enum_via_macro_emission() {
    let world = startup_from_file("tests/macros/probe_do_splice_enum_via_macro.wat").expect("freeze");
    assert!(world.symbols().get(":my::probe::Event::Created").is_some(), ":my::probe::Event::Created not registered");
    assert!(world.symbols().get(":my::probe::Event::Deleted").is_some(), ":my::probe::Event::Deleted not registered");
    assert!(world.symbols().get(":my::probe::make-created").is_some(), ":my::probe::make-created not registered");
}
