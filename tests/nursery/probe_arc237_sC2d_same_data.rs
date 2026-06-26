//! FM 2-bis probe — arc 237 Stone S-C.2d: `:wat::Record/same-data?` (type-BLIND record data equality).
//!
//! `same-data?` compares the field DATA of two records, ignoring class (type) and flavor — the
//! user's 2×2 grid + cross-type case. Distinct from `=` (type-strict, arc 238).
//!
//! DESIGN HYPOTHESIS (grounded here): now that arc 238 made `=` compare maps, `same-data?` is just
//!   `(= (record->map a) (record->map b))` — name-keyed (a record's data IS its named fields),
//!   type-blind + flavor-blind (record->map drops the class; works on either flavor).
//!
//! Two test groups:
//!   - `comp_*` — the COMPOSITION directly (`= (record->map …) (record->map …)`). GREEN NOW
//!     (proves the impl path is sound post-arc-238). If these fail, the design is wrong.
//!   - `samedata_*` — the verb `:wat::Record/same-data?` itself. RED NOW (verb absent →
//!     startup error); GREEN once the stone ships the defn. These are the load-bearing contracts.
//!
//! Two holonic record types with the SAME field names (`[x y]`) — so name-keyed comparison
//! returns true across the two types when values match (type-blind), the whole point.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PRELUDE: &str = "\
(:wat::core::defrecord :my::Pt    [x <- :wat::core::i64  y <- :wat::core::i64])\n\
(:wat::core::defrecord :my::Coord [x <- :wat::core::i64  y <- :wat::core::i64])\n";

fn eq(expr: &str) -> bool {
    let full = format!(
        "{PRELUDE}\
         (:wat::core::defn :user::compute [] -> :wat::core::bool {expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup/check error for `{}`: {:?}", expr, e));
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).map(|tv| tv.value_owned()) {
        Ok(Value::bool(b)) => b,
        other => panic!("expected bool for `{}`; got {:?}", expr, other),
    }
}

// ─── COMPOSITION (GREEN NOW — grounds the design: `= (record->map a) (record->map b)`) ───
#[test]
fn comp_same_type_equal() {
    assert!(eq("(:wat::core::= (:wat::core::record->map (:my::Pt 0 0)) \
                              (:wat::core::record->map (:my::Pt 0 0)))"));
}
#[test]
fn comp_cross_type_equal() {
    // type-BLIND: Pt and Coord, both {x:0,y:0} → equal data
    assert!(eq("(:wat::core::= (:wat::core::record->map (:my::Pt 0 0)) \
                              (:wat::core::record->map (:my::Coord 0 0)))"));
}
#[test]
fn comp_diff_value() {
    assert!(!eq("(:wat::core::= (:wat::core::record->map (:my::Pt 0 0)) \
                               (:wat::core::record->map (:my::Pt 0 9)))"));
}

// ─── THE VERB (RED NOW — GREEN after the stone ships `:wat::Record/same-data?`) ──────────
#[test]
fn samedata_same_type_equal() {
    assert!(eq("(:wat::Record/same-data? (:my::Pt 0 0) (:my::Pt 0 0))"));
}
#[test]
fn samedata_cross_type_equal() {
    assert!(eq("(:wat::Record/same-data? (:my::Pt 0 0) (:my::Coord 0 0))"));
}
#[test]
fn samedata_diff_value() {
    assert!(!eq("(:wat::Record/same-data? (:my::Pt 0 0) (:my::Pt 0 9))"));
}
