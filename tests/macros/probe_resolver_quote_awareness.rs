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
//! All three probes FAIL before Gap F-2 ships; all three PASS after.

use wat::freeze::startup_from_file;

/// Probe 1 — `:wat::core::forms` arguments are data; resolver must not descend.
#[test]
fn probe_forms_argument_is_data() {
    startup_from_file("tests/macros/probe_resolver_quote_awareness_forms_data.wat")
        .expect("startup must succeed: forms arguments are data, not live call heads");
}

/// Probe 2 — `:wat::core::quote` argument is data; resolver must not descend.
#[test]
fn probe_quote_argument_is_data() {
    startup_from_file("tests/macros/probe_resolver_quote_awareness_quote_data.wat")
        .expect("startup must succeed: quote argument is data, not a live call head");
}

/// Probe 3 — `:wat::core::quasiquote` template: data positions not descended,
/// unquote positions ARE descended and validated.
#[test]
fn probe_quasiquote_unquote_resolves_correctly() {
    startup_from_file("tests/macros/probe_resolver_quote_awareness_quasiquote.wat")
        .expect("startup must succeed: quasiquote template data not flagged; unquote content resolves");
}
