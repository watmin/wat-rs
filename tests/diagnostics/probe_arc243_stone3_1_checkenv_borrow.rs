//! Probe — arc 243 Stone 243.3.1 — CheckEnv borrow redesign structural verification
//!
//! FM 2-bis disconfirming probe: asserts the post-stone shape.
//!
//! - PRE-stone state: this probe FAILS to compile. `CheckEnv` is currently a
//!   non-generic struct (`pub struct CheckEnv { ... }`) that OWNS its inputs by
//!   deep-clone (`types: Arc<TypeEnv>` cloned at check.rs:2175; `binding_metadata:
//!   Arc<HashMap<...>>` deep-cloned at check.rs:2019). A `&CheckEnv<'a>` reference
//!   with a lifetime parameter is a type error against a struct with no `<'a>`.
//! - POST-stone state: this probe COMPILES + PASSES. `CheckEnv<'a>` BORROWS its
//!   two immutable inputs (`types: &'a TypeEnv`, `binding_metadata: Option<&'a
//!   HashMap<...>>`). Deep-clone-into-CheckEnv is structurally unrepresentable:
//!   a borrowed field cannot hold an owned copy.
//!
//! The disconfirmation is STRUCTURAL not behavioral: pre-stone CheckEnv can own a
//! deep clone of TypeEnv + binding_metadata; post-stone the borrow makes that
//! clone uncompilable. The probe demonstrates the failure-engineering roof — the
//! duplication SITUATION is never constructible, not merely avoided.
//!
//! Contract 3 (behavioral) lives in the existing :restricted-to integration
//! tests (wat_arc198_slice2_stone_*); the read-through path through the borrowed
//! binding_metadata must continue to type-check restricted calls identically.
//! Those tests are the behavioral half of this stone's verification; this probe
//! is the structural half.

use wat::check::CheckEnv;

/// Contract 1: `CheckEnv` carries a lifetime parameter — it BORROWS its
/// immutable inputs rather than owning deep clones of them.
///
/// Pre-stone: `CheckEnv` has NO lifetime param; `&CheckEnv<'a>` is a type
/// error ("struct takes 0 lifetime arguments"). This function fails to compile.
///
/// Post-stone: `CheckEnv<'a>` is the borrow shape; this compiles.
#[test]
fn checkenv_is_lifetimed() {
    // The function below only NAMES the type with a lifetime. If it compiles,
    // CheckEnv carries `<'a>` — the structural contract of the borrow redesign.
    fn _borrows_checkenv<'a>(_e: &CheckEnv<'a>) {}

    // A trivial assertion so the test body is non-empty; the real assertion is
    // the COMPILE of `_borrows_checkenv` above.
    let lifetimed = true;
    assert!(lifetimed, "CheckEnv<'a> compiles — borrow redesign in place");
}

/// Contract 2: the borrow is the ONLY shape — there is no owned-clone
/// constructor that reconstructs the duplication. `with_builtins_and_types`
/// takes `&TypeEnv` (a borrow), not `Arc<TypeEnv>` (an owned clone).
///
/// Pre-stone: `with_builtins_and_types(types: Arc<TypeEnv>)` — owns. Coercing
/// its address to a `fn(&TypeEnv) -> CheckEnv<'_>` pointer is a type error.
///
/// Post-stone: `with_builtins_and_types(types: &TypeEnv) -> CheckEnv<'_>` —
/// the coercion below type-checks.
#[test]
fn checkenv_constructor_borrows_typeenv() {
    use wat::types::TypeEnv;

    // The caller binds the TypeEnv FIRST (it cannot be a stack-local owned by
    // the constructor — that is exactly the borrow discipline T1 enforces).
    let types = TypeEnv::with_builtins();
    let env: CheckEnv<'_> = CheckEnv::with_builtins_and_types(&types);

    // The env borrows `types`; both live to end of scope. **The COMPILATION is the
    // assertion** — if `with_builtins_and_types` took ownership or an `Arc` instead
    // of a borrow, this function would not build, and the test fails at compile time
    // rather than at run time.
    //
    // There is deliberately no runtime assertion. This previously read
    // `assert!(true, "…")`, which clippy correctly flagged
    // (`assertions_on_constants`): a runtime check that cannot fail is not a check,
    // and dressing a compile-gate up as one misrepresents what is being proven
    // (R59 NISI FRANGAS, NIHIL PROBAS). The honest form states the mechanism instead
    // of faking a predicate.
    let _ = &env;
}

/// Contract 3: a CheckEnv built from a SymbolTable borrows the symbol table's
/// binding_metadata (no deep clone). This is the production path
/// (`from_symbols`). The structural assertion is the COMPILE of a `from_symbols`
/// call that passes `types` by reference.
///
/// Pre-stone: `from_symbols(sym: &SymbolTable, types: Arc<TypeEnv>)` — the
/// `&types` argument below is the wrong type (expects Arc). Fails to compile.
///
/// Post-stone: `from_symbols(sym: &'a SymbolTable, types: &'a TypeEnv)` — the
/// borrow call type-checks; binding_metadata is `Some(&sym.binding_metadata)`,
/// not a clone.
#[test]
fn checkenv_from_symbols_borrows() {
    use wat::runtime::SymbolTable;
    use wat::types::TypeEnv;

    let sym = SymbolTable::new();
    let types = TypeEnv::with_builtins();

    // Both inputs passed BY REFERENCE. Pre-stone this is a type error (types
    // expected as Arc<TypeEnv>). Post-stone it compiles — the borrow path.
    let env: CheckEnv<'_> = CheckEnv::from_symbols(&sym, &types);

    // **The COMPILATION is the assertion** — pre-stone, `from_symbols` expected an
    // `Arc<TypeEnv>`, so passing both inputs by reference was a type error. That it
    // builds at all is the proof the borrow path exists. Deliberately no runtime
    // assertion: the previous `assert!(true, "…")` could not fail, so it proved
    // nothing while looking like a check (R59) — see the sibling test above.
    let _ = &env;
}
