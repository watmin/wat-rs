//! Arc 109 — BRIEF-STONE-the-peers-bijection-keeps-its-negative-controls: makes the `:peers`
//! bijection's five hand-run destinations (`BRIEF-STONE-defservice-compares-types-as-data.md` row
//! 4, and its "non-vacuity control" reasoning) into a permanent test instead of prose that never
//! re-runs. Every fixture below is `probe_arc278_s2s_peer_on_thread.wat` (the base fixture) with
//! ONE clause changed: the `:ephemeral` peer field's spelling (old `Peer<S::Op,S::Reply>` keyword
//! sugar vs. the structural `(:wat::kernel::Peer :- [S::Op S::Reply])` form) and/or the `:peers`
//! declaration (bogus surface, or dropped entirely).
//!
//! Cases 1-2 prove the OLD spelling still round-trips through both bijection checks ("missing" and
//! "extra"). Cases 3-5 prove the NEW `:-` form spelling does too — case 3 is the positive control
//! (without it, 1/2/4/5 failing proves nothing about the accept path); case 5 is the stone: it is
//! the only case that fails on the *ephemeral* side rather than the `:peers` side, so only its
//! diagnostic can tell "the structural reader extracted `probe::Echo` out of the `:-` form" from
//! "the reader silently returned an empty list" — cases 1, 2 and 4 would read identically either
//! way, since they all fail on the `:peers` side.
//!
//! Driver: `startup_from_file` (not `--check`) — matches `tests/macros/probe_arc279_format.rs`,
//! the exemplar this stone copies, and yields the `Err` value the golden compares against; both
//! reach the same `defservice` macro expansion `--check` does.

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};

// ── Case 1: old keyword spelling, :peers names a surface with no matching ephemeral field ──────
#[test]
fn peers_bijection_old_spelling_missing_ephemeral_is_rejected() {
    let r = startup_from_file("tests/services/probe_arc278_peers_bijection_case1_old_missing.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "probe::caller: :peers declares surface :probe::Bogus but no \
                        :ephemeral field is typed :wat::kernel::Peer<probe::Bogus::Op,…::Reply> \
                        — add the dialed peer as a root :ephemeral field, or drop it from :peers"
            )
    );
    let msg = format!("{:?}", r.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc278_peers_bijection__case1_old_spelling_missing_ephemeral_is_rejected.edn",
        "old spelling, missing ephemeral peer field must match the bijection's \"missing\" diagnostic"
    );
}

// ── Case 2: old keyword spelling, :peers dropped while an ephemeral peer field remains ─────────
#[test]
fn peers_bijection_old_spelling_undeclared_peer_is_rejected() {
    let r = startup_from_file("tests/services/probe_arc278_peers_bijection_case2_old_extra.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "probe::caller: :ephemeral holds a dialed \
                        Peer<probe::Echo::Op,…::Reply> but surface :probe::Echo is not declared \
                        in :peers — add :peers [… :probe::Echo …] (the explicit s2s dependency \
                        DAG)"
            )
    );
    let msg = format!("{:?}", r.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc278_peers_bijection__case2_old_spelling_undeclared_peer_is_rejected.edn",
        "old spelling, undeclared ephemeral peer must match the bijection's \"extra\" diagnostic"
    );
}

// ── Case 3: `:-` form spelling, :peers matches — the POSITIVE control ──────────────────────────
//
// Without this case, cases 1/2/4/5 all failing would be equally consistent with "the `:-` form
// spelling is rejected outright" as with "the bijection correctly rejects a mismatch". This is
// what proves the form spelling is accepted on the happy path.
#[test]
fn peers_bijection_form_spelling_matching_peer_is_accepted() {
    let r = startup_from_file("tests/services/probe_arc278_peers_bijection_case3_form_ok.wat")
        .map(|_| ())
        .map_err(|e| format!("{e:?}"));
    assert!(
        r.is_ok(),
        "the `:-` form spelling `(:wat::kernel::Peer :- [S::Op S::Reply])` with a matching :peers \
         entry must be accepted by the bijection; got {r:?}"
    );
}

// ── Case 4: `:-` form spelling, :peers names a surface with no matching ephemeral field ────────
#[test]
fn peers_bijection_form_spelling_missing_ephemeral_is_rejected() {
    let r =
        startup_from_file("tests/services/probe_arc278_peers_bijection_case4_form_missing.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "probe::caller: :peers declares surface :probe::Bogus but no \
                        :ephemeral field is typed :wat::kernel::Peer<probe::Bogus::Op,…::Reply> \
                        — add the dialed peer as a root :ephemeral field, or drop it from :peers"
            )
    );
    let msg = format!("{:?}", r.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc278_peers_bijection__case4_form_spelling_missing_ephemeral_is_rejected.edn",
        "form spelling, missing ephemeral peer field must match the bijection's \"missing\" diagnostic"
    );
}

// ── Case 5: `:-` form spelling, :peers dropped — THE STONE ─────────────────────────────────────
//
// The non-vacuity control: this is the only case that fails on the *ephemeral* side rather than
// the `:peers` side. If the structural reader that extracts a surface name out of a `:-`-form
// `Peer` type silently returned nothing instead of `"probe::Echo"`, cases 1, 2 and 4 would still
// fail with the same messages — they never exercise that reader. Only this diagnostic can prove
// the reader actually pulled `probe::Echo` out of the form spelling, so the assertion below checks
// the diagnostic literally names `probe::Echo`, not merely that startup raised an error.
#[test]
fn peers_bijection_form_spelling_undeclared_peer_names_the_surface() {
    let r = startup_from_file("tests/services/probe_arc278_peers_bijection_case5_form_extra.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "probe::caller: :ephemeral holds a dialed \
                        Peer<probe::Echo::Op,…::Reply> but surface :probe::Echo is not declared \
                        in :peers — add :peers [… :probe::Echo …] (the explicit s2s dependency \
                        DAG)"
            )
    );
    let msg = format!("{:?}", r.unwrap_err());
    // rune:lint(loose-assert) — a targeted PRESENCE over a large structured output, and the one
    // assertion in this file that `UPDATE_EDN=1` cannot rewrite. The golden below is CAPTURED, not
    // authored: it records whatever the macro emitted at capture time. Had the structural reader
    // returned an empty surface instead of `probe::Echo`, the capture would have recorded THAT
    // message and this test would pass forever, green, while asserting nothing about the property
    // its own name claims. Tightening this to `assert_eq!` on the whole diagnostic is not the
    // alternative — the golden already does the structure-exact compare; this line states WHICH
    // substring of it is load-bearing, so a regression fails at a named assertion instead of as an
    // opaque EDN diff. Deleting it would satisfy the lint and delete the only human-stated property
    // in the stone.
    assert!(
        msg.contains("probe::Echo"),
        "the \"extra\" diagnostic must NAME the surface the structural reader extracted from the \
         `:-` form (probe::Echo) — if the reader silently returned an empty list instead, this \
         message would still exist but would not contain \"probe::Echo\"; got: {msg}"
    );
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc278_peers_bijection__case5_form_spelling_undeclared_peer_names_the_surface.edn",
        "form spelling, undeclared ephemeral peer must match the bijection's \"extra\" diagnostic \
         and must name probe::Echo"
    );
}
