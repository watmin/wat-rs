//! FM-2-bis DIAGNOSTIC PROBE — Stone 249.5f (canonical scope renumbering at hash time).
//! Also contains the hash-IS-identity living witness for src/macros/mod.rs:4-8.
//!
//! Is `hash(expanded AST)` deterministic ACROSS RUNS for a macro-using program?
//!
//! THE BUG (at HEAD): the canonical hasher (`src/hash.rs`, `write_canonical_wat`'s
//! Symbol arm, ~:165-173) emits the RAW per-process `ScopeId` u64s. `fresh_scope()`
//! is a monotonic process-global counter, so the SAME program expanded twice (in two
//! processes, or twice in one) gets DIFFERENT scope IDs → different canonical bytes →
//! different hashes. `hash(expanded AST) IS identity` — content-addressed caching AND
//! the cross-node consensus the vector_manager design rests on — is broken for any
//! program containing macro-expanded (scope-tagged) symbols. Documented-deferred in
//! `hash.rs` § "Hygiene-scope caveat" (lines 44-65).
//!
//! THE FIX (Stone 249.5f): before hashing, renumber the scope IDs in first-
//! appearance DFS order (first distinct `ScopeId` encountered → canonical 0, next →
//! 1, …). Identical-up-to-renaming programs then hash equal. The fix MUST be a
//! canonical RENUMBER, not a STRIP — distinct scope STRUCTURE must still hash
//! distinctly (the discrimination guard below).
//!
//! Run: cargo test --release --test probe_hash_scope_renumber -- --nocapture

use wat::ast::WatAST;
use wat::hash::hash_canonical_program;
use wat::macros::{expand_all, register_defmacros, MacroRegistry};
use wat::runtime::{Environment, SymbolTable};
use wat::scope::{fresh_scope, Identifier, ScopeId};

fn sym(name: &str, scope: ScopeId) -> WatAST {
    WatAST::Symbol(Identifier::bare(name).add_scope(scope), wat::rust_caller_span!())
}

// `(tmp tmp)` — both occurrences sharing ONE scope `s` (a binder + its reference, as
// `walk_template` tags them in a single expansion step).
fn shared_scope_program(s: ScopeId) -> Vec<WatAST> {
    vec![WatAST::List(vec![sym("tmp", s), sym("tmp", s)], wat::rust_caller_span!())]
}

/// THE BUG — two programs identical up to per-process scope RENAMING must hash EQUAL.
/// At HEAD they don't (the raw monotonic `ScopeId`s differ). Stone 249.5f renumbers
/// scopes canonically before hashing. RED at HEAD; GREEN after.
#[test]
fn renamed_scopes_hash_equal() {
    let a = shared_scope_program(fresh_scope()); // scope N
    let b = shared_scope_program(fresh_scope()); // scope N+1 — different raw id
    assert_eq!(
        hash_canonical_program(&a),
        hash_canonical_program(&b),
        "programs identical up to per-process scope RENAMING must hash EQUAL \
         (canonical scope renumbering at hash time, Stone 249.5f). At HEAD the raw \
         monotonic ScopeIds differ → different canonical bytes → different hash."
    );
}

/// DISCRIMINATION GUARD — programs with different scope STRUCTURE must hash
/// DIFFERENTLY. `(tmp tmp)` sharing one scope (no capture) vs `(tmp tmp)` in two
/// distinct scopes (the macro-tmp / caller-tmp distinction) are NOT the same program;
/// the renumbering must PRESERVE that. GREEN at HEAD and after — proves the fix
/// renumbers canonically rather than stripping scopes (a strip would collapse capture
/// into non-capture and destroy the identity claim).
#[test]
fn distinct_scope_structure_hashes_differently() {
    let s1 = fresh_scope();
    let s2 = fresh_scope();
    let shared = shared_scope_program(s1); // (tmp{s1} tmp{s1}) — one scope
    let distinct = vec![WatAST::List(vec![sym("tmp", s1), sym("tmp", s2)], wat::rust_caller_span!())]; // two scopes
    assert_ne!(
        hash_canonical_program(&shared),
        hash_canonical_program(&distinct),
        "different scope STRUCTURE (shared-one-scope vs two-distinct-scopes) must hash \
         DIFFERENTLY — the renumbering must be canonical (first-appearance DFS), never \
         a strip-all-scopes that would collapse capture into non-capture."
    );
}

/// HASH-IS-IDENTITY LIVING WITNESS — src/macros/mod.rs:4-8 claims:
/// "Two source files that differ only in macro aliases (e.g. `Subtract` vs
/// `Blend _ _ 1 -1`) expand to the same canonical AST and the same hash —
/// the substrate commit of hash-IS-identity holds."
///
/// This test is the living witness for that claim. Program A defines a macro
/// alias `:test::MyAlias` that expands `(:my::prim x y 1 -1)`, then calls it.
/// Program B calls `:my::prim` directly with the same arguments. The two source
/// texts are textually distinct (A has a defmacro + an aliased call; B has only
/// the direct call) — the test is non-trivial. After macro expansion, both
/// programs reduce to the same single form and must produce identical canonical
/// hashes.
#[test]
fn macro_alias_expands_to_same_hash_as_direct_primitive() {
    // Program A (co-located tests/macros/probe_hash_scope_renumber_alias.wat): defines a
    // macro alias and calls it. Textually distinct from program B — the defmacro + alias
    // call differs from the direct primitive call. After expansion, defmacro forms are
    // consumed by expand_all and the remaining output is one form: (:my::prim 42 99 1 -1).
    //
    // Program B (co-located tests/macros/probe_hash_scope_renumber_direct.wat): the same
    // primitive call, written directly — no macro involved.
    //
    // Read from disk (not inlined) so the comparison subject — the raw source TEXT of two
    // distinct programs — lives in co-located fixtures; this test drives the raw
    // parse_all!/register_defmacros/expand_all macro-engine API directly (not a FrozenWorld),
    // so `call_beside_value`/`startup_beside` don't apply here.
    fn expand(fixture: &str) -> Vec<WatAST> {
        let src = std::fs::read_to_string(fixture)
            .unwrap_or_else(|e| panic!("wat fixture {fixture:?} must exist: {e}"));
        let forms = wat::parse_all!(&src).expect("parse ok");
        let mut reg = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut reg).expect("register_defmacros ok");
        let env = Environment::default();
        let sym = SymbolTable::default();
        expand_all(rest, &mut reg, &env, &sym).expect("expand_all ok")
    }

    let expanded_a = expand("tests/macros/probe_hash_scope_renumber_alias.wat");
    let expanded_b = expand("tests/macros/probe_hash_scope_renumber_direct.wat");

    // Sanity: both expansions are a single form.
    assert_eq!(expanded_a.len(), 1, "program A should expand to 1 form; got {}", expanded_a.len());
    assert_eq!(expanded_b.len(), 1, "program B should expand to 1 form; got {}", expanded_b.len());

    // The core claim — identical canonical hashes after expansion.
    // Failing here means macro expansion is NOT transparent to the hasher:
    // a macro alias and its expansion are treated as different identities,
    // breaking content-addressed caching and cross-node consensus.
    assert_eq!(
        hash_canonical_program(&expanded_a),
        hash_canonical_program(&expanded_b),
        "macro alias (:test::MyAlias 42 99) and direct call (:my::prim 42 99 1 -1) \
         must hash EQUAL after expansion — the hash-IS-identity claim in \
         src/macros/mod.rs:4-8 requires macro-transparent canonical hashing."
    );
}
