//! FM-2-bis DIAGNOSTIC PROBE — Stone 249.5f (canonical scope renumbering at hash time).
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
use wat::scope::{fresh_scope, Identifier, ScopeId};
use wat::span::Span;

fn sym(name: &str, scope: ScopeId) -> WatAST {
    WatAST::Symbol(Identifier::bare(name).add_scope(scope), Span::unknown())
}

// `(tmp tmp)` — both occurrences sharing ONE scope `s` (a binder + its reference, as
// `walk_template` tags them in a single expansion step).
fn shared_scope_program(s: ScopeId) -> Vec<WatAST> {
    vec![WatAST::List(vec![sym("tmp", s), sym("tmp", s)], Span::unknown())]
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
    let distinct = vec![WatAST::List(vec![sym("tmp", s1), sym("tmp", s2)], Span::unknown())]; // two scopes
    assert_ne!(
        hash_canonical_program(&shared),
        hash_canonical_program(&distinct),
        "different scope STRUCTURE (shared-one-scope vs two-distinct-scopes) must hash \
         DIFFERENTLY — the renumbering must be canonical (first-appearance DFS), never \
         a strip-all-scopes that would collapse capture into non-capture."
    );
}
