//! FM 2-bis probe — arc 251 Stone 251.3: a parametric type written as a FORM
//! `(wat.type/Vector wat.type/i64)` type-checks like the `<>` keyword spelling.
//!
//! 251.3 moves parametric types from the `<>` keyword surface
//! (`:wat::core::Vector<wat::core::i64>`) to s-expr FORMS
//! (`(wat.type/Vector wat.type/i64)` — a `WatAST::List`). The type-slot readers
//! (argspec/parse.rs:187, function/parse.rs:170, types.rs:1834/1873/1940) accept
//! ONLY `WatAST::Keyword` today, so a List in a type slot is rejected.
//!
//! HEAD-disconfirmation:
//! - C01: a parametric FORM `(wat.type/Vector wat.type/i64)` in a binder slot
//!   ⇒ FAILS at HEAD. The type slot expects a keyword; a List falls through to the
//!     "expected type keyword" error. Load-bearing: the form's param is passed to a
//!     sink fn typed with the `<>` keyword spelling, so the form must produce the
//!     SAME `Parametric` for unification to succeed (not merely parse to something).
//! - C02: the `<>` keyword spelling `:wat::core::Vector<wat::core::i64>` STILL
//!   checks (PRESERVATION — dual-read; `<>` lexer + corpus retire at the 251.5 sweep).
//!
//! Post-251.3a: both contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc251_stone3_parametric_form`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn checks(decls: &str) -> Result<(), String> {
    let src = format!("{decls}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

// ─── C01: THE GAP — a parametric type FORM type-checks ──────────────────────────

#[test]
fn contract_01_parametric_form_type_checks() {
    // `(wat.type/Vector wat.type/i64)` must produce the SAME parametric type as the
    // `<>` keyword spelling — proven load-bearing by passing the form-typed param to
    // a sink fn typed with the keyword spelling (unification must succeed).
    // At HEAD: the binder type slot rejects a List → RED. Post-251.3a: GREEN.
    let ok = checks(
        "(:wat::core::defn :user::sink [v <- :wat::core::Vector<wat::core::i64>] \
           -> :wat::core::i64 0)\n\
         (:wat::core::defn :user::pass [xs <- (wat.type/Vector wat.type/i64)] \
           -> :wat::core::i64 (:user::sink xs))",
    )
    .is_ok();
    assert!(
        ok,
        "(wat.type/Vector wat.type/i64) must type-check as Vector<i64> (unify with the keyword spelling)"
    );
}

// ─── C03: the pre-normalize Symbol-head path (register_types, step 5) ───────────

#[test]
fn contract_03_parametric_form_in_type_declaration() {
    // A `typealias` declared with a parametric FORM exercises the type-DECLARATION
    // reader (types.rs parse_typealias), which runs at register_types (freeze step 5)
    // — BEFORE normalize (step 7). So the form arrives SYMBOL-headed
    // (`wat.type/Vector`, not `:wat::type::Vector`): this is the path that makes the
    // Symbol arms of parse_type_form / parse_type_node live, not dead. Load-bearing:
    // the alias is passed to a `<>`-keyword Vector<i64> sink, so the form-declared
    // alias must resolve to the SAME Parametric.
    let ok = checks(
        "(:wat::core::typealias :user::IntVec (wat.type/Vector wat.type/i64))\n\
         (:wat::core::defn :user::sink [v <- :wat::core::Vector<wat::core::i64>] \
           -> :wat::core::i64 0)\n\
         (:wat::core::defn :user::pass [xs <- :user::IntVec] -> :wat::core::i64 \
           (:user::sink xs))",
    )
    .is_ok();
    assert!(
        ok,
        "a typealias declared with (wat.type/Vector wat.type/i64) must resolve to Vector<i64> \
         (exercises the pre-normalize Symbol-head parse path)"
    );
}

// ─── C02: PRESERVATION — the `<>` keyword spelling still checks ──────────────────

#[test]
fn contract_02_angle_bracket_spelling_still_checks() {
    // The `<>` parametric keyword keeps working while the corpus migrates (dual-read;
    // `<>` lexer machinery + corpus retire at the 251.5 unified sweep). GREEN at HEAD.
    assert!(
        checks(
            "(:wat::core::defn :user::id [v <- :wat::core::Vector<wat::core::i64>] \
               -> :wat::core::i64 0)"
        )
        .is_ok(),
        ":wat::core::Vector<wat::core::i64> keyword spelling must keep type-checking"
    );
}
