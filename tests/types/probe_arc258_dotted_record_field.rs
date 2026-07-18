//! A2 de-risk probe — does a recordtype tolerate a DOTTED field name?
//!
//! program::Env's first system field is `wat.started-at` (the `wat.` reserved dot-prefix). As a
//! recordtype field, its auto-generated accessor is `:<class>/wat.started-at` — a keyword carrying
//! `::` (namespace), `/` (field separator), AND a `.` in the field part. No existing record has a
//! dotted field, so before A2 replaces the program::Env typealias with a recordtype carrying this
//! field, prove the machinery (declare + construct + access) handles it.
//!
//! This is a FORWARD proof on a *user* recordtype (independent of program::Env) — it may be GREEN
//! at HEAD (dotted fields already work → A2's field is viable) or RED (dotted fields need handling
//! → a finding A2 must address). Either outcome de-risks A2.
//!
//! Run: `cargo test --release --test probe_arc258_dotted_record_field`

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn c01_dotted_record_field_roundtrips() {
    let got = call_beside(file!(), ":user::compute").unwrap_or_else(|e| panic!("eval: {e:?}"));
    // Construct a Probe with Instant=at-millis(1234), read wat.started-at back, → 1234.
    assert!(
        matches!(got, Value::i64(1234)),
        "a dotted recordtype field `wat.started-at` should declare, construct, and access cleanly; got: {got:?}"
    );
}
