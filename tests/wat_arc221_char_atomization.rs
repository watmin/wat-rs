//! Arc 221 Stone 221.2 — `:wat::core::Char` atomization probes.
//!
//! Verifies that `Value::wat__core__Char` is fully atomizable end-to-end:
//!   (a) `(:wat::holon::Atom \c)` dispatches via `value_to_atom` Char arm to
//!       `HolonAST::char_(c)` (Stone 221.2 — holon-rs commit `243eded`).
//!   (b) `is_atomizable(":wat::core::Char")` returns true (Stone 221.2 predicate
//!       extension), so the check layer approves `HashMap<Char, V>` and `HashSet<Char>`.
//!   (c) Char is usable as a HashMap key and HashSet element at the WAT runtime level.
//!
//! ## Tests
//!
//!  1 — `(:wat::holon::to-holon \a)` round-trip; atom(Char) distinct from atom(i64)
//!  2 — `HashMap<Char, i64>` insert + lookup (char-frequency-tally pattern)
//!  3 — `HashSet<Char>` insert + contains? (vowels-set pattern)

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

// ─── Probe 1 — `(:wat::holon::to-holon \a)` round-trip; atom(Char) ≠ atom(i64) ──

/// `(:wat::holon::to-holon \a)` dispatches through `value_to_atom` Char arm to
/// `HolonAST::char_('a')`. The result is a `Value::holon__HolonAST` wrapping
/// a `HolonAST::Char('a')` leaf.
///
/// Cross-type distinctness: `atom(\a)` must NOT equal `atom(97)` — the i64
/// representation of 'a' as a codepoint. This proves Char is encoded as a
/// first-class primitive leaf, not silently coerced to i64.
///
/// Arc 221 Stone 221.2 — Char arm in `value_to_atom` (Stone 221.1 Char leaf).
#[test]
fn probe_1_char_atom_round_trip_distinct_from_i64() {
    // atom(\a) = atom(\a) — same Char produces identical HolonAST.
    let same = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-a1  (:wat::holon::to-holon \a)
                       atom-a2  (:wat::holon::to-holon \a)]
                      (:wat::core::= atom-a1 atom-a2)))
    "#);
    assert!(same, "Atom(\\a) must equal Atom(\\a) — same Char leaf");

    // atom(\a) ≠ atom(\b) — different Chars produce distinct HolonAST.
    let diff = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-a  (:wat::holon::to-holon \a)
                       atom-b  (:wat::holon::to-holon \b)
                       eq      (:wat::core::= atom-a atom-b)]
                      (:wat::core::not eq)))
    "#);
    assert!(diff, "Atom(\\a) must NOT equal Atom(\\b) — distinct Char leaves");

    // atom(\a) ≠ atom(97) — Char leaf is distinct from i64 leaf.
    // 'a' has codepoint 97; HolonAST::Char('a') ≠ HolonAST::I64(97).
    let not_i64 = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [atom-char  (:wat::holon::to-holon \a)
                       atom-int   (:wat::holon::to-holon 97)
                       eq         (:wat::core::= atom-char atom-int)]
                      (:wat::core::not eq)))
    "#);
    assert!(not_i64, "Atom(\\a) must NOT equal Atom(97) — Char leaf is distinct from i64 leaf");
}

// ─── Probe 2 — `HashMap<Char, i64>` insert + lookup ─────────────────────────

/// `HashMap<Char, i64>` with Char keys — char-frequency-tally pattern.
///
/// Verifies that `is_atomizable(Char)` returns true at the check layer
/// (enabling `HashMap<Char, V>` as atomizable K), and that Char values can
/// serve as HashMap keys at runtime (Hash + Eq impls from Stone 220.2).
///
/// Also verifies `HashMap/assoc` builds the map and `HashMap/get` looks up
/// by Char key, returning `Some(v)` on hit.
///
/// Arc 221 Stone 221.2 — `is_atomizable` Char extension enables HashMap<Char,V>.
#[test]
fn probe_2_hashmap_char_key_insert_lookup() {
    // Insert \a -> 3, \b -> 7; get \a returns Some(3).
    let a_val = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [tally   (:wat::core::HashMap :wat::core::Char :wat::core::i64)
                       tally2  (:wat::core::HashMap/assoc tally \a 3)
                       tally3  (:wat::core::HashMap/assoc tally2 \b 7)]
                      (:wat::core::match (:wat::core::HashMap/get tally3 \a) -> :wat::core::i64
                        ((:wat::core::Some v) v)
                        (_ -1))))
    "#);
    assert_eq!(a_val, 3, "HashMap<Char,i64>: get \\a after insert must return 3");

    // get \b returns Some(7).
    let b_val = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [tally   (:wat::core::HashMap :wat::core::Char :wat::core::i64)
                       tally2  (:wat::core::HashMap/assoc tally \a 3)
                       tally3  (:wat::core::HashMap/assoc tally2 \b 7)]
                      (:wat::core::match (:wat::core::HashMap/get tally3 \b) -> :wat::core::i64
                        ((:wat::core::Some v) v)
                        (_ -1))))
    "#);
    assert_eq!(b_val, 7, "HashMap<Char,i64>: get \\b after insert must return 7");

    // length = 2 — two distinct Char keys.
    let len = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [tally   (:wat::core::HashMap :wat::core::Char :wat::core::i64)
                       tally2  (:wat::core::HashMap/assoc tally \a 3)
                       tally3  (:wat::core::HashMap/assoc tally2 \b 7)]
                      (:wat::core::HashMap/length tally3)))
    "#);
    assert_eq!(len, 2, "HashMap<Char,i64>: two distinct Char keys → length 2");
}

// ─── Probe 3 — `HashSet<Char>` insert + contains? ────────────────────────────

/// `HashSet<Char>` — vowels-set pattern.
///
/// Verifies that `is_atomizable(Char)` at the check layer permits
/// `HashSet<Char>` as an atomizable element type, and that Char values
/// serve as HashSet elements at runtime (Hash + Eq impls from Stone 220.2).
///
/// `contains?` returns true for a vowel in the set, false for a consonant.
///
/// Arc 221 Stone 221.2 — `is_atomizable` Char extension enables HashSet<Char>.
#[test]
fn probe_3_hashset_char_insert_contains() {
    // \a is in the vowels set.
    let has_a = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [vowels (:wat::core::HashSet :wat::core::Char \a \e \i \o \u)]
                      (:wat::core::contains? vowels \a)))
    "#);
    assert!(has_a, "HashSet<Char>: \\a must be found in vowels set");

    // \e is in the vowels set.
    let has_e = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [vowels (:wat::core::HashSet :wat::core::Char \a \e \i \o \u)]
                      (:wat::core::contains? vowels \e)))
    "#);
    assert!(has_e, "HashSet<Char>: \\e must be found in vowels set");

    // \z is NOT in the vowels set.
    let has_z = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [vowels (:wat::core::HashSet :wat::core::Char \a \e \i \o \u)
                       found  (:wat::core::contains? vowels \z)]
                      (:wat::core::not found)))
    "#);
    assert!(has_z, "HashSet<Char>: \\z must NOT be found in vowels set");

    // Length = 5 — five distinct vowel Char elements.
    let len = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [vowels (:wat::core::HashSet :wat::core::Char \a \e \i \o \u)]
                      (:wat::core::HashSet/length vowels)))
    "#);
    assert_eq!(len, 5, "HashSet<Char>: five distinct vowels → length 5");
}
