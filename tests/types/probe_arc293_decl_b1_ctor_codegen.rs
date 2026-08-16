//! RED probe — arc 293 decl-b.1: the bare `:T` ctor is codegen'd for EVERY nature.
//!
//! Today `register_struct_methods` (runtime.rs:924) codegens the ctor only for structs
//! (`nature == Struct`); record + holon ctors are emitted by the `defrecord`/`holon::defrecord`
//! MACROS (a `defn`) — which is exactly why those two macros carry the duplicated `syms`-
//! extraction dance. decl-b.1 extends ctor codegen to record + holon (body `aggregate-new`,
//! like structs), so the macros can drop the ctor `defn` and the duplication dies.
//!
//! RED at HEAD: a record declared via the RAW `recordtype` primitive (no macro) has accessors
//! (R2.2 codegens those for all natures) but NO ctor — `(:test::db::BR 7 8)` is unresolved.
//! GREEN after decl-b.1: the ctor is codegen'd → construction works for raw-recordtype records.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// A base record via raw `recordtype` constructs via its codegen'd ctor; field a = 7.
#[test]
fn raw_recordtype_record_has_codegen_ctor() {
    let got = call_beside_value(file!(), ":user::db-br-a").expect("eval db-br-a");
    match got {
        Value::i64(7) => {}
        other => panic!("raw recordtype record ctor: expected i64(7), got {:?}", other),
    }
}

/// A holon record via raw `recordtype` constructs via its codegen'd ctor; field a = 7.
#[test]
fn raw_recordtype_holon_has_codegen_ctor() {
    let got = call_beside_value(file!(), ":user::db-hr-a").expect("eval db-hr-a");
    match got {
        Value::i64(7) => {}
        other => panic!("raw recordtype holon ctor: expected i64(7), got {:?}", other),
    }
}

/// A holon record built via the RAW primitive must carry a hologram (cosine(h,h)==1.0).
/// RED at HEAD: the fallback uses `:wat::core::Record::of` (base ctor) → no hologram. GREEN after
/// decl-b.1 routes the fallback through `aggregate-new` (gated on decl-b.1.0 — `aggregate-new`
/// must first handle inherited fields).
///
/// ⚠ UN-IGNORED 2026-08-16 — AND ITS STATED UNLOCK NEVER LANDED. `decl-b.1.0` (aggregate-new
/// inheritance) was **ANNIHILATED**, not built: `19ace45e` — *"inheritance ANNIHILATION is next
/// (decl-b.1.0 deleted)"*. So this is green by a DIFFERENT route than the one the old reason
/// named, and the doc paragraph above it is history, not a live prediction. The test itself is
/// sound and non-vacuous: it evaluates `:user::db-hr-cos` and asserts `cosine(h,h) == 1.0` on a
/// real `CosineOutcome::Similarity`, which a record without a hologram cannot produce. Only the
/// ignore's REASON was stale — a superseded unlock, the third disposition
/// (staleness / finding / SUPERSEDED).
#[test]
fn raw_recordtype_holon_has_a_hologram() {
    // Arc 278 the cosine outcome wall — cosine now returns
    // :wat::holon::CosineOutcome, not a bare f64; extract the Similarity
    // variant's field (cosine(h,h) on a real hologram is never Degenerate/
    // DimensionMismatch).
    let got = call_beside_value(file!(), ":user::db-hr-cos").expect("eval db-hr-cos");
    match got {
        Value::Enum(ev) if ev.type_path == ":wat::holon::CosineOutcome" => {
            match (ev.variant_name.as_str(), ev.fields.as_slice()) {
                ("Similarity", [Value::f64(c)]) => assert!(
                    (c - 1.0).abs() < 1e-6,
                    "raw holon recordtype must carry a hologram: cosine(h,h) must be 1.0; got {}",
                    c
                ),
                other => panic!("raw holon cosine: expected CosineOutcome::Similarity[f64], got {:?}", other),
            }
        }
        other => panic!("raw holon cosine: expected CosineOutcome, got {:?}", other),
    }
}
