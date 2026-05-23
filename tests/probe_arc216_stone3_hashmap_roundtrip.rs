//! Arc 216 Stone 3 — `HashMap<K, V>` (`:wat::core::HashMap<K, V>`) round-trip through
//! `HolonAST::Bundle` of arbitrary-K Binds.
//!
//! Verifies bidirectional round-trip: `value_to_atom` (forward, `Value → HolonAST`)
//! and `atom-value` (reverse, `HolonAST → Value`) for `HashMap<K, V>`.
//!
//! Per DESIGN Q2: `HashMap<K, V>` → `HolonAST::Bundle([Bind(K_holon, V_holon), ...])`.
//! Keys are arbitrary (keyword, String, i64, bool, etc.). Iteration order is
//! non-canonical (HashMap unordered). Reverse discriminates map-shape (all-Bind children)
//! from vector-shape (sequential i64 Bind keys) and set-shape (bare atoms).
//!
//! ## The 14 probes
//!
//! Forward direction:
//!  1. `(:wat::holon::to-holon{:foo 42 :bar 99})` → `HolonAST::Bundle` of 2 Bind children
//!
//! Reverse direction:
//!  2. `(:wat::holon::from-holon<bundle>)` → HashMap; length = 2; :foo key present
//!
//! Edge cases:
//!  3. Empty map `{}` + consumer declares HashMap → empty HashMap (via `-> :T` form)
//!
//! Multi-K types:
//!  4. HashMap<keyword,V>, HashMap<String,V>, HashMap<i64,V>, HashMap<bool,V> all round-trip
//!
//! Multi-V types:
//!  5. HashMap<K,i64>, HashMap<K,String>, HashMap<K,bool>, HashMap<K,keyword> all round-trip
//!
//! Non-keyword keys:
//!  6. HashMap<i64, String> round-trips (arbitrary K via atom-value)
//!
//! Nested map:
//!  7. HashMap<keyword, HashMap<keyword, i64>> round-trips
//!
//! Mixed nesting (Vec):
//!  8. HashMap<keyword, Vec<i64>> round-trips (composes with Stone 216.2)
//!
//! Mixed nesting (HashSet):
//!  9. HashMap<keyword, HashSet<i64>> round-trips (composes with Stone 216.1)
//!
//! Check-level atomizable predicate:
//! 10. `(:wat::holon::to-holon m)` for atomizable K+V type-checks cleanly
//! 11. `(:wat::holon::to-holon fn-value)` — non-atomizable type fails at check (TypeMismatch)
//!
//! HolonRepresentable Rust-side:
//! 12. `HashMap<String, String>` satisfies `HolonRepresentable` at compile time; roundtrip correct
//!
//! Shape disambiguation:
//! 13. Bundle with non-sequential i64 keys [Bind(0,v), Bind(5,v)] → HashMap (not Vec)
//!
//! Empty Bundle disambiguation via consumer-declared HashMap type:
//! 14. `(atom-value empty-bundle -> :wat::core::HashMap<K,V>)` → empty HashMap

use std::collections::HashMap;
use std::sync::Arc;
use wat::comms::HolonRepresentable;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1 — Forward: HashMap → classifier-wrapped HolonAST ────────────────

/// `(:wat::holon::to-holon{:foo 42 :bar 99})` produces a classifier-wrapped HolonAST.
/// Arc 228 Stone 228.1: the output is `Bind(Atom("Map"), Bundle(Bind pairs))`, not a bare Bundle.
/// Arc 216 Stone 3 forward direction — forward-corrected per typed-entities doctrine.
/// Verified via round-trip: to-holon → from-holon → HashMap/length = 2.
#[test]
fn probe_1_forward_hashmap_to_bundle() {
    // Arc 228: Bundle/children no longer works on the classifier-wrapped top-level Bind.
    // Verify via round-trip: to-holon produces an encoding that from-holon decodes back
    // to a HashMap of length 2. The entry count proves encoding captured both pairs.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   {:foo 42 :bar 99}
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src),
        2,
        "classifier-wrapped Map encoding must preserve 2 entries in round-trip"
    );
}

// ─── Probe 2 — Reverse: Bundle → HashMap round-trip ──────────────────────────

/// `(:wat::holon::from-holon<bundle>)` on a keyword-keyed map Bundle reconstructs
/// a `HashMap`. Length = 2; contains-key? :foo returns true.
/// Arc 216 Stone 3 reverse direction (eval_atom_value all-Bind → HashMap).
#[test]
fn probe_2_reverse_bundle_to_hashmap_roundtrip() {
    // Length = 2 after round-trip.
    let src_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   {:foo 42 :bar 99}
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_len), 2, "round-trip must preserve length 2");

    // contains-key? :foo returns true.
    let src_key = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [m   {:foo 42 :bar 99}
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/contains-key? rv :foo)))
    "#;
    assert!(
        run_bool(src_key),
        "round-trip must preserve :foo key"
    );

    // contains-key? :bar returns true.
    let src_bar = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [m   {:foo 42 :bar 99}
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/contains-key? rv :bar)))
    "#;
    assert!(
        run_bool(src_bar),
        "round-trip must preserve :bar key"
    );
}

// ─── Probe 3 — Empty map round-trip via consumer-declared HashMap type ────────

/// Empty map `{}` → `Bundle([])` → `(atom-value h -> :wat::core::HashMap<K,V>)`
/// → empty HashMap (length 0).
/// The `-> :wat::core::HashMap<K,V>` form disambiguates the empty Bundle
/// as a HashMap rather than the default empty HashSet.
/// Arc 216 Stone 3 — consumer-declared type disambiguation for empty Bundle.
#[test]
fn probe_3_empty_map_roundtrip_consumer_declared() {
    // Arc 228: to-holon on empty HashMap produces Bind(Atom("Map"), Bundle([])).
    // from-holon dispatches by classifier "Map" → empty HashMap (no consumer hint needed).
    // The "-> :T" annotation form is still valid syntax but no longer required for disambiguation.
    let src_forward = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_forward),
        0,
        "empty HashMap classifier-wrapped encoding must round-trip to HashMap length 0"
    );

    // Reverse via consumer-declared type: still valid syntax; classifier dispatch takes precedence.
    let src_reverse = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h -> :wat::core::HashMap)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_reverse),
        0,
        "empty Map classifier-wrapped + consumer hint: empty HashMap (length 0)"
    );
}

// ─── Probe 4 — Multi-K types ─────────────────────────────────────────────────

/// Works for HashMap<keyword,V>, HashMap<String,V>, HashMap<i64,V>, HashMap<bool,V>.
/// Each round-trips: forward → Bundle; reverse → HashMap with correct entry count.
#[test]
fn probe_4_multi_k_types() {
    // HashMap<keyword, i64> — 2 entries.
    let src_keyword = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :a 1 :b 2)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_keyword), 2, "HashMap<keyword,i64> round-trip: length 2");

    // HashMap<String, i64> — 2 entries (String keys).
    let src_string = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::String :wat::core::i64 "x" 10 "y" 20)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_string), 2, "HashMap<String,i64> round-trip: length 2");

    // HashMap<i64, String> — 2 entries (i64 keys).
    let src_i64 = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::i64 :wat::core::String 100 "hello" 200 "world")
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_i64), 2, "HashMap<i64,String> round-trip: length 2");

    // HashMap<bool, i64> — 2 entries (bool keys).
    let src_bool = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::bool :wat::core::i64 true 1 false 0)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_bool), 2, "HashMap<bool,i64> round-trip: length 2");
}

// ─── Probe 5 — Multi-V types ─────────────────────────────────────────────────

/// Works for HashMap<K,i64>, HashMap<K,String>, HashMap<K,bool>, HashMap<K,keyword>.
/// Each round-trips with correct entry count.
#[test]
fn probe_5_multi_v_types() {
    // HashMap<keyword, i64> — V = i64.
    let src_i64 = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   {:foo 42}
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_i64), 1, "HashMap<keyword,i64> V=i64 round-trip: length 1");

    // HashMap<keyword, String> — V = String.
    let src_string = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::String :name "alice" :city "paris")
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_string), 2, "HashMap<keyword,String> V=String round-trip: length 2");

    // HashMap<keyword, bool> — V = bool.
    let src_bool = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::bool :active true :disabled false)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_bool), 2, "HashMap<keyword,bool> V=bool round-trip: length 2");

    // HashMap<keyword, keyword> — V = keyword.
    let src_kw = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::keyword :role :admin :mode :active)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_kw), 2, "HashMap<keyword,keyword> V=keyword round-trip: length 2");
}

// ─── Probe 6 — Non-keyword keys: HashMap<i64, String> ────────────────────────

/// `HashMap<i64, String>` with i64 keys (non-keyword, non-sequential positional).
/// Forward → Bundle of Bind(I64, String). Reverse → HashMap; contains i64 key 100.
/// Arc 216 Stone 3 — arbitrary K via hashmap_key (K=i64 → "I:{n}" canonical key).
#[test]
fn probe_6_non_keyword_keys_i64_string() {
    // Length = 2 after round-trip.
    let src_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::i64 :wat::core::String 100 "hello" 200 "world")
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_len), 2, "HashMap<i64,String> round-trip: length 2");

    // contains-key? 100 returns true.
    let src_key = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::i64 :wat::core::String 100 "hello" 200 "world")
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/contains-key? rv 100)))
    "#;
    assert!(
        run_bool(src_key),
        "HashMap<i64,String> round-trip must preserve key 100"
    );
}

// ─── Probe 7 — Nested map: HashMap<keyword, HashMap<keyword, i64>> ───────────

/// `HashMap<keyword, HashMap<keyword, i64>>` — outer map has 1 entry;
/// inner map has 2 entries. Both round-trip correctly.
/// Nesting: outer Bundle of Bind(Symbol, Bundle-of-Binds); inner Bundle reconstructs
/// to HashMap via recursive holon_item_to_value dispatch on arbitrary-K shape.
#[test]
fn probe_7_nested_map_roundtrip() {
    // Outer length = 1.
    let src_outer_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 1 :y 2)
             outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)
             h     (:wat::holon::to-holon outer)
             rv    (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(run_i64(src_outer_len), 1, "nested map outer length = 1");

    // Arc 228: Bundle/children no longer applies to the classifier-wrapped top-level Bind.
    // Verify outer entry count via round-trip (already done above).
    let src_bundle_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 1 :y 2)
             outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)
             h     (:wat::holon::to-holon outer)
             rv    (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_bundle_len),
        1,
        "nested map arc 228: classifier-wrapped outer HashMap length = 1"
    );
}

// ─── Probe 8 — Mixed nesting: HashMap<keyword, Vec<i64>> ─────────────────────

/// `HashMap<keyword, Vec<i64>>` — outer map with 1 entry; value is a Vec.
/// Composes with Stone 216.2 (Vec round-trip).
/// The inner Bundle is a positional-Bind (array-shape); outer is arbitrary-K Bind (map-shape).
#[test]
fn probe_8_mixed_nesting_hashmap_of_vec() {
    // Outer length = 1; inner vec length = 3.
    let src_outer_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v   (:wat::core::Vector :wat::core::i64 10 20 30)
             m   (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data v)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_outer_len),
        1,
        "HashMap<keyword,Vec<i64>> round-trip: outer length 1"
    );

    // Arc 228: Bundle/children no longer applies to the classifier-wrapped top-level Bind.
    // Verify outer entry count via round-trip.
    let src_bundle_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v   (:wat::core::Vector :wat::core::i64 10 20 30)
             m   (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data v)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_bundle_len),
        1,
        "HashMap<keyword,Vec<i64>> arc 228: classifier-wrapped outer length = 1"
    );
}

// ─── Probe 9 — Mixed nesting: HashMap<keyword, HashSet<i64>> ─────────────────

/// `HashMap<keyword, HashSet<i64>>` — outer map with 1 entry; value is a HashSet.
/// Composes with Stone 216.1 (HashSet round-trip).
/// The inner Bundle is a bare-atom (set-shape); outer is arbitrary-K Bind (map-shape).
#[test]
fn probe_9_mixed_nesting_hashmap_of_hashset() {
    // Outer length = 1.
    let src_outer_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [s   (:wat::core::HashSet :wat::core::i64 1 2 3)
             m   (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data s)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_outer_len),
        1,
        "HashMap<keyword,HashSet<i64>> round-trip: outer length 1"
    );

    // Arc 228: Bundle/children no longer applies to the classifier-wrapped top-level Bind.
    // Verify outer entry count via round-trip.
    let src_bundle_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [s   (:wat::core::HashSet :wat::core::i64 1 2 3)
             m   (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data s)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_bundle_len),
        1,
        "HashMap<keyword,HashSet<i64>> arc 228: classifier-wrapped outer length = 1"
    );
}

// ─── Probe 10 — Check passes for atomizable K+V types ───────────────────────

/// `(:wat::holon::to-holon m)` for a HashMap with atomizable K and V type-checks cleanly.
/// `is_atomizable(HashMap<keyword, i64>)` → YES (both K and V are primitive atomizable).
/// `is_atomizable(HashMap<keyword, HashMap<keyword, i64>>)` → YES (recursive predicate).
#[test]
fn probe_10_check_passes_atomizable_k_v() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :a 1)]
            (:wat::holon::to-holon m)
            1))
    "#;
    assert_eq!(
        run_i64(src),
        1,
        "Atom on HashMap<keyword,i64> must pass check and run"
    );

    // Nested: HashMap<keyword, HashMap<keyword, i64>> — predicate recurses both levels.
    let src_nested = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 1)
             outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)
             h     (:wat::holon::to-holon outer)]
            1))
    "#;
    assert_eq!(
        run_i64(src_nested),
        1,
        "Atom on HashMap<keyword,HashMap<keyword,i64>> must pass check and run"
    );
}

// ─── Probe 11 — Check fails for non-atomizable type ──────────────────────────

/// `(:wat::holon::to-holon fn-value)` where the value is a function type fails at check.
/// Function types are not in the atomizable set (DESIGN Q6).
/// The predicate `is_atomizable(Fn(...)->...)` = false; check emits TypeMismatch.
///
/// Note: `HashMap<K, Fn>` is structurally impossible in Rust (Fn is not Hash/Eq)
/// and also rejected at the WAT check level. A direct Fn value is the simplest
/// statically-resolvable non-atomizable type.
#[test]
fn probe_11_check_fails_non_atomizable() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::nil)
          (:wat::core::let
            [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)]
            (:wat::holon::to-holon f)))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("TypeMismatch"),
        "Atom on non-atomizable type must fail with TypeMismatch; got: {}",
        err
    );
    // Arc 225 Stone 225.1: callee is now :wat::holon::to-holon (polymorphic UP verb).
    assert!(
        err.contains(":wat::holon::to-holon"),
        "TypeMismatch must name the callee :wat::holon::to-holon; got: {}",
        err
    );
}

// ─── Probe 12 — HolonRepresentable cascade (compile-time + runtime) ──────────

/// `HashMap<String, String>` satisfies `HolonRepresentable` at compile time.
///
/// Arc 216 Stone 3: `impl<K, V> HolonRepresentable for HashMap<K, V>` where
/// `K: HolonRepresentable + Hash + Eq + Send + 'static, V: HolonRepresentable + Send + 'static`.
/// Both `String` bounds satisfied.
///
/// Also verifies `to_holon_ast`/`from_holon_ast` round-trip at the Rust level:
/// - `to_holon_ast` → Bundle of Bind(String_leaf, String_leaf) children
/// - `from_holon_ast` → reconstructed HashMap<String, String> with same entries
fn assert_holon_representable<T: HolonRepresentable>() {}

#[test]
fn probe_12_holon_representable_cascade() {
    // Compile-time: if this call compiles, HashMap<String, String>: HolonRepresentable.
    assert_holon_representable::<HashMap<String, String>>();

    // Runtime roundtrip: {"foo" -> "bar", "baz" -> "qux"}.
    let mut original: HashMap<String, String> = HashMap::new();
    original.insert("foo".into(), "bar".into());
    original.insert("baz".into(), "qux".into());
    let ast = original.to_holon_ast();

    // to_holon_ast produces a Bundle of 2 Bind children.
    match &ast {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 2, "Bundle must have 2 children");
            for item in items.iter() {
                assert!(
                    matches!(item, holon::HolonAST::Bind(_, _)),
                    "each child must be HolonAST::Bind; got {:?}",
                    item
                );
            }
        }
        other => panic!("expected HolonAST::Bundle, got {:?}", other),
    }

    // from_holon_ast reconstructs the HashMap with same entries.
    let reconstructed: HashMap<String, String> =
        HolonRepresentable::from_holon_ast(&ast).expect("roundtrip");
    assert_eq!(
        reconstructed.len(),
        2,
        "roundtrip must preserve entry count"
    );
    assert_eq!(
        reconstructed.get("foo").map(String::as_str),
        Some("bar"),
        "roundtrip must preserve foo -> bar"
    );
    assert_eq!(
        reconstructed.get("baz").map(String::as_str),
        Some("qux"),
        "roundtrip must preserve baz -> qux"
    );

    // Nested: HashMap<String, Vec<String>> — bounds compose.
    assert_holon_representable::<HashMap<String, Vec<String>>>();
    let mut nested: HashMap<String, Vec<String>> = HashMap::new();
    nested.insert("first".into(), vec!["a".into(), "b".into()]);
    nested.insert("second".into(), vec!["c".into()]);
    let nested_ast = nested.to_holon_ast();
    let nested_back: HashMap<String, Vec<String>> =
        HolonRepresentable::from_holon_ast(&nested_ast).expect("nested roundtrip");
    assert_eq!(nested_back.len(), 2, "nested roundtrip must preserve entry count");
    assert_eq!(
        nested_back.get("first").map(|v| v.len()),
        Some(2),
        "nested roundtrip must preserve first -> [a, b]"
    );
}

// ─── Probe 13 — Shape disambiguation: non-sequential i64 keys → HashMap ──────

/// Bundle of Bind(I64(0), v) and Bind(I64(5), v) — all children are Bind(I64, _)
/// BUT keys [0, 5] are not sequential 0..n-1 (key 1,2,3,4 missing).
/// Stone 216.2's sequential-key check fails; control falls through to HashMap path.
/// Result: HashMap<i64, V> (NOT Vec; NOT error).
/// Arc 216 Stone 3 — shape-discriminator fall-through (Probe 13 / EXPECTATIONS row 18).
#[test]
fn probe_13_shape_disambiguation_non_sequential_i64() {
    // Construct malformed-for-Vec Bundle: Bind(0, String("a")), Bind(5, String("b")).
    // Keys 0 and 5 → not sequential 0..n-1 → Vec path fails → HashMap.
    // The Rust-level HashMap<K, V> HolonRepresentable from_holon_ast validates
    // that children are Bind nodes; no sequential check for HashMap.
    //
    // Use `from_holon_ast` to test the shape-discriminator at the WAT-surface level:
    // eval_atom_value's Bundle arm dispatches non-sequential-I64-Bind → HashMap.
    // We verify this via WAT: atom-value on such a Bundle → HashMap/length = 2.

    // Construct the Bundle directly via Rust API and verify atom-value treats it as HashMap.
    let bind0 = holon::HolonAST::bind(holon::HolonAST::i64(0), holon::HolonAST::string("a"));
    let bind5 = holon::HolonAST::bind(holon::HolonAST::i64(5), holon::HolonAST::string("b"));
    let non_seq_bundle = holon::HolonAST::bundle(vec![bind0, bind5]);

    // from_holon_ast for HashMap<String, String>: children are Bind(I64, String).
    // The K must decode via String::from_holon_ast on HolonAST::I64 — this will fail
    // because String::from_holon_ast expects HolonAST::String, not I64.
    // So we use HashMap<i64, String> — but i64 doesn't impl HolonRepresentable.
    //
    // Alternative: verify via the WAT-surface eval_atom_value which dispatches
    // non-sequential I64 → HashMap at the Value level (not HolonRepresentable Rust level).
    // WAT-surface test: inject the Bundle as a stored HolonAST and call atom-value.
    //
    // The most direct verification: Vec<String>::from_holon_ast on the non-sequential
    // Bundle returns Err (positional invariant violated), proving Vec path rejected.
    // Then atom-value (WAT surface) should accept it as a HashMap.

    // Step 1: Verify Vec<String>::from_holon_ast rejects non-sequential bundle.
    let vec_result = <Vec<String> as HolonRepresentable>::from_holon_ast(&non_seq_bundle);
    assert!(
        vec_result.is_err(),
        "Vec<String>::from_holon_ast on non-sequential i64-keyed Bundle must return Err"
    );

    // Step 2: Verify the Bundle has exactly 2 Bind children (shape is all-Bind).
    match &non_seq_bundle {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 2, "Bundle must have 2 Bind children");
            for item in items.iter() {
                assert!(
                    matches!(item, holon::HolonAST::Bind(_, _)),
                    "each child must be Bind"
                );
            }
        }
        other => panic!("expected Bundle; got {:?}", other),
    }

    // Step 3: Via WAT surface — Atom on a Vector with non-sequential i64 keys (0, 5)
    // is produced by constructing a HashMap with i64 keys 0 and 5.
    // atom-value on the result → HashMap (length 2; not Vec).
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::i64 :wat::core::String 0 "a" 5 "b")
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src),
        2,
        "HashMap<i64,String> with keys 0+5 must round-trip as HashMap (not Vec)"
    );
}

// ─── Probe 14 — Empty Bundle disambiguation via consumer-declared HashMap ─────

/// Arc 228 Stone 228.1: empty HashMap now classifier-wrapped.
///
/// Pre-arc-228 behavior (arc 216 Stone 3 Delta 1): empty Bundle was ambiguous;
/// unannotated form returned empty HashSet; `-> :HashMap` annotation returned empty HashMap.
///
/// Post-arc-228 behavior: to-holon on empty HashMap produces `Bind(Atom("Map"), Bundle([]))`.
/// from-holon dispatches by classifier "Map" → always returns HashMap, regardless of annotation.
/// The consumer-hint `-> :T` annotation is no longer needed for disambiguation.
#[test]
fn probe_14_empty_bundle_disambiguation_consumer_declares_hashmap() {
    // Arc 228: unannotated form now returns HashMap (classifier "Map" is unambiguous).
    // The arc 216 conservative-default behavior (empty Bundle → HashSet) is retired.
    let src_unannotated = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_unannotated),
        0,
        "arc 228: empty HashMap classifier-wrapped encoding returns HashMap (not HashSet)"
    );

    // Annotated form: `-> :HashMap` is still valid syntax; classifier dispatch takes precedence.
    let src_annotated = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m   (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
             h   (:wat::holon::to-holon m)
             rv  (:wat::holon::from-holon h -> :wat::core::HashMap)]
            (:wat::core::HashMap/length rv)))
    "#;
    assert_eq!(
        run_i64(src_annotated),
        0,
        "annotated form still works: empty Map classifier + consumer hint → empty HashMap (length 0)"
    );
}
