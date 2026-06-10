//! FM 2-bis probe — arc 251 Stone 251.2: a `wat.type/` type atom type-checks like
//! the `:wat::core::` keyword it replaces.
//!
//! 251.2 moves scalar type ATOMS out of the value/keyword space into a symbol
//! namespace: `:wat::core::i64` (Keyword, dual-role) → `wat.type/i64` (Symbol). The
//! 251.1b normalize-layer already rewrites any `WatAST::Symbol` with `/` to its
//! keyword FQDN, so `wat.type/i64` normalizes to `:wat::type::i64`. The ONLY new
//! substrate work is teaching the type parser the `:wat::type::` namespace.
//!
//! HEAD-disconfirmation:
//! - C01: a `wat.type/i64` type annotation (binder + return position) type-checks
//!   ⇒ FAILS at HEAD. normalize rewrites `wat.type/i64` → `:wat::type::i64`, but
//!     `parse_type_expr_with_span` (types.rs:2185) only knows `:wat::core::i64` /
//!     bare `:i64` — `:wat::type::i64` is an unknown type path ⇒ check error.
//! - C02: the legacy `:wat::core::i64` spelling STILL type-checks
//!   (PRESERVATION — dual-read through the corpus migration; keyword type spellings
//!    HARD-CUT only at 251.5).
//!
//! Post-251.2a: both contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc251_stone2_type_namespace`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Freeze + check `decls` (plus a legacy-spelled main). Ok(()) iff the whole
/// program type-checks clean.
fn checks(decls: &str) -> Result<(), String> {
    let src = format!("{decls}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

// ─── C01: THE GAP — a `wat.type/` type atom type-checks ─────────────────────────

#[test]
fn contract_01_wat_type_atom_type_checks() {
    // `wat.type/i64` must be RECOGNIZED AS i64 — not merely accepted as some opaque
    // nominal path. The body does i64 arithmetic on the param, so the annotation must
    // resolve to the i64 scalar for `:wat::core::i64::+` to accept `x`. At HEAD:
    // normalize → `:wat::type::i64`, which `parse_type_expr` does not map to i64
    // (it falls through to an unknown/nominal path) → the `+` rejects x → RED.
    // Post-251.2a: `:wat::type::i64` IS i64 → GREEN.
    assert!(
        checks(
            "(:wat::core::defn :user::inc [x <- wat.type/i64] -> wat.type/i64 \
               (:wat::core::i64::+ x 1))"
        )
        .is_ok(),
        "wat.type/i64 must be recognized as i64 (the body does i64 arithmetic)"
    );
}

// ─── C02: PRESERVATION — legacy keyword type spelling still checks ──────────────

#[test]
fn contract_02_legacy_keyword_type_still_checks() {
    // The `:wat::core::i64` spelling keeps working while the corpus migrates
    // (dual-read; HARD-CUT only at 251.5). GREEN at HEAD; must NOT regress.
    assert!(
        checks("(:wat::core::defn :user::id [x <- :wat::core::i64] -> :wat::core::i64 x)").is_ok(),
        ":wat::core::i64 keyword type must keep type-checking during the transition"
    );
}

// ─── C03: the alias is GENERAL across the scalar atom set ───────────────────────

#[test]
fn contract_03_wat_type_atoms_across_scalars() {
    // The `:wat::type::` → `:wat::core::` canonicalization is general, not i64-only.
    // Exercise f64, bool, and String through `wat.type/` with type-identity load-
    // bearing bodies (each op demands the param actually BE that scalar).
    assert!(
        checks("(:wat::core::defn :user::fadd [x <- wat.type/f64] -> wat.type/f64 \
                  (:wat::core::f64::+ x 1.0))")
        .is_ok(),
        "wat.type/f64 must be recognized as f64"
    );
    assert!(
        checks("(:wat::core::defn :user::neg [b <- wat.type/bool] -> wat.type/bool \
                  (:wat::core::not b))")
        .is_ok(),
        "wat.type/bool must be recognized as bool"
    );
    // String: a `:wat::core::String`-typed sink fn that `s` must unify against —
    // load-bearing without depending on any builtin's signature.
    assert!(
        checks("(:wat::core::defn :user::sink [t <- :wat::core::String] -> :wat::core::i64 0)\n\
                (:wat::core::defn :user::pass [s <- wat.type/String] -> :wat::core::i64 \
                  (:user::sink s))")
        .is_ok(),
        "wat.type/String must be recognized as String (it must unify with a String param)"
    );
}
