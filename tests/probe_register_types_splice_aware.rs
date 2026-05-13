//! Arc 170 slice 3 Gap J — regression probes for `register_types` splice-awareness.
//!
//! `register_types` previously only processed type declarations (struct/enum/newtype/
//! typealias) at the TOP LEVEL of the form list. When a top-level
//! `(:wat::core::do ...)` or `(:wat::core::let ...)` form contained type declarations
//! in its body, those declarations were ABSENT from `TypeEnv` after startup — causing
//! `expand_alias` failures, match-scrutinee inference failures, and child-process type
//! registry misses.
//!
//! Gap J extends `register_types` and `register_stdlib_types` to recurse into top-level
//! do/let body forms and register any type declarations found there, mirroring the
//! splice-recursion pattern already used by `preregister_fn_defs_in_do`/`_in_let`
//! (runtime.rs) for function definitions.
//!
//! These probes prove the fix directly: each creates a minimal source with a type
//! declaration nested in a top-level do/let, then asserts the type appears in the
//! TypeEnv after startup.
//!
//! All probes FAIL before Gap J ships; all PASS after.
//!
//! Honest delta (probe design): top-level do bodies with ONLY type declarations
//! produce an empty-body do after stripping — which the check_program validator
//! correctly rejects ("do form requires at least one form; got zero"). In practice,
//! deftest's macro expansion always includes a (:define ...) alongside the type
//! decls, so this degenerate case never arises in real code. Probes below include
//! a minimal non-type-decl body form to stay in the valid-do shape.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

// ─── do-body typealias ──────────────────────────────────────────────────────

/// Typealias nested in top-level do lands in TypeEnv.
/// Before Gap J: TypeEnv.get(":diag::MyAlias") → None; expand_alias fails.
/// After Gap J: TypeEnv.get(":diag::MyAlias") → Some(TypeDef::Alias).
///
/// The `define` alongside the typealias keeps the do body non-empty (valid-do
/// shape); this also exercises the primary failure from Phase E V5 Pattern A.
#[test]
fn do_typealias_registers_in_type_env() {
    let src = r#"
        (:wat::core::do
          (:wat::core::typealias :diag::MyAlias :wat::core::i64)
          (:wat::core::define (:diag::alias-probe -> :diag::MyAlias)
            42))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup failed");
    assert!(
        world.types().get(":diag::MyAlias").is_some(),
        ":diag::MyAlias must be registered in TypeEnv after Gap J"
    );
    assert!(
        world.symbols().get(":diag::alias-probe").is_some(),
        ":diag::alias-probe must be registered"
    );
}

// ─── do-body struct ─────────────────────────────────────────────────────────

/// Struct nested in top-level do lands in TypeEnv.
/// Includes a define body to keep the do valid; the define references the struct
/// constructor, verifying both TypeEnv registration AND accessor availability.
#[test]
fn do_struct_registers_in_type_env() {
    let src = r#"
        (:wat::core::do
          (:wat::core::struct :diag::Point
            (x :wat::core::i64)
            (y :wat::core::i64))
          (:wat::core::define (:diag::origin -> :diag::Point)
            (:diag::Point/new 0 0)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup failed");
    assert!(
        world.types().get(":diag::Point").is_some(),
        ":diag::Point must be registered in TypeEnv after Gap J"
    );
    assert!(
        world.symbols().get(":diag::Point/new").is_some(),
        ":diag::Point/new accessor stub must be present"
    );
    assert!(
        world.symbols().get(":diag::origin").is_some(),
        ":diag::origin must be registered"
    );
}

// ─── do-body newtype ─────────────────────────────────────────────────────────

/// Newtype nested in top-level do lands in TypeEnv.
/// Newtype is NOMINAL — `:diag::UserId` is distinct from its inner `:i64`.
/// The probe verifies TypeEnv registration only; the define body returns
/// `:wat::core::nil` (unit) and the newtype is referenced only as a type
/// annotation for an argument, not the return type (to avoid type mismatch).
#[test]
fn do_newtype_registers_in_type_env() {
    let src = r#"
        (:wat::core::do
          (:wat::core::newtype :diag::UserId :wat::core::i64)
          (:wat::core::define (:diag::uses-user-id -> :wat::core::nil)
            :wat::core::nil))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup failed");
    assert!(
        world.types().get(":diag::UserId").is_some(),
        ":diag::UserId must be registered in TypeEnv after Gap J"
    );
}

// ─── do-body enum ────────────────────────────────────────────────────────────

/// Enum nested in top-level do lands in TypeEnv.
/// Unit variants use keyword syntax `:Red` (not bare symbol).
/// Probe verifies TypeEnv registration only; the enum constructor stubs are
/// handled by preregister_enum_constructors_from_form (Gap F-1, already shipped).
#[test]
fn do_enum_registers_in_type_env() {
    let src = r#"
        (:wat::core::do
          (:wat::core::enum :diag::Color
            :Red
            :Green
            :Blue)
          (:wat::core::define (:diag::something -> :wat::core::i64)
            42))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup failed");
    assert!(
        world.types().get(":diag::Color").is_some(),
        ":diag::Color must be registered in TypeEnv after Gap J"
    );
}

// ─── let-body typealias ──────────────────────────────────────────────────────

/// Typealias nested in top-level let body (items[2..]) lands in TypeEnv.
/// Arc 168 multi-form body: items[0]=:let, items[1]=bindings, items[2..]=body.
#[test]
fn let_body_typealias_registers() {
    let src = r#"
        (:wat::core::let
          []
          (:wat::core::typealias :diag::LetAlias :wat::core::i64))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup failed");
    assert!(
        world.types().get(":diag::LetAlias").is_some(),
        ":diag::LetAlias must be registered from let-body after Gap J"
    );
}

// ─── nested do-within-do ─────────────────────────────────────────────────────

/// Typealias nested in do-within-do registers.
/// Verifies recursive termination: `(:do (:do (:typealias :A :i64) <body>))`.
/// The inner do must have a non-type-decl body form to be valid.
#[test]
fn nested_do_typealias_registers() {
    let src = r#"
        (:wat::core::do
          (:wat::core::do
            (:wat::core::typealias :diag::NestedAlias :wat::core::i64)
            (:wat::core::define (:diag::nested-probe -> :diag::NestedAlias)
              99)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup failed");
    assert!(
        world.types().get(":diag::NestedAlias").is_some(),
        ":diag::NestedAlias must be registered from do-within-do after Gap J"
    );
}

// ─── end-to-end: typealias in do + function returning that alias ─────────────

/// End-to-end proof: typealias declared in top-level do can be used as a
/// function return type annotation. This is the primary failure pattern from
/// Phase E V5 (Pattern A — typealias unification).
///
/// Before Gap J: `expand_alias(types, ":diag::Score")` returns `:diag::Score`
/// unchanged (not in TypeEnv); unification against `:wat::core::i64` fails.
/// After Gap J: alias registered → expand_alias resolves → unification passes.
#[test]
fn do_typealias_usage_typechecks() {
    let src = r#"
        (:wat::core::do
          (:wat::core::typealias :diag::Score :wat::core::i64)
          (:wat::core::define (:diag::make-score -> :diag::Score)
            42))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("do_typealias_usage_typechecks: startup should succeed after Gap J");
    assert!(
        world.symbols().get(":diag::make-score").is_some(),
        ":diag::make-score must be registered"
    );
    assert!(
        world.types().get(":diag::Score").is_some(),
        ":diag::Score must be in TypeEnv"
    );
}
