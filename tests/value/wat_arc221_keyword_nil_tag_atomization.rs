//! Arc 221 Stone 221.4 — `keyword`, `nil`, and `Uuid` atomization probes.
//!
//! Verifies that the new `value_to_atom` arms (Stone 221.4) dispatch correctly:
//!   (a) `Value::wat__core__keyword` → `HolonAST::keyword(&k)` (Keyword leaf,
//!       not Symbol). Closes the pre-arc-221 convention where keyword atoms
//!       were silently encoded as `HolonAST::Symbol(":foo")`.
//!   (b) `Value::Unit` (wat's nil) → `HolonAST::Nil` (Nil leaf, not Symbol("nil")).
//!   (c) `Value::wat__core__Uuid` → `HolonAST::Bind(Tag("uuid"), String(hex))`
//!       per arc 221 doctrine correction. Closes arc 207 false-flag (5-day-latent
//!       gap since 2026-05-17; Uuid had no value_to_atom arm until Stone 221.4).
//!
//! ## Tests
//!
//!  1 — `(:wat::holon::to-holon :foo)` round-trip; atom(:foo) ≠ atom("foo") (distinct from String)
//!  2 — `(:wat::holon::to-holon nil)` round-trip; atom(nil) ≠ atom(:nil) (distinct from Keyword)
//!  3 — `(:wat::holon::to-holon<uuid-val>)` round-trip via tagged composition; closes arc 207
//!  4 — `HashMap<keyword, i64>` insert + lookup (keyword as map key)
//!  5 — `HashSet<keyword>` insert + contains? (keyword as set element)
//!  6 — `HashMap<Uuid, String>` insert + lookup (Uuid as map key — arc 207 false-flag close)
//!
//! Wat source lives in the co-located fixture: wat_arc221_keyword_nil_tag_atomization.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_bool(world: &wat::freeze::FrozenWorld, expr: &str) -> bool {
    let ast = wat::parse_one!(expr).expect("parse expr");
    match eval_in_frozen(&ast, world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
    {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn run_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> i64 {
    let ast = wat::parse_one!(expr).expect("parse expr");
    match eval_in_frozen(&ast, world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn run_string(world: &wat::freeze::FrozenWorld, expr: &str) -> String {
    let ast = wat::parse_one!(expr).expect("parse expr");
    match eval_in_frozen(&ast, world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
    {
        Value::String(s) => s.as_str().to_string(),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Probe 1 — `(:wat::holon::to-holon :foo)` round-trip; distinct from String ──

/// `(:wat::holon::to-holon :foo)` dispatches through `value_to_atom` Keyword arm
/// to `HolonAST::keyword("foo")` (a `HolonAST::Keyword` leaf).
///
/// Distinctness: atom(:foo) must NOT equal atom("foo") — Keyword leaf vs String leaf.
/// This proves Keyword is encoded as a first-class primitive, not string-coerced.
///
/// Arc 221 Stone 221.4 — Keyword arm in `value_to_atom` (Stone 221.3 Keyword leaf).
#[test]
fn probe_1_keyword_atom_round_trip_distinct_from_string() {
    let world = startup_beside(file!()).expect("startup");

    // atom(:foo) = atom(:foo) — same keyword produces identical HolonAST.
    let same = run_bool(&world, "(:t::p1-same)");
    assert!(same, "Atom(:foo) must equal Atom(:foo) — same Keyword leaf");

    // atom(:foo) ≠ atom(:bar) — different keywords produce distinct HolonAST.
    let diff = run_bool(&world, "(:t::p1-diff)");
    assert!(diff, "Atom(:foo) must NOT equal Atom(:bar) — distinct Keyword leaves");

    // atom(:foo) ≠ atom("foo") — Keyword leaf is distinct from String leaf.
    let not_string = run_bool(&world, "(:t::p1-not-string)");
    assert!(not_string, "Atom(:foo) must NOT equal Atom(\"foo\") — Keyword leaf distinct from String leaf");
}

// ─── Probe 2 — `(:wat::holon::to-holon nil)` round-trip ──────────────

/// `(:wat::holon::to-holon nil)` dispatches through `value_to_atom` Nil arm
/// to `HolonAST::Nil` (the proper Nil primitive leaf, not Symbol("nil")).
///
/// Arc 221 Stone 221.4 — Nil arm in `value_to_atom` (Stone 221.3 Nil leaf).
#[test]
fn probe_2_nil_atom_round_trip_distinct_from_keyword_nil() {
    let world = startup_beside(file!()).expect("startup");

    // atom(:wat::core::nil) = atom(:wat::core::nil) — same nil produces identical HolonAST::Nil.
    let same = run_bool(&world, "(:t::p2-same)");
    assert!(same, "Atom(:wat::core::nil) must equal itself — same Nil leaf");

    // atom(:wat::core::nil) ≠ atom(:nil) — HolonAST::Nil is distinct from HolonAST::Keyword("nil").
    let diff = run_bool(&world, "(:t::p2-diff)");
    assert!(diff, "Atom(:wat::core::nil) must NOT equal Atom(:nil) — Nil leaf distinct from Keyword leaf");
}

// ─── Probe 3 — `(:wat::holon::to-holon<uuid-val>)` round-trip — closes arc 207 ─

/// `(:wat::holon::to-holon<uuid>)` dispatches through `value_to_atom` Uuid arm to
/// `HolonAST::Bind(Tag("uuid"), String(hex))` — the tagged composition shape.
///
/// Arc 221 Stone 221.4 — Uuid arm in `value_to_atom` (arc 221 doctrine).
#[test]
fn probe_3_uuid_atom_round_trip_closes_arc_207_false_flag() {
    let world = startup_beside(file!()).expect("startup");

    // atom(uuid-v5-same-args) = atom(uuid-v5-same-args) — deterministic Uuid.
    let same = run_bool(&world, "(:t::p3-same)");
    assert!(same, "Atom(Uuid/v5 same-args) must equal itself — deterministic tagged composition");

    // atom(uuid-v5-"hello") ≠ atom(uuid-v5-"world") — different UUIDs produce distinct Bind.
    let diff = run_bool(&world, "(:t::p3-diff)");
    assert!(diff, "Atom(Uuid/v5 \"hello\") must NOT equal Atom(Uuid/v5 \"world\") — distinct tagged compositions");
}

// ─── Probe 4 — `HashMap<keyword, i64>` insert + lookup ────────────────────────

/// `HashMap<keyword, i64>` with keyword keys — tag-frequency-tally pattern.
///
/// Arc 221 Stone 221.4 — Keyword arm in `value_to_atom` enables HashMap<keyword, V>.
#[test]
fn probe_4_hashmap_keyword_key_insert_lookup() {
    let world = startup_beside(file!()).expect("startup");

    // Insert :tag-a → 10, :tag-b → 20; get :tag-a returns Some(10).
    let a_val = run_i64(&world, "(:t::p4-a-val)");
    assert_eq!(a_val, 10, "HashMap<keyword,i64>: get :tag-a after insert must return 10");

    // get :tag-b returns Some(20).
    let b_val = run_i64(&world, "(:t::p4-b-val)");
    assert_eq!(b_val, 20, "HashMap<keyword,i64>: get :tag-b after insert must return 20");

    // length = 2 — two distinct keyword keys.
    let len = run_i64(&world, "(:t::p4-len)");
    assert_eq!(len, 2, "HashMap<keyword,i64>: two distinct keyword keys → length 2");
}

// ─── Probe 5 — `HashSet<keyword>` insert + contains? ─────────────────────────

/// `HashSet<keyword>` — tag-set pattern.
///
/// Arc 221 Stone 221.4 — Keyword arm enables HashSet<keyword>.
#[test]
fn probe_5_hashset_keyword_insert_contains() {
    let world = startup_beside(file!()).expect("startup");

    // :foo is in the set.
    let has_foo = run_bool(&world, "(:t::p5-has-foo)");
    assert!(has_foo, "HashSet<keyword>: :foo must be found in set");

    // :bar is in the set.
    let has_bar = run_bool(&world, "(:t::p5-has-bar)");
    assert!(has_bar, "HashSet<keyword>: :bar must be found in set");

    // :unknown is NOT in the set.
    let no_unknown = run_bool(&world, "(:t::p5-no-unknown)");
    assert!(no_unknown, "HashSet<keyword>: :unknown must NOT be found in set");

    // length = 3 — three distinct keyword elements.
    let len = run_i64(&world, "(:t::p5-len)");
    assert_eq!(len, 3, "HashSet<keyword>: three distinct keywords → length 3");
}

// ─── Probe 6 — `HashMap<Uuid, String>` insert + lookup — closes arc 207 ──────

/// `HashMap<Uuid, String>` with Uuid keys — closes arc 207 false-flag.
///
/// Arc 221 Stone 221.4 — Uuid arm in `value_to_atom` + Bind(Tag, String) shape.
#[test]
fn probe_6_hashmap_uuid_key_insert_lookup_closes_arc_207() {
    let world = startup_beside(file!()).expect("startup");

    // Insert (v5-nil-"hello") → "world-entry"; get same uuid returns Some("world-entry").
    let retrieved = run_string(&world, "(:t::p6-retrieved)");
    assert_eq!(retrieved, "world-entry", "HashMap<Uuid,String>: get after insert must return the inserted value");

    // Different Uuid key returns None (mapped to "NOT-FOUND" sentinel).
    let not_found = run_string(&world, "(:t::p6-not-found)");
    assert_eq!(not_found, "NOT-FOUND", "HashMap<Uuid,String>: lookup by different Uuid must return None");
}
