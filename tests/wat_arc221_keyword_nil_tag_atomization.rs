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
//!  2 — `(:wat::holon::to-holonnil)` round-trip; atom(nil) ≠ atom(:nil) (distinct from Keyword)
//!  3 — `(:wat::holon::to-holon<uuid-val>)` round-trip via tagged composition; closes arc 207
//!  4 — `HashMap<keyword, i64>` insert + lookup (keyword as map key)
//!  5 — `HashSet<keyword>` insert + contains? (keyword as set element)
//!  6 — `HashMap<Uuid, String>` insert + lookup (Uuid as map key — arc 207 false-flag close)

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    )
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn run_string(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
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
    // atom(:foo) = atom(:foo) — same keyword produces identical HolonAST.
    let same = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-foo1  (:wat::holon::to-holon :foo)
                       atom-foo2  (:wat::holon::to-holon :foo)]
                      (:wat::core::= atom-foo1 atom-foo2)))
    "#);
    assert!(same, "Atom(:foo) must equal Atom(:foo) — same Keyword leaf");

    // atom(:foo) ≠ atom(:bar) — different keywords produce distinct HolonAST.
    let diff = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-foo  (:wat::holon::to-holon :foo)
                       atom-bar  (:wat::holon::to-holon :bar)
                       eq        (:wat::core::= atom-foo atom-bar)]
                      (:wat::core::not eq)))
    "#);
    assert!(diff, "Atom(:foo) must NOT equal Atom(:bar) — distinct Keyword leaves");

    // atom(:foo) ≠ atom("foo") — Keyword leaf is distinct from String leaf.
    let not_string = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-kw  (:wat::holon::to-holon :foo)
                       atom-str (:wat::holon::to-holon "foo")
                       eq       (:wat::core::= atom-kw atom-str)]
                      (:wat::core::not eq)))
    "#);
    assert!(not_string, "Atom(:foo) must NOT equal Atom(\"foo\") — Keyword leaf distinct from String leaf");
}

// ─── Probe 2 — `(:wat::holon::to-holon :wat::core::nil)` round-trip ──────────────

/// `(:wat::holon::to-holon :wat::core::nil)` dispatches through `value_to_atom` Nil arm
/// to `HolonAST::Nil` (the proper Nil primitive leaf, not Symbol("nil")).
///
/// In WAT, nil is the keyword `:wat::core::nil` — it evaluates to `Value::Unit`
/// (wat's nil value). So `(:wat::holon::to-holon :wat::core::nil)` first evaluates
/// `:wat::core::nil` → `Value::Unit`, then `value_to_atom(Value::Unit)` → `HolonAST::Nil`.
///
/// Distinctness: atom(:wat::core::nil) must NOT equal atom(:nil) — the Nil leaf
/// (from Value::Unit via the nil keyword eval path) is distinct from a Keyword leaf
/// for ":nil" (a plain keyword named "nil").
///
/// Arc 221 Stone 221.4 — Nil arm in `value_to_atom` (Stone 221.3 Nil leaf).
#[test]
fn probe_2_nil_atom_round_trip_distinct_from_keyword_nil() {
    // atom(:wat::core::nil) = atom(:wat::core::nil) — same nil produces identical HolonAST::Nil.
    let same = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-nil1  (:wat::holon::to-holon :wat::core::nil)
                       atom-nil2  (:wat::holon::to-holon :wat::core::nil)]
                      (:wat::core::= atom-nil1 atom-nil2)))
    "#);
    assert!(same, "Atom(:wat::core::nil) must equal itself — same Nil leaf");

    // atom(:wat::core::nil) ≠ atom(:nil) — HolonAST::Nil is distinct from HolonAST::Keyword("nil").
    // :wat::core::nil evaluates to Value::Unit → HolonAST::Nil (PRIM_TAG_NIL="nil").
    // :nil evaluates to Value::keyword(":nil") → HolonAST::Keyword("nil") (PRIM_TAG_KEYWORD="keyword").
    // Two distinct leaf tags → distinct canonical bytes → distinct HolonAST identity.
    let diff = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-nil  (:wat::holon::to-holon :wat::core::nil)
                       atom-knil (:wat::holon::to-holon :nil)
                       eq        (:wat::core::= atom-nil atom-knil)]
                      (:wat::core::not eq)))
    "#);
    assert!(diff, "Atom(:wat::core::nil) must NOT equal Atom(:nil) — Nil leaf distinct from Keyword leaf");
}

// ─── Probe 3 — `(:wat::holon::to-holon<uuid-val>)` round-trip — closes arc 207 ─

/// `(:wat::holon::to-holon<uuid>)` dispatches through `value_to_atom` Uuid arm to
/// `HolonAST::Bind(Tag("uuid"), String(hex))` — the tagged composition shape.
///
/// This probe CLOSES ARC 207 FALSE-FLAG. Before Stone 221.4, `value_to_atom`
/// had no Uuid arm, so `(:wat::holon::to-holon(:wat::core::Uuid/v5 ns name))`
/// would fall through to the TypeMismatch error arm at runtime — a 5-day-latent
/// gap since 2026-05-17.
///
/// Uuid is already hashable (Arc 207 + Stone 216.5a); this stone makes it
/// atomizable for use as HashMap/HashSet keys.
///
/// Arc 221 Stone 221.4 — Uuid arm in `value_to_atom` (arc 221 doctrine).
#[test]
fn probe_3_uuid_atom_round_trip_closes_arc_207_false_flag() {
    // atom(uuid-v5-same-args) = atom(uuid-v5-same-args) — deterministic Uuid produces
    // identical HolonAST::Bind(Tag("uuid"), String(hex)) via the Uuid arm.
    let same = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [ns    (:wat::core::Uuid/nil)
                       u1    (:wat::core::Uuid/v5 ns "hello")
                       u2    (:wat::core::Uuid/v5 ns "hello")
                       a1    (:wat::holon::to-holon u1)
                       a2    (:wat::holon::to-holon u2)]
                      (:wat::core::= a1 a2)))
    "#);
    assert!(same, "Atom(Uuid/v5 same-args) must equal itself — deterministic tagged composition");

    // atom(uuid-v5-"hello") ≠ atom(uuid-v5-"world") — different UUIDs produce distinct Bind.
    let diff = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [ns    (:wat::core::Uuid/nil)
                       u1    (:wat::core::Uuid/v5 ns "hello")
                       u2    (:wat::core::Uuid/v5 ns "world")
                       a1    (:wat::holon::to-holon u1)
                       a2    (:wat::holon::to-holon u2)
                       eq    (:wat::core::= a1 a2)]
                      (:wat::core::not eq)))
    "#);
    assert!(diff, "Atom(Uuid/v5 \"hello\") must NOT equal Atom(Uuid/v5 \"world\") — distinct tagged compositions");
}

// ─── Probe 4 — `HashMap<keyword, i64>` insert + lookup ────────────────────────

/// `HashMap<keyword, i64>` with keyword keys — tag-frequency-tally pattern.
///
/// Verifies `is_atomizable(keyword)` at the check layer (Stone 221.4 doc update
/// to the pre-existing `:wat::core::keyword` entry), and that keyword values
/// serve as HashMap keys at runtime (Hash + Eq already shipped in earlier arcs).
///
/// Arc 221 Stone 221.4 — Keyword arm in `value_to_atom` enables HashMap<keyword, V>.
#[test]
fn probe_4_hashmap_keyword_key_insert_lookup() {
    // Insert :tag-a → 10, :tag-b → 20; get :tag-a returns Some(10).
    let a_val = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
                       m2  (:wat::core::HashMap/assoc m :tag-a 10)
                       m3  (:wat::core::HashMap/assoc m2 :tag-b 20)]
                      (:wat::core::match (:wat::core::HashMap/get m3 :tag-a) -> :wat::core::i64
                        ((:wat::core::Some v) v)
                        (_ -1))))
    "#);
    assert_eq!(a_val, 10, "HashMap<keyword,i64>: get :tag-a after insert must return 10");

    // get :tag-b returns Some(20).
    let b_val = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
                       m2  (:wat::core::HashMap/assoc m :tag-a 10)
                       m3  (:wat::core::HashMap/assoc m2 :tag-b 20)]
                      (:wat::core::match (:wat::core::HashMap/get m3 :tag-b) -> :wat::core::i64
                        ((:wat::core::Some v) v)
                        (_ -1))))
    "#);
    assert_eq!(b_val, 20, "HashMap<keyword,i64>: get :tag-b after insert must return 20");

    // length = 2 — two distinct keyword keys.
    let len = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
                       m2  (:wat::core::HashMap/assoc m :tag-a 10)
                       m3  (:wat::core::HashMap/assoc m2 :tag-b 20)]
                      (:wat::core::HashMap/length m3)))
    "#);
    assert_eq!(len, 2, "HashMap<keyword,i64>: two distinct keyword keys → length 2");
}

// ─── Probe 5 — `HashSet<keyword>` insert + contains? ─────────────────────────

/// `HashSet<keyword>` — tag-set pattern.
///
/// Verifies `is_atomizable(keyword)` at the check layer and that keyword values
/// serve as HashSet elements at runtime.
///
/// Arc 221 Stone 221.4 — Keyword arm enables HashSet<keyword>.
#[test]
fn probe_5_hashset_keyword_insert_contains() {
    // :foo is in the set.
    let has_foo = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [tags (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)]
                      (:wat::core::contains? tags :foo)))
    "#);
    assert!(has_foo, "HashSet<keyword>: :foo must be found in set");

    // :bar is in the set.
    let has_bar = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [tags (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)]
                      (:wat::core::contains? tags :bar)))
    "#);
    assert!(has_bar, "HashSet<keyword>: :bar must be found in set");

    // :unknown is NOT in the set.
    let no_unknown = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [tags  (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)
                       found (:wat::core::contains? tags :unknown)]
                      (:wat::core::not found)))
    "#);
    assert!(no_unknown, "HashSet<keyword>: :unknown must NOT be found in set");

    // length = 3 — three distinct keyword elements.
    let len = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [tags (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)]
                      (:wat::core::HashSet/length tags)))
    "#);
    assert_eq!(len, 3, "HashSet<keyword>: three distinct keywords → length 3");
}

// ─── Probe 6 — `HashMap<Uuid, String>` insert + lookup — closes arc 207 ──────

/// `HashMap<Uuid, String>` with Uuid keys — closes arc 207 false-flag.
///
/// Before Stone 221.4, `is_atomizable(Uuid)` was true but `value_to_atom` had
/// no Uuid arm — so `HashMap/assoc` on a `HashMap<Uuid, V>` would reach
/// `value_to_atom` at atomization time and hit the TypeMismatch error arm.
///
/// This probe verifies the full end-to-end: insert two Uuid→String pairs,
/// retrieve by the same Uuid key, confirm the lookup succeeds.
///
/// Arc 221 Stone 221.4 — Uuid arm in `value_to_atom` + Bind(Tag, String) shape.
#[test]
fn probe_6_hashmap_uuid_key_insert_lookup_closes_arc_207() {
    // Insert (v5-nil-"hello") → "world-entry"; get same uuid returns Some("world-entry").
    let retrieved = run_string(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [ns   (:wat::core::Uuid/nil)
                       u1   (:wat::core::Uuid/v5 ns "hello")
                       m    (:wat::core::HashMap :wat::core::Uuid :wat::core::String)
                       m2   (:wat::core::HashMap/assoc m u1 "world-entry")]
                      (:wat::core::match (:wat::core::HashMap/get m2 u1) -> :wat::core::String
                        ((:wat::core::Some v) v)
                        (_ "NOT-FOUND"))))
    "#);
    assert_eq!(retrieved, "world-entry", "HashMap<Uuid,String>: get after insert must return the inserted value");

    // Different Uuid key returns None (mapped to "NOT-FOUND" sentinel).
    let not_found = run_string(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [ns   (:wat::core::Uuid/nil)
                       u1   (:wat::core::Uuid/v5 ns "hello")
                       u2   (:wat::core::Uuid/v5 ns "world")
                       m    (:wat::core::HashMap :wat::core::Uuid :wat::core::String)
                       m2   (:wat::core::HashMap/assoc m u1 "hello-entry")]
                      (:wat::core::match (:wat::core::HashMap/get m2 u2) -> :wat::core::String
                        ((:wat::core::Some v) v)
                        (_ "NOT-FOUND"))))
    "#);
    assert_eq!(not_found, "NOT-FOUND", "HashMap<Uuid,String>: lookup by different Uuid must return None");
}
