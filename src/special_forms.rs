//! Arc 144 slice 2 — special-form registry.
//!
//! Special forms are syntactic constructs the type checker + runtime
//! handle directly (not via dispatch through Function or TypeScheme).
//! Examples: `:wat::core::if`, `let*`, `lambda`, `define`, `match`,
//! `quasiquote`, `try`, retired-but-poisoned heads like
//! `:wat::kernel::spawn`.
//!
//! This registry lets `:wat::runtime::lookup-form` (arc 144 slice 1)
//! return `Binding::SpecialForm` for each known form, exposing a
//! synthesized signature sketch the consumer (e.g., a future `(help
//! :if)` form) can render.
//!
//! Each entry carries the form's full keyword name + a synthesized
//! `HolonAST::Bundle` showing the syntax shape + a placeholder `None`
//! doc_string (arc 141 will populate it).
//!
//! # Sketch format
//!
//! Each `signature` is a `HolonAST::Bundle` whose first child is the
//! form's head as a Keyword (`HolonAST::keyword(":wat::core::if")`);
//! remaining children are bare-symbol placeholders for the syntactic
//! slots (`HolonAST::symbol("<cond>")`). Repeating slots use `<name>+`
//! (one or more) or `<name>*` (zero or more). The format is honest
//! about structure-not-types: each slot is a symbol naming the slot's
//! role, not a type. Consumers render this to a help string or AST.
//!
//! # Audit
//!
//! The registry's coverage was audited against:
//!   - `src/check.rs:2950-3430` — primary special-form dispatch in
//!     `infer_list` (heads that get non-scheme handling).
//!   - `src/runtime.rs:2400-2425` — runtime dispatch for the
//!     evaluation-side equivalents of those forms.
//!   - `src/freeze.rs:825-840` — definitional special forms handled
//!     at freeze (top-level only).
//!
//! Forms registered as TypeScheme primitives (e.g., `:wat::core::Vector`,
//! `:wat::kernel::spawn-thread`, `:wat::kernel::send`) do NOT appear
//! here — they are reachable through `lookup_form`'s Primitive branch
//! (slice 3 territory) instead. User-defined wat helpers like
//! `:wat::kernel::run-sandboxed-ast` (defined in `wat/std/sandbox.wat`)
//! reach through the UserFunction branch.

use holon::HolonAST;
use std::collections::HashMap;
use std::sync::OnceLock;

/// One special-form entry. Owned data — cloned out at lookup time.
pub struct SpecialFormDef {
    pub name: String,
    pub signature: HolonAST,
    pub doc_string: Option<String>,
}

static REGISTRY: OnceLock<HashMap<String, SpecialFormDef>> = OnceLock::new();

/// Lookup by full keyword name. Returns `Some(&SpecialFormDef)` for
/// every known special form; `None` otherwise.
///
/// The first call lazily initializes the registry; subsequent calls
/// share the same `&'static HashMap` (no Mutex/RwLock — `OnceLock`
/// initialization is the substrate's permitted concurrency primitive
/// per `docs/ZERO-MUTEX.md`).
pub fn lookup_special_form(name: &str) -> Option<&'static SpecialFormDef> {
    REGISTRY.get_or_init(build_registry).get(name)
}

/// Build a `HolonAST::Bundle` whose first child is `head` as a
/// Keyword leaf and remaining children are `slots` as bare Symbol
/// leaves (each slot's name is a literal placeholder string like
/// `"<cond>"` or `"<body>+"`).
fn sketch(head: &str, slots: &[&str]) -> HolonAST {
    let mut children = Vec::with_capacity(1 + slots.len());
    children.push(HolonAST::keyword(head));
    for s in slots {
        children.push(HolonAST::symbol(*s));
    }
    HolonAST::bundle(children)
}

/// Insert one form into the registry. The signature head MUST equal
/// the lookup name; the helper enforces this by reusing `name` in
/// both positions.
fn insert(m: &mut HashMap<String, SpecialFormDef>, name: &str, slots: &[&str]) {
    let signature = sketch(name, slots);
    m.insert(
        name.to_string(),
        SpecialFormDef {
            name: name.to_string(),
            signature,
            doc_string: None,
        },
    );
}

fn build_registry() -> HashMap<String, SpecialFormDef> {
    let mut m = HashMap::new();

    // ─── Control / branching ────────────────────────────────────────────
    // Dispatch sites: `src/check.rs:2956-2959` + `src/runtime.rs:2402-2405`.
    insert(&mut m, ":wat::core::if", &["<cond>", "<then>", "<else>"]);
    insert(&mut m, ":wat::core::cond", &["<clause>+"]);
    // Bindings: layout is `(let ((<name> <expr>)*) <body>+)` — the
    // bindings slot is a list of name/expr pairs; the type checker
    // walks it specially (arc 057 et al.).
    insert(&mut m, ":wat::core::let", &["<bindings>", "<body>+"]);
    insert(&mut m, ":wat::core::let*", &["<bindings>", "<body>+"]);
    // Match: `(match <scrutinee> -> <T> <arm>+)`. The `->` and `<T>`
    // are part of the surface form (arc 091 / arc 098 grammar).
    insert(
        &mut m,
        ":wat::core::match",
        &["<scrutinee>", "->", "<T>", "<arm>+"],
    );

    // ─── Lambdas / functions ────────────────────────────────────────────
    // Dispatch sites: `src/check.rs:3381` (lambda), `src/check.rs:3392-3397`
    // (define + defmacro return None at expression position; freeze
    // handles them as top-level forms — `src/freeze.rs:831-832`).
    insert(&mut m, ":wat::core::lambda", &["<params>", "<body>+"]);
    insert(&mut m, ":wat::core::define", &["<head>", "<body>"]);
    insert(&mut m, ":wat::core::defmacro", &["<head>", "<template>"]);

    // ─── Type definitions ───────────────────────────────────────────────
    // Dispatch sites: `src/check.rs:3393-3396` (return None at
    // expression position) + `src/freeze.rs:833-836` (top-level
    // mutation forms).
    insert(&mut m, ":wat::core::struct", &["<name>", "<field>+"]);
    insert(&mut m, ":wat::core::enum", &["<name>", "<variant>+"]);
    insert(&mut m, ":wat::core::newtype", &["<name>", "<target>"]);
    insert(&mut m, ":wat::core::typealias", &["<name>", "<target>"]);

    // ─── Error handling — canonical (post-arc-109 § J) ─────────────────
    // Dispatch sites: `src/check.rs:3000-3019` + `src/runtime.rs:2439-2449`.
    insert(&mut m, ":wat::core::Result/try", &["<expr>"]);
    insert(&mut m, ":wat::core::Option/try", &["<expr>"]);
    insert(
        &mut m,
        ":wat::core::Option/expect",
        &["->", "<T>", "<opt>", "<msg>"],
    );
    insert(
        &mut m,
        ":wat::core::Result/expect",
        &["->", "<T>", "<res>", "<msg>"],
    );

    // ─── Error handling — RETIRED (arc 109 § D' Pattern 2 poison) ──────
    // Dispatch sites: `src/check.rs:2964-2998` + `src/runtime.rs:2431-2436`.
    // These still dispatch to keep the program type-checking through
    // the migration window; the type checker pushes a synthetic
    // TypeMismatch redirecting to the canonical head. Registering them
    // here keeps reflection uniform: `(help :wat::core::try)` /just
    // works/ — even when the form itself prints a poison hint.
    insert(&mut m, ":wat::core::try", &["<retired-use-Result/try>"]);
    insert(
        &mut m,
        ":wat::core::option::expect",
        &["<retired-use-Option/expect>"],
    );
    insert(
        &mut m,
        ":wat::core::result::expect",
        &["<retired-use-Result/expect>"],
    );

    // ─── Quote / quasiquote / AST ──────────────────────────────────────
    // Dispatch sites: `src/check.rs:3083-3107, 3401-3413` + `src/runtime.rs:2406-2407, 2421`.
    insert(&mut m, ":wat::core::quote", &["<expr>"]);
    insert(&mut m, ":wat::core::quasiquote", &["<template>"]);
    // `unquote` and `unquote-splicing` are only legal INSIDE a
    // quasiquote template; at the top level they return None from
    // expression-position inference (`src/check.rs:3401-3402`).
    // Registered here for uniform reflection.
    insert(&mut m, ":wat::core::unquote", &["<expr>"]);
    insert(&mut m, ":wat::core::unquote-splicing", &["<expr>"]);
    insert(&mut m, ":wat::core::forms", &["<form>*"]);
    insert(&mut m, ":wat::core::struct->form", &["<struct-value>"]);

    // ─── Boolean shortcircuit ───────────────────────────────────────────
    // Dispatch site: `src/check.rs:3378` (special: returns :bool;
    // walks args without unifying against a fixed scheme so callers
    // can pass any boolean expression).
    insert(&mut m, ":wat::core::and", &["<expr>*"]);
    insert(&mut m, ":wat::core::or", &["<expr>*"]);

    // ─── Macro debug primitives ─────────────────────────────────────────
    // Dispatch site: `src/check.rs:3205` (special: takes :wat::WatAST,
    // returns :wat::WatAST, no scheme registration).
    insert(&mut m, ":wat::core::macroexpand-1", &["<form>"]);
    insert(&mut m, ":wat::core::macroexpand", &["<form>"]);

    // ─── Pattern-matcher entry point (arc 098) ─────────────────────────
    // Dispatch site: `src/check.rs:3269` (substrate-recognized; macros
    // expand before type-check and can't query the struct registry, so
    // matches? gets its own grammar walker).
    insert(
        &mut m,
        ":wat::form::matches?",
        &["<subject>", "<clause>+"],
    );

    // ─── Resolve-pass declaration ───────────────────────────────────────
    // Dispatch site: `src/check.rs:3382` (no-op returning :() —
    // validation happens at the resolve pass, not type inference).
    insert(&mut m, ":wat::core::use!", &["<path>"]);

    // ─── Top-level loaders (freeze-time mutation forms) ────────────────
    // Dispatch sites: `src/check.rs:3398-3400` (return None at
    // expression position) + `src/freeze.rs:837-839` (mutation forms).
    insert(&mut m, ":wat::load-file!", &["<path>"]);
    insert(&mut m, ":wat::digest-load!", &["<path>", "<digest>"]);
    insert(
        &mut m,
        ":wat::signed-load!",
        &["<path>", "<signature>", "<key>"],
    );

    // ─── Spawn family — RETIRED (arc 114 Pattern 2 poison) ─────────────
    // Dispatch sites: `src/check.rs:3334, 3343, 3356` (each pushes a
    // synthetic TypeMismatch redirecting to the canonical
    // `:wat::kernel::spawn-thread` + `:wat::kernel::Thread/join-result`
    // shape per arc 114). No runtime arms — fully retired. Registered
    // here so `(help :wat::kernel::spawn)` /just works/ and surfaces
    // the migration redirect cleanly.
    insert(
        &mut m,
        ":wat::kernel::spawn",
        &["<retired-use-spawn-thread>"],
    );
    insert(
        &mut m,
        ":wat::kernel::join",
        &["<retired-use-Thread/join-result>"],
    );
    insert(
        &mut m,
        ":wat::kernel::join-result",
        &["<retired-use-Thread/join-result>"],
    );

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_some_for_if() {
        let def = lookup_special_form(":wat::core::if").expect("if");
        assert_eq!(def.name, ":wat::core::if");
        assert!(def.doc_string.is_none());
        match &def.signature {
            HolonAST::Bundle(children) => {
                // head + 3 slots (cond, then, else)
                assert_eq!(children.len(), 4);
                assert_eq!(
                    children[0].as_symbol(),
                    Some(":wat::core::if"),
                    "first child should be the keyword head"
                );
                assert_eq!(children[1].as_symbol(), Some("<cond>"));
                assert_eq!(children[2].as_symbol(), Some("<then>"));
                assert_eq!(children[3].as_symbol(), Some("<else>"));
            }
            other => panic!("expected Bundle, got {:?}", other),
        }
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup_special_form(":wat::core::not-a-special-form").is_none());
    }

    #[test]
    fn registry_covers_audited_forms() {
        // Spot-check one entry per group.
        for name in [
            ":wat::core::if",
            ":wat::core::let*",
            ":wat::core::lambda",
            ":wat::core::define",
            ":wat::core::struct",
            ":wat::core::Result/try",
            ":wat::core::try",
            ":wat::core::quote",
            ":wat::core::quasiquote",
            ":wat::core::and",
            ":wat::core::macroexpand-1",
            ":wat::form::matches?",
            ":wat::core::use!",
            ":wat::load-file!",
            ":wat::kernel::spawn",
        ] {
            assert!(
                lookup_special_form(name).is_some(),
                "expected {} in registry",
                name
            );
        }
    }
}
