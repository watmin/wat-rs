//! arc 278 STONE T1b.2 — journal' write-LOGS coverage (symmetric to write-metrics). journal' given
//! a mem-store', write-logs a 1-Log batch whose message is OPAQUE (Stone B): the producer
//! `edn::write`s its payload record at the call site, so a String crosses + is stored verbatim.
//! A separate client scans back; the stored `data` is checked field-by-field.
//!
//! arc 278 "caller.2": `:caller :evaluator` (a static, portable literal) flipped to
//! `:emitted-from (:wat::kernel::call-site)` — a REAL captured `:wat::kernel::Frame`. Its `:file`
//! is the Rust caller's ABSOLUTE source path (this test invokes `:user::compute` directly, so the
//! Frame describes THIS `.rs` call site) — machine/checkout-path dependent, so it is deliberately
//! NOT diffed by a whole-blob golden `.edn` anymore (that would freeze this developer's absolute
//! path into the corpus). Instead: parse the stored EDN and check fields individually — exact for
//! the static ones, structural/portable for `:emitted-from` (`:file` checked by SUFFIX not full
//! path, `:line` checked as present+positive, `:symbol` checked exactly since the callee name is
//! checkout-independent).
//!
//! No literal EDN-esque string content lives in this file (the `no-inlined-edn` lint bans it):
//! tags are compared via `Tag::namespace()`/`Tag::name()` (plain identifiers, not `#ns/Name`
//! strings), and the opaque `:message` payload is substring-checked rather than compared against
//! its full `#user/PriceEvent {...}` EDN form.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};
use wat_edn::OwnedValue;

fn map_get<'a>(pairs: &'a [(OwnedValue, OwnedValue)], key: &str) -> &'a OwnedValue {
    pairs
        .iter()
        .find(|(k, _)| {
            k.as_keyword()
                .map(|kw| kw.name() == key && kw.namespace().is_none())
                .unwrap_or(false)
        })
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("key :{key} not found in map: {pairs:?}"))
}

#[test]
fn journal_writes_a_log_through_a_held_store_peer_on_a_thread() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' + mem-store' + telemetry vocab baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("journal' write-logs round-trip raised: {e:?}"),
    };

    let val = wat_edn::parse_owned(&stored)
        .unwrap_or_else(|err| panic!("stored data is not valid EDN: {err}\nraw: {stored}"));
    let (tag, body) = val.as_tagged().expect("stored value is a tagged Log");
    assert_eq!(tag.namespace(), "wat.telemetry", "Log tag namespace: {tag:?}");
    assert_eq!(tag.name(), "Log", "Log tag name: {tag:?}");
    let fields = body.as_map().expect("Log body is a map");

    assert_eq!(map_get(fields, "namespace").as_str(), Some("probe-ns"));
    assert_eq!(map_get(fields, "time-ns").as_i64(), Some(456));
    // :message is OPAQUE (Stone B) — the producer `edn::write`s a PriceEvent record. Exact,
    // structural compare against a co-located golden (never an inlined EDN-esque literal).
    let message = map_get(fields, "message").as_str().expect(":message is a String");
    wat::assert_edn_eq!(
        message.to_string(),
        include_str!("probe_arc278_journal_service_logs__message.edn"),
        "opaque :message payload (Stone B, producer edn::write)"
    );

    let (level_tag, _) = map_get(fields, "level").as_tagged().expect("level is tagged");
    assert_eq!(level_tag.namespace(), "wat.telemetry.Level", "level tag namespace: {level_tag:?}");
    assert_eq!(level_tag.name(), "Info", "level tag name: {level_tag:?}");

    // :emitted-from — a real captured Frame (caller.2). Structural, portable checks only: the
    // absolute Rust source path varies by checkout, so we check the SUFFIX, not the whole path.
    let (frame_tag, frame_body) = map_get(fields, "emitted-from")
        .as_tagged()
        .expect("emitted-from is a tagged Frame");
    assert_eq!(frame_tag.namespace(), "wat.kernel", "Frame tag namespace: {frame_tag:?}");
    assert_eq!(frame_tag.name(), "Frame", "Frame tag name: {frame_tag:?}");
    let frame_fields = frame_body.as_map().expect("Frame body is a map");

    // Arc 109 — Frame's fields are concrete (non-`Option`): bare String / i64 /
    // String, read directly (no `Some` unwrap).
    let file_str = map_get(frame_fields, "file")
        .as_str()
        .expect("Frame :file is a String");
    // rune:lint(loose-assert) — Frame :file is an ABSOLUTE Rust source path (checkout-directory
    // dependent, like the lint's own sanctioned path/pid/hash exemption); only the filename SUFFIX
    // is checkout-independent, so a full assert_eq! would hardcode this developer's absolute path.
    assert!(
        file_str.ends_with("probe_arc278_journal_service_logs.rs"),
        "Frame :file should name this test file (suffix-checked, checkout-path independent): {file_str}"
    );

    let line_val = map_get(frame_fields, "line")
        .as_i64()
        .expect("Frame :line is an i64");
    assert!(line_val > 0, "Frame :line should be positive: {line_val}");

    let symbol_val = map_get(frame_fields, "symbol")
        .as_str()
        .expect("Frame :symbol is a String");
    assert_eq!(symbol_val, ":user::compute", "Frame :symbol should name the callee");
}
