//! PROBE — arc 296 stone H-2: a variant is a tagged MAP, and its tag cannot be a record's.
//!
//! Two claims, both RED at HEAD, both measured 2026-09-06:
//!
//!   record  `:usr::Shape::Circle`            -> #usr.Shape/Circle {:r 2}
//!   enum    `:usr::Shape` variant `:Circle`  -> #usr.Shape/Circle [2]
//!            ^ BYTE-IDENTICAL TAGS, from the identical `tag_from_type_path` call.
//!
//! Today the ONLY thing separating them is body shape, which H's design calls out: the current
//! design "does not HAVE a discriminator in the tag — it has an ambiguity that the body happens to
//! mask." H-2 moves the classification into the tag (`#usr/Shape.Circle`) where the dot means
//! variant, and gives the variant the same map body every other named datum already has. The wall
//! that makes the dot real is already live (`src/resolve/registration.rs` `DottedName`, H-1).
//!
//! `EnumValue.names` is ALREADY carried (arc 296 G′ — "the enum mirror of AggregateValue.names",
//! same length as `fields`, always), so the map body's keys need no registry lookup and no
//! `field-N` fallback. That half of H's design is done.
//!
//! ⚠ THE PROGRAMS ARE SEPARATE FILES ON PURPOSE. Declared together, the record silently STEALS the
//! variant's constructor in either order (NOTE-a-record-silently-steals-a-variants-constructor.md)
//! — so a single-program probe would measure the theft, not the tag. That defect is NOT H-2's to
//! close and deliberately has no row here.
//!
//! RED at HEAD; `#[ignore]`d until H-2 un-ignores them.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn stdout_of(fixture: &str) -> String {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(fixture);
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    assert!(
        out.status.success(),
        "{fixture} must run clean; stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// Split a rendered tagged value into `(tag, body)`. Both halves are exact — no substring
/// checks, because a loose assertion passes on reordered fields and appended garbage.
fn split_tagged(rendered: &str) -> (&str, &str) {
    rendered
        .split_once(' ')
        .unwrap_or_else(|| panic!("a rendered tagged value is `<tag> <body>`; got {rendered:?}"))
}

#[test]
#[ignore = "RED at HEAD — arc 296 stone H-2 (a variant is a tagged map); un-ignored BY that stone"]
fn a_variant_tag_is_not_a_records_tag() {
    let variant = stdout_of("probe_arc296_h2__variant.wat");
    let record = stdout_of("probe_arc296_h2__record.wat");
    assert_ne!(
        split_tagged(&variant).0,
        split_tagged(&record).0,
        "a record and an enum variant render the SAME tag — the tag carries no discriminator, \
         only the body shape does (variant={variant:?} record={record:?})"
    );
}

#[test]
#[ignore = "RED at HEAD — arc 296 stone H-2 (a variant is a tagged map); un-ignored BY that stone"]
fn a_variant_body_is_rendered_exactly_like_a_records() {
    let variant = stdout_of("probe_arc296_h2__variant.wat");
    let record = stdout_of("probe_arc296_h2__record.wat");
    // The two fixtures declare the SAME single field `r` with the SAME value, one as a record
    // field and one as a variant payload. H's thesis is "one rule for named data, both kinds,
    // both directions" — so the bodies must be byte-identical and only the tags may differ.
    // The record's body is the CONTROL: the expected text is derived from it, never written
    // out by hand, so this row cannot drift from what the writer actually does for named data.
    assert_eq!(
        split_tagged(&variant).1,
        split_tagged(&record).1,
        "the variant body must be the map a record already gets ({record:?}); today it is a \
         positional vector ({variant:?}) — the one place named data is still thrown away"
    );
}
