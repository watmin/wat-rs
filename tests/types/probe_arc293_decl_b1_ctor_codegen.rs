//! RED probe — arc 293 decl-b.1: the bare `:T` ctor is codegen'd for EVERY holder.
//!
//! Today `register_struct_methods` (runtime.rs:924) codegens the ctor only for structs
//! (`holder == Struct`); record + holon ctors are emitted by the `defrecord`/`holon::defrecord`
//! MACROS (a `defn`) — which is exactly why those two macros carry the duplicated `syms`-
//! extraction dance. decl-b.1 extends ctor codegen to record + holon (body `aggregate-new`,
//! like structs), so the macros can drop the ctor `defn` and the duplication dies.
//!
//! RED at HEAD: a record declared via the RAW `recordtype` primitive (no macro) has accessors
//! (R2.2 codegens those for all holders) but NO ctor — `(:test::db::BR 7 8)` is unresolved.
//! GREEN after decl-b.1: the ctor is codegen'd → construction works for raw-recordtype records.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// A base record via raw `recordtype` constructs via its codegen'd ctor; field a = 7.
#[test]
fn raw_recordtype_record_has_codegen_ctor() {
    let world = startup_beside(file!()).expect("startup must succeed (record ctor is codegen'd)");
    let ast = wat::parse_one!("(:user::db-br-a)").expect("parse db-br-a call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new()).expect("eval db-br-a");
    match tv.value_owned() {
        Value::i64(7) => {}
        other => panic!("raw recordtype record ctor: expected i64(7), got {:?}", other),
    }
}

/// A holon record via raw `recordtype` constructs via its codegen'd ctor; field a = 7.
#[test]
fn raw_recordtype_holon_has_codegen_ctor() {
    let world = startup_beside(file!()).expect("startup must succeed (holon ctor is codegen'd)");
    let ast = wat::parse_one!("(:user::db-hr-a)").expect("parse db-hr-a call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new()).expect("eval db-hr-a");
    match tv.value_owned() {
        Value::i64(7) => {}
        other => panic!("raw recordtype holon ctor: expected i64(7), got {:?}", other),
    }
}

/// A holon record built via the RAW primitive must carry a hologram (cosine(h,h)==1.0).
/// RED at HEAD: the fallback uses `:wat::Record::of` (base ctor) → no hologram. GREEN after
/// decl-b.1 routes the fallback through `aggregate-new` (gated on decl-b.1.0 — `aggregate-new`
/// must first handle inherited fields). #[ignore]'d STRIKE-READY until then.
#[test]
#[ignore = "RED until decl-b.1.0 (aggregate-new inheritance) + decl-b.1 (fallback→aggregate-new) land"]
fn raw_recordtype_holon_has_a_hologram() {
    let world = startup_beside(file!()).expect("startup must succeed");
    let ast = wat::parse_one!("(:user::db-hr-cos)").expect("parse db-hr-cos call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new()).expect("eval db-hr-cos");
    match tv.value_owned() {
        Value::f64(c) => assert!(
            (c - 1.0).abs() < 1e-6,
            "raw holon recordtype must carry a hologram: cosine(h,h) must be 1.0; got {}",
            c
        ),
        other => panic!("raw holon cosine: expected f64, got {:?}", other),
    }
}
