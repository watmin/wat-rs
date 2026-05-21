//! Arc 216 Stone 5 — `hashmap_key` full coverage.
//!
//! Verifies that every type admitted by `is_atomizable` is also hashable via
//! `hashmap_key` — closing the predicate→runtime contract gap surfaced by Stone
//! 216.4's Delta 2 substitution.
//!
//! Gap before this stone: `hashmap_key` accepted only
//!   {String, i64, f64, bool, keyword, HolonAST, Uuid, HashSet<T>}.
//! Missing arms: `Value::Vec`, `Value::wat__std__HashMap`, `Value::wat__WatAST`.
//!
//! After this stone: `hashmap_key` accepts all atomizable types; the thesis of
//! arc 216 ("class of failure eliminated: 'values that look HolonRepresentable
//! but silently aren't at runtime'") is now TRUE on the branch.
//!
//! ## The 12 probes (16 EXPECTATIONS rows total)
//!
//! Rows 7-15 map to probes here (rows 1-6 are implementation rows):
//!
//!  1. `HashSet<Vector<i64>>` round-trip (the verify-gap probe's positive twin)
//!  2. `HashSet<HashMap<keyword, i64>>` round-trip
//!  3. `HashSet<WatAST>` round-trip — WatAST constructible via `quote`
//!  4. `HashMap<Vector<i64>, String>` round-trip — Vector as K
//!  5. `HashMap<HashMap<keyword, i64>, String>` round-trip — HashMap as K
//!  6. `HashMap<WatAST, String>` round-trip — WatAST as K
//!  7. Nested: `HashSet<Vector<HashSet<i64>>>` — three-deep nesting
//!  8. Nested: `HashMap<Vector<i64>, HashSet<i64>>`
//!  9. Dedupe: `HashSet<Vector<i64>>` with two equal-content Vectors collapses
//! 10. Diagnostic: new `other =>` message names Vec, HashMap, WatAST
//! 11. Collision-safety: `["a","b,c"]` vs `["a,b","c"]` → distinct canonical keys
//! 12. `HolonRepresentable` cascade: `Vec<String>` round-trip at Rust level
//!
//! `src/runtime.rs` — `hashmap_key` function (Stone 5 new arms at the
//! `Value::Vec`, `Value::wat__std__HashMap`, and `Value::wat__WatAST` match arms).

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

fn runtime_err(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env) {
        Ok(v) => panic!("expected runtime error; got {:?}", v),
        Err(e) => format!("{}", e),
    }
}

// ─── Probe 1 — HashSet<Vector<i64>> round-trip ───────────────────────────────

/// `HashSet<Vector<i64>>` — the positive twin of the verify-gap probe.
///
/// Before Stone 216.5: `hashmap_key` rejected `Value::Vec` at runtime with
/// `TypeMismatch { expected: "hashable value (primitive, HolonAST, or HashSet<T>)",
/// got: "wat::core::Vector" }`.
///
/// After Stone 216.5: `Value::Vec` arm added; length-prefix canonical key scheme.
/// Two distinct Vectors → outer HashSet length = 2 → Atom produces Bundle with 2 children.
///
/// Arc 216 Stone 5 Probe 1.
#[test]
fn probe_1_hashset_of_vector_roundtrip() {
    // Forward: HashSet<Vector<i64>> → Atom → Bundle with 2 children.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v1     (:wat::core::Vector :wat::core::i64 1 2)
             v2     (:wat::core::Vector :wat::core::i64 3 4)
             outer  (:wat::core::HashSet :wat::type::Infer v1 v2)
             h      (:wat::holon::Atom outer)
             cs     (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 2, "HashSet<Vector<i64>> Atom must produce Bundle with 2 children");

    // Reverse: atom-value reconstructs the HashSet; length = 2.
    let src_rev = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v1     (:wat::core::Vector :wat::core::i64 1 2)
             v2     (:wat::core::Vector :wat::core::i64 3 4)
             outer  (:wat::core::HashSet :wat::type::Infer v1 v2)
             h      (:wat::holon::Atom outer)
             s      (:wat::core::atom-value h)]
            (:wat::core::HashSet/length s)))
    "#;
    assert_eq!(run_i64(src_rev), 2, "atom-value on HashSet<Vector<i64>> must reconstruct length 2");
}

// ─── Probe 2 — HashSet<HashMap<keyword, i64>> round-trip ─────────────────────

/// `HashSet<HashMap<keyword, i64>>` — HashMap as HashSet element type.
///
/// Before Stone 216.5: `hashmap_key` rejected `Value::wat__std__HashMap`.
/// After: `Value::wat__std__HashMap` arm added; sorted-pairs canonical key scheme.
///
/// Arc 216 Stone 5 Probe 2.
#[test]
fn probe_2_hashset_of_hashmap_roundtrip() {
    // Two distinct HashMaps → outer HashSet length = 2 → Bundle with 2 children.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m1    (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :a 1)
             m2    (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :b 2)
             outer (:wat::core::HashSet :wat::type::Infer m1 m2)
             h     (:wat::holon::Atom outer)
             cs    (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 2, "HashSet<HashMap<keyword,i64>> Atom must produce Bundle with 2 children");

    // Reverse round-trip: atom-value → length = 2.
    let src_rev = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m1    (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :a 1)
             m2    (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :b 2)
             outer (:wat::core::HashSet :wat::type::Infer m1 m2)
             h     (:wat::holon::Atom outer)
             s     (:wat::core::atom-value h)]
            (:wat::core::HashSet/length s)))
    "#;
    assert_eq!(run_i64(src_rev), 2, "atom-value on HashSet<HashMap> must reconstruct length 2");
}

// ─── Probe 3 — HashSet<WatAST> round-trip ────────────────────────────────────

/// `HashSet<WatAST>` — WatAST as HashSet element type.
///
/// WatAST values are constructible at WAT surface via `(:wat::core::quote expr)`.
/// Before Stone 216.5: `hashmap_key` had no `Value::wat__WatAST` arm and would
/// reject WatAST values.
/// After: `Value::wat__WatAST` arm added; DefaultHasher over Debug representation.
///
/// Arc 216 Stone 5 Probe 3.
#[test]
fn probe_3_hashset_of_watast_roundtrip() {
    // Two distinct quoted WAT AST nodes → HashSet<WatAST> length = 2 → Bundle with 2 children.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [q1    (:wat::core::quote :foo)
             q2    (:wat::core::quote :bar)
             outer (:wat::core::HashSet :wat::WatAST q1 q2)
             h     (:wat::holon::Atom outer)
             cs    (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 2, "HashSet<WatAST> Atom must produce Bundle with 2 children");
}

// ─── Probe 4 — HashMap<Vector<i64>, String> round-trip ───────────────────────

/// `HashMap<Vector<i64>, String>` — Vector as HashMap key type.
///
/// Before Stone 216.5: `hashmap_key` rejected `Value::Vec`; HashMap construction
/// with a Vector key would fail at runtime.
/// After: `Value::Vec` arm added.
///
/// Note: `:K` type parameter uses `:wat::type::Infer` (the HashMap constructor
/// only accepts simple keyword type args; parameterized type keywords like
/// `:wat::core::Vector<i64>` are not valid in that position). Type is inferred
/// from the provided key argument.
///
/// Arc 216 Stone 5 Probe 4.
#[test]
fn probe_4_hashmap_vector_key_roundtrip() {
    // HashMap<Vector<i64>, String> with 1 entry → Atom → Bundle with 1 child.
    // Use :Infer for the K type parameter; type is inferred from the key value.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [k   (:wat::core::Vector :wat::core::i64 10 20)
             m   (:wat::core::HashMap :wat::type::Infer :wat::core::String k "hello")
             h   (:wat::holon::Atom m)
             cs  (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 1, "HashMap<Vector<i64>,String> Atom must produce Bundle with 1 child");

    // Contains-key round-trip: the original key must be found.
    let src_contains = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [k   (:wat::core::Vector :wat::core::i64 10 20)
             m   (:wat::core::HashMap :wat::type::Infer :wat::core::String k "hello")]
            (:wat::core::HashMap/contains-key? m k)))
    "#;
    assert!(run_bool(src_contains), "HashMap<Vector,String> must contain the Vector key");
}

// ─── Probe 5 — HashMap<HashMap<keyword, i64>, String> round-trip ─────────────

/// `HashMap<HashMap<keyword, i64>, String>` — HashMap as HashMap key type.
///
/// Before Stone 216.5: `hashmap_key` rejected `Value::wat__std__HashMap`.
/// After: `Value::wat__std__HashMap` arm added; sorted-pairs canonical key.
///
/// Note: `:K` type parameter uses `:wat::type::Infer`; type inferred from the key.
///
/// Arc 216 Stone 5 Probe 5.
#[test]
fn probe_5_hashmap_hashmap_key_roundtrip() {
    // HashMap<HashMap<keyword,i64>, String> with 1 entry → Atom → Bundle with 1 child.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [inner  (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 99)
             outer  (:wat::core::HashMap :wat::type::Infer :wat::core::String inner "val")
             h      (:wat::holon::Atom outer)
             cs     (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 1, "HashMap<HashMap,String> Atom must produce Bundle with 1 child");

    // contains-key? with the inner map as key.
    let src_contains = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [inner  (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 99)
             outer  (:wat::core::HashMap :wat::type::Infer :wat::core::String inner "val")]
            (:wat::core::HashMap/contains-key? outer inner)))
    "#;
    assert!(run_bool(src_contains), "HashMap<HashMap,String> must contain the inner map as key");
}

// ─── Probe 6 — HashMap<WatAST, String> round-trip ────────────────────────────

/// `HashMap<WatAST, String>` — WatAST as HashMap key type.
///
/// WatAST constructible via `(:wat::core::quote expr)`.
/// Before Stone 216.5: `hashmap_key` had no `Value::wat__WatAST` arm.
/// After: arm added; DefaultHasher over Debug representation.
///
/// Arc 216 Stone 5 Probe 6.
#[test]
fn probe_6_hashmap_watast_key_roundtrip() {
    // HashMap<WatAST, String> with 1 entry → length = 1.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [k  (:wat::core::quote :some-key)
             m  (:wat::core::HashMap :wat::WatAST :wat::core::String k "world")
             h  (:wat::holon::Atom m)
             cs (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 1, "HashMap<WatAST,String> Atom must produce Bundle with 1 child");

    // contains-key? with the quoted AST.
    let src_contains = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [k  (:wat::core::quote :some-key)
             m  (:wat::core::HashMap :wat::WatAST :wat::core::String k "world")]
            (:wat::core::HashMap/contains-key? m k)))
    "#;
    assert!(run_bool(src_contains), "HashMap<WatAST,String> must contain the quoted key");
}

// ─── Probe 7 — Nested: HashSet<Vector<HashSet<i64>>> ─────────────────────────

/// Three-deep nesting: `HashSet<Vector<HashSet<i64>>>`.
///
/// Predicate recursion path:
///   is_atomizable(HashSet<Vector<HashSet<i64>>>)
///   → is_atomizable(Vector<HashSet<i64>>)
///     → is_atomizable(HashSet<i64>) → is_atomizable(i64) = true
///
/// Runtime: `hashmap_key` for each HashSet element is a Vector; the Vec arm
/// recurses into each Vector element, which is a HashSet; the HashSet arm
/// recurses into each set element (i64). All three recursive arms compose.
///
/// Arc 216 Stone 5 Probe 7.
#[test]
fn probe_7_nested_hashset_vector_hashset() {
    // One Vector<HashSet<i64>> → outer HashSet length = 1 → Bundle with 1 child.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [s1     (:wat::core::HashSet :wat::core::i64 1 2)
             v      (:wat::core::Vector :wat::type::Infer s1)
             outer  (:wat::core::HashSet :wat::type::Infer v)
             h      (:wat::holon::Atom outer)
             cs     (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 1, "HashSet<Vector<HashSet<i64>>> Atom must produce Bundle with 1 child");
}

// ─── Probe 8 — Nested: HashMap<Vector<i64>, HashSet<i64>> ────────────────────

/// `HashMap<Vector<i64>, HashSet<i64>>` — Vector as K; HashSet as V.
///
/// Both K and V need `hashmap_key` support. Vec arm handles K; HashSet arm
/// handles V (when V is atomized for the round-trip).
///
/// Note: both `:K` and `:V` use `:wat::type::Infer`; types inferred from args.
///
/// Arc 216 Stone 5 Probe 8.
#[test]
fn probe_8_hashmap_vector_key_hashset_val() {
    // HashMap<Vector<i64>, HashSet<i64>> with 1 entry → Atom → Bundle with 1 child.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [k   (:wat::core::Vector :wat::core::i64 5 6)
             v   (:wat::core::HashSet :wat::core::i64 7 8)
             m   (:wat::core::HashMap :wat::type::Infer :wat::type::Infer k v)
             h   (:wat::holon::Atom m)
             cs  (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 1, "HashMap<Vector<i64>,HashSet<i64>> Atom must produce Bundle with 1 child");
}

// ─── Probe 9 — Dedupe: equal-content Vectors collapse in HashSet ──────────────

/// Two `Vector<i64>` values with identical content (`[1,2]` and `[1,2]`) should
/// produce the SAME canonical key via the length-prefix scheme, and thus be
/// deduped to a single element in `HashSet<Vector<i64>>`.
///
/// This verifies the canonical-key scheme correctly identifies equal Vectors as
/// the same HashSet element (dedupe semantics).
///
/// Arc 216 Stone 5 Probe 9.
#[test]
fn probe_9_dedupe_equal_content_vectors() {
    // Two Vectors with identical elements [1,2] → HashSet dedupes to length 1.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v1    (:wat::core::Vector :wat::core::i64 1 2)
             v2    (:wat::core::Vector :wat::core::i64 1 2)
             outer (:wat::core::HashSet :wat::type::Infer v1 v2)]
            (:wat::core::HashSet/length outer)))
    "#;
    assert_eq!(run_i64(src), 1, "Two Vectors with same content must dedupe to 1 element in HashSet");
}

// ─── Probe 10 — Diagnostic message updated ────────────────────────────────────

/// The `other =>` arm in `hashmap_key` must enumerate the new accepted set
/// honestly. Before Stone 216.5 the message was:
///   "hashable value (primitive, HolonAST, or HashSet<T>)"
/// After Stone 216.5 it must include Vec, HashMap, WatAST:
///   "hashable value (primitive, HolonAST, WatAST, HashSet<T>, Vec<T>, or HashMap<K,V>)"
///
/// We trigger the error by attempting to use a non-hashable inline fn value
/// as a HashSet element. The fn value reaches `hashmap_key` and hits the
/// `other =>` arm because Function values aren't structural.
///
/// Note: this probe triggers a RUNTIME error (the check-level is_atomizable
/// returns true for Infer; fn type is non-atomizable but Infer is conservative).
/// We verify the error message contains the newly added type names.
///
/// Arc 216 Stone 5 Probe 10.
#[test]
fn probe_10_diagnostic_message_updated() {
    // An inline fn value is not hashable; triggers the `other =>` arm at runtime.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [f  (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
             s  (:wat::core::HashSet :wat::type::Infer f)]
            (:wat::core::HashSet/length s)))
    "#;
    let err = runtime_err(src);
    assert!(
        err.contains("Vec") || err.contains("Vec<T>"),
        "diagnostic must mention Vec; got: {}",
        err
    );
    assert!(
        err.contains("HashMap"),
        "diagnostic must mention HashMap; got: {}",
        err
    );
    assert!(
        err.contains("WatAST"),
        "diagnostic must mention WatAST; got: {}",
        err
    );
}

// ─── Probe 11 — Collision-safety: length-prefix scheme ───────────────────────

/// Two Vecs with content `["a", "b,c"]` and `["a,b", "c"]` must produce
/// DIFFERENT canonical keys via the length-prefix scheme.
///
/// Under naive comma-join both would produce `"a,b,c"` — identical, causing
/// false dedupe in a `HashSet<Vector<String>>`. Under the length-prefix scheme:
///   `["a", "b,c"]` → `"Vec:[3:S:a,5:S:b,c]"`
///   `["a,b", "c"]` → `"Vec:[5:S:a,b,3:S:c]"`
/// These are distinct. The HashSet must contain 2 elements, not 1.
///
/// Arc 216 Stone 5 Probe 11 — collision-safety gate.
#[test]
fn probe_11_collision_safety_length_prefix() {
    // Two Vectors that would collide under naive comma-join must NOT collide
    // under the length-prefix scheme.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v1    (:wat::core::Vector :wat::core::String "a" "b,c")
             v2    (:wat::core::Vector :wat::core::String "a,b" "c")
             outer (:wat::core::HashSet :wat::type::Infer v1 v2)]
            (:wat::core::HashSet/length outer)))
    "#;
    assert_eq!(
        run_i64(src),
        2,
        "Vecs [\"a\",\"b,c\"] and [\"a,b\",\"c\"] must NOT collide — must produce 2 distinct HashSet elements"
    );
}

// ─── Probe 12 — HolonRepresentable cascade: Vec<String> ──────────────────────

/// `Vec<String>` satisfies `HolonRepresentable` at compile time (Stone 216.2).
/// This probe verifies the round-trip at the Rust level: `to_holon_ast` and
/// `from_holon_ast` on a `Vec<String>` produce the same value.
///
/// This is the Rust-level cascade check: once `HolonRepresentable` is impl'd for
/// `Vec<T: HolonRepresentable>`, any nesting composes automatically.
///
/// Arc 216 Stone 5 Probe 12.
#[test]
fn probe_12_holon_representable_vec_cascade() {
    fn assert_holon_representable<T: HolonRepresentable>() {}
    assert_holon_representable::<Vec<String>>();

    // Round-trip at Rust level.
    let original: Vec<String> = vec!["hello".to_string(), "world".to_string()];
    let holon_ast = original.to_holon_ast();
    let reconstructed = Vec::<String>::from_holon_ast(&holon_ast)
        .expect("from_holon_ast must succeed for Vec<String>");
    assert_eq!(
        original, reconstructed,
        "Vec<String> HolonRepresentable round-trip must be identity"
    );
}
