//! Arc 220 slice 2 — `:wat::core::char` typed primitive (Stone 242.1: renamed from Char).
//!
//! Verifies the `Value::wat__core__Char` variant and the `char/of` constructor.
//! Also verifies the lexer `\c` literal syntax and BMP-only enforcement.
//!
//! Test cases:
//!   1 — Lexer accepts `\a` single-char literal
//!   2 — Lexer accepts named chars: `\newline`, `\space`, `\tab`, `\return`
//!   3 — Lexer accepts `\uNNNN` Unicode escape (BMP)
//!   4 — Lexer rejects supplementary-plane literal (produces diagnostic)
//!   5 — `(:wat::core::char "x")` returns Value::wat__core__Char('x')
//!   6 — `(:wat::core::char "")` errors with "length-1" diagnostic
//!   7 — `(:wat::core::char "ab")` errors with "length-2" diagnostic
//!   8 — `(:wat::core::char "\u{1F600}")` errors with "supplementary-plane"
//!   9 — Round-trip: `\x` in wat source → Value → EDN write → reparse → identical
//!  10 — `(= \a \a)` true; `(= \a \b)` false
//!
//! Wat source lives in the co-located fixture: wat_arc220_char.wat
//! (slurped via startup_beside(file!())).
//! Test 4 uses: tests/value/wat_arc220_char_supplementary_plane.wat.bad (negative — fails at lex time).

use wat::freeze::{startup_beside, startup_from_file};
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

fn run_expecting_runtime_err(world: &wat::freeze::FrozenWorld, fn_name: &str) -> String {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"))
        .clone();
    let err = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect_err("expected runtime error");
    format!("{:?}", err)
}

// ─── 1: Lexer accepts `\a` single-char literal ──────────────────────────────

/// The `\a` literal produces a `:wat::core::char` value (Stone 242.1).
/// Verified by using `(= \a (:wat::core::char "a"))` — if the lexer
/// produces the correct typed Char, both sides are equal.
#[test]
fn char_literal_single_letter() {
    let world = startup_beside(file!()).expect("startup");
    let ok = run_bool(&world, ":t::test1-char-literal-single-letter");
    assert!(ok, "\\a literal must produce Char('a')");
}

// ─── 2: Lexer accepts named chars ────────────────────────────────────────────

/// `\newline`, `\space`, `\tab`, `\return` named char forms.
/// Each compared against the String-form equivalents to verify content.
#[test]
fn char_literal_named_chars() {
    let world = startup_beside(file!()).expect("startup");
    let ok = run_bool(&world, ":t::test2-char-literal-named-chars");
    assert!(ok, "named char literals must produce correct Char values");
}

// ─── 3: Lexer accepts `\uNNNN` Unicode BMP escape ────────────────────────────

/// `A` (U+0041 = 'A') produces a Char equal to `(:wat::core::char "A")`.
#[test]
fn char_literal_unicode_escape() {
    let world = startup_beside(file!()).expect("startup");
    let ok = run_bool(&world, ":t::test3-char-literal-unicode-escape");
    assert!(ok, "\\u0041 must produce Char('A')");
}

// ─── 4: Lexer rejects supplementary-plane literal ────────────────────────────

/// A supplementary-plane char literal (e.g. `\😀`) must fail at lex time
/// with a diagnostic mentioning "supplementary-plane" or "BMP".
/// This tests that the lexer enforces BMP-only at the source level.
/// Uses the negative fixture: tests/value/wat_arc220_char_supplementary_plane.wat.bad
#[test]
fn char_literal_supplementary_plane_rejected() {
    let result = startup_from_file("tests/value/wat_arc220_char_supplementary_plane.wat.bad");
    // Must fail at startup (lex/parse time).
    assert!(
        result.is_err(),
        "supplementary-plane char literal must fail at lex time"
    );
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(msg, "wat_arc220_char__char_literal_supplementary_plane_rejected.edn", "error must be exact lex rejection golden");
}

// ─── 5: `(:wat::core::char "x")` returns typed Char ──────────────────────

/// `Char/of` with a valid single-char string constructs a typed Char.
/// We verify by equality with another Char/of call (proving the type is correct).
#[test]
fn char_of_valid_single_char() {
    let world = startup_beside(file!()).expect("startup");
    let ok = run_bool(&world, ":t::test5-char-of-valid-single-char");
    assert!(ok, "Char/of must return typed Char equal to same Char");
}

// ─── 6: `Char/of ""` errors with length diagnostic ───────────────────────────

/// Empty string is rejected with a clear "length-1" diagnostic.
/// Arc 221 Stone 221.2: Char/of is now type-registered as `String → Char`.
#[test]
fn char_of_empty_string_rejected() {
    let world = startup_beside(file!()).expect("startup");
    let err = run_expecting_runtime_err(&world, ":t::test6-char-of-empty");
    assert!(
        err.contains("length-1") || err.contains("empty"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "error must mention length-1 or empty: got {:?}", err
    );
}

// ─── 7: `Char/of "ab"` errors with length diagnostic ────────────────────────

/// Multi-char string is rejected with a clear "length" diagnostic.
/// Arc 221 Stone 221.2: Char/of is now type-registered as `String → Char`.
#[test]
fn char_of_multi_char_rejected() {
    let world = startup_beside(file!()).expect("startup");
    let err = run_expecting_runtime_err(&world, ":t::test7-char-of-multi");
    assert!(
        err.contains("length") || err.contains("got 2") || err.contains("length-2"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "error must mention length: got {:?}", err
    );
}

// ─── 8: `Char/of` with supplementary-plane char rejected ─────────────────────

/// A supplementary-plane char in a String arg is rejected with BMP diagnostic.
/// Arc 221 Stone 221.2: Char/of is now type-registered as `String → Char`.
/// The emoji 😀 (U+1F600) in a string literal passes the WAT lexer (string
/// literals are not BMP-restricted) but is rejected by char/of at runtime.
#[test]
fn char_of_supplementary_plane_rejected() {
    let world = startup_beside(file!()).expect("startup");
    let err = run_expecting_runtime_err(&world, ":t::test8-char-of-supplementary");
    assert!(
        err.contains("supplementary") || err.contains("BMP") || err.contains("1F600"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "error must mention supplementary-plane: got {:?}", err
    );
}

// ─── 9: Round-trip: Char/of → EDN write → parse → identical ─────────────────

/// `(:wat::core::char "x")` → EDN write → `(:wat::edn::read ...)` → typed Char.
/// Proves the EDN bridge is bidirectional for Char values.
#[test]
fn char_edn_round_trip() {
    let world = startup_beside(file!()).expect("startup");
    let ok = run_bool(&world, ":t::test9-char-edn-round-trip");
    assert!(ok, "Char must round-trip through EDN write/read");
}

// ─── 10: Equality ─────────────────────────────────────────────────────────────

/// `(= \a \a)` is true; `(= \a \b)` is false.
#[test]
fn char_equality() {
    let world = startup_beside(file!()).expect("startup");
    let ok = run_bool(&world, ":t::test10-char-equality");
    assert!(ok, "Char equality must be correct for same and different chars");
}
