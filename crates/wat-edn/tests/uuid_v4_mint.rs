//! Arc 092 — `wat-edn` ships `new_uuid_v4()` behind the `mint`
//! feature. These tests prove the contract:
//!
//!   1. A v4 UUID minted by wat-edn round-trips through its own
//!      `write` + `parse` cycle without loss.
//!   2. Repeated mints produce distinct values (smoke screen against
//!      a constant return; not a formal entropy proof).
//!
//! The whole file is `#[cfg(feature = "mint")]` — `cargo test`
//! without features compiles wat-edn without the v4 dep, and these
//! tests sit out the run. Both lanes must stay green; the gate has
//! to actually gate.

#![cfg(feature = "mint")]

use std::collections::HashSet;
use wat_edn::{new_uuid_v4, parse, write, Value};

/// A freshly-minted v4 UUID survives wat-edn's own write/parse
/// cycle. If a future change to the writer drops a hex digit or
/// miscanonicalizes the form, this fails before users do.
#[test]
fn v4_roundtrips_through_write_and_parse() {
    let original = new_uuid_v4();
    let edn = write(&Value::Uuid(original));
    let parsed = parse(&edn).expect("EDN must parse");
    assert_eq!(
        parsed.as_uuid(),
        Some(&original),
        "v4 UUID must survive write+parse roundtrip; got {edn:?}"
    );
}

/// Many mints produce many distinct UUIDs. The v4 RFC mandates ≥122
/// bits of entropy; in practice we expect zero collisions across
/// a few hundred mints. We assert 256 distinct as a sanity smoke
/// screen against accidental constant returns or seed-stuck RNGs —
/// not a formal entropy proof.
#[test]
fn many_v4_mints_are_unique() {
    const N: usize = 256;
    let set: HashSet<_> = (0..N).map(|_| new_uuid_v4()).collect();
    assert_eq!(
        set.len(),
        N,
        "256 mints must produce 256 distinct UUIDs (saw {} collisions)",
        N - set.len()
    );
}

/// The minted UUID's canonical-hyphenated string form must match the
/// regex EDN's `#uuid` parser accepts: 8-4-4-4-12 hex, lowercase,
/// hyphenated. Belt-and-suspenders against a future Uuid::to_string
/// representation drift.
#[test]
fn v4_string_form_is_canonical_hyphenated() {
    let id = new_uuid_v4();
    let s = id.to_string();
    let parts: Vec<&str> = s.split('-').collect();
    assert_eq!(parts.len(), 5, "must have 4 hyphens; got {s:?}");
    assert_eq!(parts[0].len(), 8, "first group must be 8 hex chars");
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
    assert!(
        s.chars().all(|c| c == '-' || c.is_ascii_hexdigit()),
        "must be hex digits + hyphens only; got {s:?}"
    );
    // EDN's #uuid form quotes the canonical string body.
    let edn = format!("#uuid \"{s}\"");
    let parsed = parse(&edn).expect("canonical #uuid form must parse");
    assert_eq!(parsed.as_uuid(), Some(&id));
}
