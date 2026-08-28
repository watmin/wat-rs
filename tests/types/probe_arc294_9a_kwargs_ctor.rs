//! Acceptance probe — arc 294 item 9a: the ctor codegen FLIP.
//!
//! The bare aggregate type name (`:ns::T`) is now a KWARGS macro (order-free
//! `(:ns::T :field val …)`); the raw positional ctor moved to the type-name PRIME
//! (`:ns::T'`). `register_aggregate_methods` (runtime.rs) mints the positional ctor at
//! the prime; `:wat::core::defrecord` / `:wat::holon::defrecord` / `:wat::core::defstruct`
//! (Record.wat / core.wat) each emit a companion `defmacro` at the bare name that
//! thin-forwards to `:wat::core::kwargs-lower` in pure-positional mode.
//!
//! This probe passes even while the overall floor is red (Phase A does NOT migrate
//! call-sites — a pre-existing bare-positional call inside the stdlib itself,
//! `wat/sqlite.wat`'s `:wat::sqlite'::Fault` construction in `classify`, blocks the
//! shared baked-stdlib startup path used by `startup_from_file`; see the arc-294-9a
//! shadowdancer report for the full cascade worklist). The FOUR assertions below are
//! therefore encoded as a single `.wat` fixture (co-located) whose forms this file
//! exercises directly against a scoped world — no dependency on `wat/sqlite.wat`.

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

/// The fixture loads clean: bare kwargs (either key order) reorders to the prime
/// positional call, and the prime itself accepts raw positional args.
#[test]
fn kwargs_ctor_fixture_checks_clean() {
    let r = startup_from_file("tests/types/probe_arc294_9a_kwargs_ctor.wat");
    assert!(
        r.is_ok(),
        "arc294 9a: fresh defrecord + bare kwargs (both key orders) + prime positional \
         must all check clean; got {:?}",
        r.err()
    );
}

/// Bare-positional construction is a LOCATED error — the bare name is no longer the
/// positional ctor once the flip lands.
#[test]
fn bare_positional_construction_is_rejected() {
    let r = startup_from_file("tests/types/probe_arc294_9a_kwargs_ctor_bad.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::kwargs-construct"
            && reason == "bare-positional construction of :probe294abad::Pair is retired (the \
                           bare name is the kwargs macro); write kwargs \
                           `(:probe294abad::Pair :field value …)` or use the positional prime \
                           `:probe294abad::Pair'`"
    );
}
