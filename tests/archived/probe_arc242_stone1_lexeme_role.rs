//! FM 2-bis probe for Stone 242.1 — bare `nil` lexer + `:wat::core::Char` HARD CUT + doctrine inscription.
//!
//! Arc 242 inscribes the lexeme-role-doctrine:
//! - Doctrine 1: bare lexeme = value; keyword lexeme `:wat::core::*` = type
//! - Doctrine 2: scalar types lowercase; non-scalar/container types PascalCase
//!
//! HEAD-disconfirmation map:
//! - C01: bare `nil` works as a primitive VALUE in expression position
//!   ⇒ FAILS at HEAD (bare `nil` parses as symbol, not primitive value;
//!      using bare `nil` where the type is `:wat::core::nil` may type-mismatch)
//! - C02: `:wat::core::nil` STILL works as TYPE in signature position
//!   (preservation contract; ensures we didn't break type semantics)
//! - C03: `:wat::core::Char` HARD-CUT-rejected with structured retirement remedy
//!   ⇒ FAILS at HEAD (`:wat::core::Char` is the active type name; no rejection)
//! - C04: `:wat::core::char` (lowercase) works as type
//!   ⇒ FAILS at HEAD (`:wat::core::char` doesn't exist as a type name)
//!
//! Post-stone: all 4 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc242_stone1_lexeme_role`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn try_startup(src: &str) -> Result<(), String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn try_startup_display(src: &str) -> String {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    match startup_from_source(&full, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── C01: bare `nil` works as primitive value in expression position ───────────

#[test]
fn contract_01_bare_nil_works_as_value() {
    // A function returning :wat::core::nil with body bare `nil` must work post-stone.
    // At HEAD: bare `nil` is a SYMBOL; the symbol may not match the :wat::core::nil
    // return type, causing ReturnTypeMismatch.
    // Post-stone: bare `nil` is the primitive nil value; matches the type.
    let src = r#"
        (:wat::core::defn :test::returns-nil [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "bare nil should work as primitive value in expression position; got: {:?}",
        result
    );
}

// ─── C02: :wat::core::nil PRESERVED as type in signature ───────────────────────

#[test]
fn contract_02_keyword_nil_preserved_as_type() {
    // :wat::core::nil in signature position must still work post-stone (Doctrine 1
    // says keyword form is type; primitive nil type lowercase per Doctrine 2).
    // This is a preservation contract — was already working; must NOT break.
    let src = r#"
        (:wat::core::defn :test::accepts-nil [x <- :wat::core::nil] -> :wat::core::nil x)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        ":wat::core::nil must still work as type in signature position; got: {:?}",
        result
    );
}

// ─── C03: :wat::core::Char HARD-CUT-rejected with retirement remedy ────────────

#[test]
fn contract_03_legacy_char_hard_cut_with_remedy() {
    // :wat::core::Char retires per Doctrine 2 (scalar types lowercase).
    // Post-stone: HARD CUT rejection with structured retirement remedy
    // pointing at :wat::core::char.
    // At HEAD: :wat::core::Char works → no error → assertion fails.
    let src = r#"
        (:wat::core::defn :test::needs-char [c <- :wat::core::Char] -> :wat::core::Char c)
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains("did you mean") && msg.contains(":wat::core::char"),
        "retirement remedy must name :wat::core::char; got:\n{}",
        msg
    );
    assert!(
        msg.contains("[replaces a retired form]"),
        "retirement remedy must carry '[replaces a retired form]' annotation; got:\n{}",
        msg
    );
}

// ─── C04: :wat::core::char (lowercase) works as type ───────────────────────────

#[test]
fn contract_04_lowercase_char_works_as_type() {
    // :wat::core::char (lowercase) is the live char type post-stone.
    // At HEAD: :wat::core::char doesn't exist; startup errors.
    // Post-stone: works as scalar primitive type per Doctrine 2.
    let src = r#"
        (:wat::core::defn :test::needs-char [c <- :wat::core::char] -> :wat::core::char c)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        ":wat::core::char (lowercase) should work as type post-stone; got: {:?}",
        result
    );
}
