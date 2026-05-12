//! Arc 170 slice 3 Gap F-2 — resolver quote-awareness probes.
//!
//! Three probes confirm that `resolve_references` does NOT recurse into
//! quote-family form arguments:
//!
//! - `:wat::core::forms` — all arguments are data; no descent
//! - `:wat::core::quote` — single argument is data; no descent
//! - `:wat::core::quasiquote` — template is data EXCEPT inside
//!   `:wat::core::unquote` / `:wat::core::unquote-splicing` escapes
//!
//! All three probes FAIL before Gap F-2 ships (resolver recurses into
//! quote-family arguments and flags unregistered inner call heads as
//! `UnresolvedReference`). All three PASS after.
//!
//! Pattern: the probe places quote-family forms inside a top-level
//! `(:wat::core::do ...)` form (which survives into `rest` and is
//! therefore walked by `resolve_references`). Inner call heads that
//! appear inside quote-family data are user paths NOT registered in
//! `sym.functions` — they are data, not live code.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — `:wat::core::forms` arguments are data; resolver must not descend.
///
/// A top-level `do` contains:
///   1. A helper define (`:my::probe-f2::forms-helper`) registered via
///      `preregister_fn_defs_in_do`.
///   2. A `forms` call whose argument contains
///      `(:my::probe-f2::ghost-inner arg)` — a form with an inner call
///      head that is NOT in `sym.functions`. This form is pure data.
///
/// Before Gap F-2: resolver recurses into `forms` children, finds
/// `:my::probe-f2::ghost-inner` as a call head, emits `UnresolvedReference`.
/// After Gap F-2: resolver skips `forms` children entirely; startup succeeds.
#[test]
fn probe_forms_argument_is_data() {
    let src = r#"
        (:wat::core::do
          (:wat::core::define (:my::probe-f2::forms-helper -> :wat::core::nil)
            :wat::core::nil)
          (:wat::core::forms
            (:my::probe-f2::ghost-inner some-arg)
            (:my::probe-f2::ghost-other 1 2 3)))
    "#;
    // The inner :my::probe-f2::ghost-inner and :my::probe-f2::ghost-other
    // are inside forms-quoted data — the resolver must NOT flag them.
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup must succeed: forms arguments are data, not live call heads");
}

/// Probe 2 — `:wat::core::quote` argument is data; resolver must not descend.
///
/// A top-level `do` contains a `quote` form whose argument contains
/// `(:my::probe-f2::ghost-quoted arg)` — an inner form whose call head is NOT
/// registered. This form is quoted data.
///
/// Before Gap F-2: resolver recurses into `quote` argument, finds
/// `:my::probe-f2::ghost-quoted` as a call head, emits `UnresolvedReference`.
/// After Gap F-2: resolver skips `quote` argument; startup succeeds.
#[test]
fn probe_quote_argument_is_data() {
    let src = r#"
        (:wat::core::do
          (:wat::core::define (:my::probe-f2::quote-helper -> :wat::core::nil)
            :wat::core::nil)
          (:wat::core::quote
            (:my::probe-f2::ghost-quoted deeply-nested-arg)))
    "#;
    // The inner :my::probe-f2::ghost-quoted is inside quote — resolver must
    // treat the entire argument as data and not flag the inner call head.
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup must succeed: quote argument is data, not a live call head");
}

/// Probe 3 — `:wat::core::quasiquote` template: data positions not descended,
/// unquote positions ARE descended and validated.
///
/// A top-level `do` contains:
///   1. `:my::probe-f2::live-fn` — a real registered define.
///   2. A `quasiquote` template containing:
///      - `(:my::probe-f2::ghost-template arg)` — a form whose call head is
///        NOT registered. Inside quasiquote template data → must NOT be flagged.
///      - `(:wat::core::unquote (:my::probe-f2::live-fn))` — an unquote escape.
///        `:my::probe-f2::live-fn` IS registered → must resolve correctly.
///
/// Before Gap F-2: resolver descends into the template unconditionally, flags
/// `:my::probe-f2::ghost-template` as unresolved.
/// After Gap F-2: resolver skips template data positions; descends only into
/// unquote children; `:my::probe-f2::live-fn` resolves; startup succeeds.
#[test]
fn probe_quasiquote_unquote_resolves_correctly() {
    let src = r#"
        (:wat::core::do
          (:wat::core::define (:my::probe-f2::live-fn -> :wat::core::nil)
            :wat::core::nil)
          (:wat::core::quasiquote
            (:my::probe-f2::ghost-template
              (:wat::core::unquote (:my::probe-f2::live-fn)))))
    "#;
    // :my::probe-f2::ghost-template is quasiquote template data → no error.
    // (:my::probe-f2::live-fn) inside unquote IS live code → must be registered.
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup must succeed: quasiquote template data not flagged; unquote content resolves");
}
