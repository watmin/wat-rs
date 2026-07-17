//! arc 278 T1b.2 groundwork — KNOWN-UNKNOWN #2: can a live `Metric` record be serialized to its
//! tagged-EDN string via `:wat::edn::write` (arc-300 records-are-EDN)? journal''s write path needs
//! exactly this to fill `StoredRow.data`. If `:wat::edn::write` were fenced to `:wat::edn::Tagged`
//! rather than accepting an arbitrary record, this would fail to type-check / freeze — and the
//! serialize step would be a substrate stone BEFORE journal', not a one-liner inside it.
//!
//! Asserts the encoded string carries the Metric's tag + field values (proves a real tagged-EDN
//! render, not an opaque/empty result).

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn a_metric_record_serializes_to_tagged_edn_via_edn_write() {
    let world = startup_beside(file!()).expect("startup should succeed (telemetry baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!(":wat::edn::write on a Metric raised: {e:?}"));
    let s = match got {
        Value::String(ref s) => s.as_str().to_string(),
        other => panic!("expected a String from :wat::edn::write; got {other:?}"),
    };
    // A tagged-EDN render of the Metric must name the type and carry its field values.
    // Compare against the co-located golden .edn. assert_edn_eq! parses BOTH sides as EDN and
    // compares structurally — so this also proves the render is well-formed EDN (a malformed
    // "dangling nil" would fail parse_owned with STOP-1). The trailing `#…/Count nil` is the
    // fieldless enum variant's valid tagged-nil payload, not a dangling map element.
    wat::assert_edn_eq!(
        s,
        include_str!("probe_arc278_metric_edn_write__metric.edn"),
        "Metric serializes to valid, deterministic tagged EDN"
    );
}
