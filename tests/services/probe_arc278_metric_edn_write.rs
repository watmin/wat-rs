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
    assert!(
        s.contains("Metric"),
        "encoded EDN should carry the Metric type tag; got: {s}"
    );
    assert!(
        s.contains("probe-ns") && s.contains("requests"),
        "encoded EDN should carry the Metric's field values (namespace + name); got: {s}"
    );
}
