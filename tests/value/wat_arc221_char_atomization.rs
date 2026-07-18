//! Arc 221 Stone 221.2 — `:wat::core::char` atomization probes.
//!
//! Verifies that `Value::wat__core__Char` is fully atomizable end-to-end:
//!   (a) `(:wat::holon::Atom \c)` dispatches via `value_to_atom` Char arm to
//!       `HolonAST::char_(c)` (Stone 221.2 — holon-rs commit `243eded`).
//!   (b) `is_atomizable(":wat::core::char")` returns true (Stone 221.2 predicate
//!       extension), so the check layer approves `HashMap<Char, V>` and `HashSet<Char>`.
//!   (c) Char is usable as a HashMap key and HashSet element at the WAT runtime level.
//!
//! ## Tests
//!
//!  1 — `(:wat::holon::to-holon \a)` round-trip; atom(Char) distinct from atom(i64)
//!  2 — `HashMap<Char, i64>` insert + lookup (char-frequency-tally pattern)
//!  3 — `HashSet<Char>` insert + contains? (vowels-set pattern)
//!
//! Wat source lives in the co-located fixture: wat_arc221_char_atomization.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `:t::…` fixture fn is a zero-arg entry; fetch it from the frozen
// world and `apply_function` it — no inline wat driver.
fn call0(world: &wat::freeze::FrozenWorld, fn_name: &str) -> Value {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

fn run_bool(world: &wat::freeze::FrozenWorld, fn_name: &str) -> bool {
    match call0(world, fn_name) {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn run_i64(world: &wat::freeze::FrozenWorld, fn_name: &str) -> i64 {
    match call0(world, fn_name) {
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
    let world = startup_beside(file!()).expect("startup");

    // atom(\a) = atom(\a) — same Char produces identical HolonAST.
    let same = run_bool(&world, ":t::p1-same");
    assert!(same, "Atom(\\a) must equal Atom(\\a) — same Char leaf");

    // atom(\a) ≠ atom(\b) — different Chars produce distinct HolonAST.
    let diff = run_bool(&world, ":t::p1-diff");
    assert!(diff, "Atom(\\a) must NOT equal Atom(\\b) — distinct Char leaves");

    // atom(\a) ≠ atom(97) — Char leaf is distinct from i64 leaf.
    let not_i64 = run_bool(&world, ":t::p1-not-i64");
    assert!(not_i64, "Atom(\\a) must NOT equal Atom(97) — Char leaf is distinct from i64 leaf");
}

// ─── Probe 2 — `HashMap<Char, i64>` insert + lookup ─────────────────────────

/// `HashMap<Char, i64>` with Char keys — char-frequency-tally pattern.
///
/// Verifies that `is_atomizable(Char)` returns true at the check layer
/// (enabling `HashMap<Char, V>` as atomizable K), and that Char values can
/// serve as HashMap keys at runtime (Hash + Eq impls from Stone 220.2).
///
/// Arc 221 Stone 221.2 — `is_atomizable` Char extension enables HashMap<Char,V>.
#[test]
fn probe_2_hashmap_char_key_insert_lookup() {
    let world = startup_beside(file!()).expect("startup");

    // Insert \a -> 3, \b -> 7; get \a returns Some(3).
    let a_val = run_i64(&world, ":t::p2-a-val");
    assert_eq!(a_val, 3, "HashMap<Char,i64>: get \\a after insert must return 3");

    // get \b returns Some(7).
    let b_val = run_i64(&world, ":t::p2-b-val");
    assert_eq!(b_val, 7, "HashMap<Char,i64>: get \\b after insert must return 7");

    // length = 2 — two distinct Char keys.
    let len = run_i64(&world, ":t::p2-len");
    assert_eq!(len, 2, "HashMap<Char,i64>: two distinct Char keys → length 2");
}

// ─── Probe 3 — `HashSet<Char>` insert + contains? ────────────────────────────

/// `HashSet<Char>` — vowels-set pattern.
///
/// Verifies that `is_atomizable(Char)` at the check layer permits
/// `HashSet<Char>` as an atomizable element type, and that Char values
/// serve as HashSet elements at runtime (Hash + Eq impls from Stone 220.2).
///
/// Arc 221 Stone 221.2 — `is_atomizable` Char extension enables HashSet<Char>.
#[test]
fn probe_3_hashset_char_insert_contains() {
    let world = startup_beside(file!()).expect("startup");

    // \a is in the vowels set.
    let has_a = run_bool(&world, ":t::p3-has-a");
    assert!(has_a, "HashSet<Char>: \\a must be found in vowels set");

    // \e is in the vowels set.
    let has_e = run_bool(&world, ":t::p3-has-e");
    assert!(has_e, "HashSet<Char>: \\e must be found in vowels set");

    // \z is NOT in the vowels set.
    let no_z = run_bool(&world, ":t::p3-no-z");
    assert!(no_z, "HashSet<Char>: \\z must NOT be found in vowels set");

    // Length = 5 — five distinct vowel Char elements.
    let len = run_i64(&world, ":t::p3-len");
    assert_eq!(len, 5, "HashSet<Char>: five distinct vowels → length 5");
}
