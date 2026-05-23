//! Arc 214 Slice 4 Stone 4.1 — `:wat::program::Env` typealias probes.
//!
//! Verifies that:
//! - `:wat::program::Env` is registered as a typealias for `HashMap<keyword, HolonAST>`
//! - `parse_type_expr(":wat::program::Env")` returns Ok
//! - `expand_alias` resolves the alias to the underlying Parametric form
//! - A function declaring `:wat::program::Env` param type-checks
//! - `{:foo (:wat::holon::to-holon 42)}` (explicit HolonAST value) unifies with the param
//! - `{}` (empty map) unifies with the param
//! - `{:foo "string"}` (V = String, not HolonAST) fails at check with TypeMismatch
//!
//! ## The 6 probes
//!
//! 1. `parse_type_expr(":wat::program::Env")` returns `Ok(...)`
//! 2. `expand_alias` resolves `:wat::program::Env` to `Parametric { head: "wat::core::HashMap", args: [keyword, HolonAST] }`
//! 3. Function signature `(m :wat::program::Env) -> :wat::core::nil` type-checks cleanly
//! 4. Calling with `{:foo (:wat::holon::to-holon 42)}` (V infers HolonAST) type-checks
//! 5. Calling with `{}` (empty literal; HM unification fills K + V from param) type-checks
//! 6. Calling with `{:foo "string"}` (V = String) fails at check with TypeMismatch

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::types::{expand_alias, parse_type_expr, TypeEnv, TypeExpr};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn startup_ok(src: &str) {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup should succeed; got error:\n{}\n---\n{:?}", e, e));
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1: `parse_type_expr(":wat::program::Env")` returns Ok ─────────────

/// `:wat::program::Env` is a valid type-keyword; `parse_type_expr` must parse
/// it without error. This verifies that the parser accepts the FQDN before any
/// TypeEnv lookup.
#[test]
fn probe_1_parse_type_expr_ok() {
    let result = parse_type_expr(":wat::program::Env");
    assert!(
        result.is_ok(),
        "parse_type_expr(\":wat::program::Env\") must return Ok; got {:?}",
        result
    );
    // The parse result is a Path — the alias lookup happens at check time, not parse time.
    match result.unwrap() {
        TypeExpr::Path(p) => assert_eq!(
            p, ":wat::program::Env",
            "parsed path must be :wat::program::Env"
        ),
        other => panic!("expected Path; got {:?}", other),
    }
}

// ─── Probe 2: `expand_alias` resolves to underlying HashMap parametric ────────

/// `expand_alias(:wat::program::Env, builtin_env)` must return
/// `Parametric { head: "wat::core::HashMap", args: [keyword, HolonAST] }`.
/// Uses `TypeEnv::with_builtins()` which runs `register_builtin_types` —
/// the same code that registers `:wat::program::Env`.
#[test]
fn probe_2_expand_alias_resolves_to_hashmap_parametric() {
    let env = TypeEnv::with_builtins();
    let alias_expr = TypeExpr::Path(":wat::program::Env".into());
    let expanded = expand_alias(&alias_expr, &env);

    match &expanded {
        TypeExpr::Parametric { head, args } => {
            assert_eq!(
                head, "wat::core::HashMap",
                "expand_alias head must be wat::core::HashMap; got {}",
                head
            );
            assert_eq!(
                args.len(),
                2,
                "expand_alias must have 2 args (K, V); got {} args",
                args.len()
            );
            assert_eq!(
                args[0],
                TypeExpr::Path(":wat::core::keyword".into()),
                "K arg must be :wat::core::keyword; got {:?}",
                args[0]
            );
            assert_eq!(
                args[1],
                TypeExpr::Path(":wat::holon::HolonAST".into()),
                "V arg must be :wat::holon::HolonAST; got {:?}",
                args[1]
            );
        }
        other => panic!(
            "expand_alias(:wat::program::Env) must return Parametric; got {:?}",
            other
        ),
    }
}

// ─── Probe 3: function signature with `:wat::program::Env` param type-checks ─

/// Declaring a function that accepts `:wat::program::Env` and returns
/// `:wat::core::nil` must type-check cleanly.
/// Calling it with a keyword-keyed HolonAST map `{:foo (:wat::holon::to-holon 42)}`
/// is NOT in this probe — this probe checks only the definition.
#[test]
fn probe_3_function_signature_accepts_program_env() {
    let src = r#"
        (:wat::core::define (:user::take-env (m :wat::program::Env) -> :wat::core::nil)
          :wat::core::nil)
        (:wat::core::define (:user::compute -> :wat::core::nil)
          (:user::take-env {}))
    "#;
    startup_ok(src);
}

// ─── Probe 4: `{:foo (:wat::holon::to-holon 42)}` unifies with `:wat::program::Env` ─

/// Calling `take-env` with `{:foo (:wat::holon::to-holon 42)}`:
/// - K is inferred as :wat::core::keyword from :foo
/// - V is inferred as :wat::holon::HolonAST from `(:wat::holon::to-holon 42)`
/// - The inferred type unifies with the param type :wat::program::Env
///   (which expands to HashMap<keyword, HolonAST>)
/// Should type-check without error.
#[test]
fn probe_4_explicit_atom_literal_accepted() {
    let src = r#"
        (:wat::core::define (:user::take-env (m :wat::program::Env) -> :wat::core::nil)
          :wat::core::nil)
        (:wat::core::define (:user::compute -> :wat::core::nil)
          (:user::take-env {:foo (:wat::holon::to-holon 42)}))
    "#;
    startup_ok(src);
}

// ─── Probe 5: `{}` empty literal unifies with `:wat::program::Env` ───────────

/// Calling `take-env` with `{}` (empty map literal):
/// - K and V are fresh type variables (no concrete values to infer from)
/// - HM unification resolves K → keyword, V → HolonAST from the param sig
/// Should type-check without error.
#[test]
fn probe_5_empty_map_literal_accepted() {
    let src = r#"
        (:wat::core::define (:user::take-env (m :wat::program::Env) -> :wat::core::nil)
          :wat::core::nil)
        (:wat::core::define (:user::compute -> :wat::core::nil)
          (:user::take-env {}))
    "#;
    startup_ok(src);
}

// ─── Probe 6: `{:foo "string"}` (V = String) rejected at check ───────────────

/// Calling `take-env` with `{:foo "string"}`:
/// - K is inferred as :wat::core::keyword from :foo (OK)
/// - V is inferred as :wat::core::String from "string"
/// - The param type demands V = :wat::holon::HolonAST
/// → TypeMismatch: String ≠ HolonAST
#[test]
fn probe_6_wrong_value_type_rejected_with_type_mismatch() {
    let src = r#"
        (:wat::core::define (:user::take-env (m :wat::program::Env) -> :wat::core::nil)
          :wat::core::nil)
        (:wat::core::define (:user::compute -> :wat::core::nil)
          (:user::take-env {:foo "string"}))
    "#;
    let err = startup_err(src);
    assert!(
        err.to_lowercase().contains("typemismatch")
            || err.to_lowercase().contains("type mismatch")
            || err.contains("TypeMismatch"),
        "{{:foo \"string\"}} must fail with TypeMismatch (V=String ≠ HolonAST); got:\n{}",
        err
    );
}
