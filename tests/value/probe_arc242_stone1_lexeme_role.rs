//! FM 2-bis probe for Stone 242.1 — bare `nil` lexer + `:wat::core::Char` HARD CUT + doctrine inscription.
//!
//! Arc 242 inscribes the lexeme-role-doctrine:
//! - Doctrine 1: bare lexeme = value; keyword lexeme `:wat::core::*` = type
//! - Doctrine 2: scalar types lowercase; non-scalar/container types PascalCase
//!
//! HEAD-disconfirmation map:
//! - C01: bare `nil` works as a primitive VALUE in expression position
//!   ⇒ FAILS at HEAD (bare `nil` parses as symbol, not primitive value;
//!   using bare `nil` where the type is `:wat::core::nil` may type-mismatch)
//! - C02: `:wat::core::nil` STILL works as TYPE in signature position
//!   (preservation contract; ensures we didn't break type semantics)
//! - C03: `:wat::core::Char` HARD-CUT-rejected with structured retirement remedy
//!   ⇒ FAILS at HEAD (`:wat::core::Char` is the active type name; no rejection)
//! - C04: `:wat::core::char` (lowercase) works as type
//!   ⇒ FAILS at HEAD (`:wat::core::char` doesn't exist as a type name)
//!
//! Post-stone: all 4 contracts PASS.
//!
//! Wat source for C01/C02/C04: co-located probe_arc242_stone1_lexeme_role.wat
//! (slurped via startup_beside(file!())); startup SUCCESS is the assertion.
//! Negative fixture for C03: probe_arc242_stone1_lexeme_role.wat.bad
//! (loaded via startup_from_file; startup FAILURE + retirement-remedy message is the assertion).

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};

// ─── C01: bare `nil` works as primitive value in expression position ───────────

#[test]
fn contract_01_bare_nil_works_as_value() {
    // The co-located fixture defines :test::returns-nil with body bare `nil`.
    // Startup success proves bare nil is accepted as a primitive value.
    // At HEAD pre-stone: bare `nil` parses as symbol, causing ReturnTypeMismatch.
    // Post-stone: bare `nil` is the primitive nil value; matches :wat::core::nil type.
    startup_beside(file!()).expect(
        "bare nil should work as primitive value in expression position",
    );
}

// ─── C02: :wat::core::nil PRESERVED as type in signature ───────────────────────

#[test]
fn contract_02_keyword_nil_preserved_as_type() {
    // The co-located fixture defines :test::accepts-nil [x <- :wat::core::nil].
    // Startup success proves keyword form still works as type in signature.
    // This is a preservation contract — was already working; must NOT break.
    startup_beside(file!()).expect(
        ":wat::core::nil must still work as type in signature position",
    );
}

// ─── C03: :wat::core::Char HARD-CUT-rejected with retirement remedy ────────────

#[test]
fn contract_03_legacy_char_hard_cut_with_remedy() {
    // The _bad fixture defines :test::needs-char with :wat::core::Char (uppercase).
    // :wat::core::Char retires per Doctrine 2 (scalar types lowercase).
    // Post-stone: HARD CUT rejection with structured retirement remedy pointing at :wat::core::char.
    // At HEAD: :wat::core::Char works → no error → assertion fails.
    let result = startup_from_file("tests/value/probe_arc242_stone1_lexeme_role.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::Char"
            && reason == "':wat::core::Char' is retired (Stone 242.1); use ':wat::core::char' \
                instead (scalar types lowercase per arc 242 Doctrine 2)"
    );
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(msg, "probe_arc242_stone1_lexeme_role__contract_03_legacy_char_hard_cut_with_remedy.edn", "retirement remedy must carry exact golden");
}

// ─── C04: :wat::core::char (lowercase) works as type ───────────────────────────

#[test]
fn contract_04_lowercase_char_works_as_type() {
    // The co-located fixture defines :test::needs-char-lowercase with :wat::core::char.
    // Startup success proves lowercase char type is accepted post-stone.
    // At HEAD pre-stone: :wat::core::char doesn't exist; startup errors.
    startup_beside(file!()).expect(
        ":wat::core::char (lowercase) should work as type post-stone",
    );
}
