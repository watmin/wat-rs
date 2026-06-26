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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// A recordtype with a dotted field `wat.started-at : Instant`, constructed and read back as
/// epoch-millis (i64) so we can assert a concrete value end-to-end.
fn eval_dotted_field_roundtrip() -> Result<i64, String> {
    let src = "\
        (:wat::core::defrecord :user::Probe [wat.started-at <- :wat::time::Instant])\n\
        (:wat::core::defn :user::compute [] -> :wat::core::i64\n\
          (:wat::time::epoch-millis\n\
            (:user::Probe/wat.started-at (:user::Probe (:wat::time::at-millis 1234)))))\n\
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

#[test]
fn c01_dotted_record_field_roundtrips() {
    // Construct a Probe with Instant=at-millis(1234), read wat.started-at back, → 1234.
    assert_eq!(eval_dotted_field_roundtrip(), Ok(1234),
        "a dotted recordtype field `wat.started-at` should declare, construct, and access cleanly");
}
