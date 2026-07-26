//! FM-2-bis DIAGNOSTIC PROBE — does wat's sets-of-scopes hygiene actually
//! prevent classic macro variable capture at RUNTIME?
//!
//! circumspicere (arc 249 macros re-ward) flagged a CLAIM-vs-CODE contradiction:
//! `mod.rs`/`identifier.rs` claim "lexical-scope lookup compares (name, scope_set)
//! pairs so a macro's `tmp` and a user's `tmp` resolve to distinct bindings" — but
//! `Environment` is `HashMap<String, BoundEntry>` keyed on the BARE name, and
//! `Identifier::add_scope` adds to a separate `.scopes` field, leaving `.name`
//! bare. Scope sets feed AST hashing (program identity) but appear UNUSED at
//! runtime lookup. No hygiene test exists.
//!
//! Wat source lives in the co-located fixture: probe_macro_hygiene_capture.wat
//! (slurped via startup_beside(file!())). Three named compute functions in the
//! fixture: :test::compute-1, :test::compute-2, :test::compute-3.
//!
//! Run: cargo test --release --test probe_macro_hygiene_capture -- --nocapture

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.

/// DEFCLAUSE MACRO HYGIENE GUARD — proves that a macro-generated defclause
/// correctly resolves its parameter bindings when the arg idents carry a
/// macro scope tag.
///
/// History: Stone 249.5b fixed the `fn`/`let`/`match`/symbol-lookup bind sites
/// but missed the `defclause` arg-binding path. `parse_defclause_clause` called
/// `parse_argspec_triples` which returns bare names (`ident.name`) via
/// `parse_triple`; the clause bind site used those bare names directly.
/// `walk_template` had already added a macro scope to ALL template-origin
/// symbols — both the arg-binder `x` in `[x <- :i64]` AND the body reference
/// `x` in `(i64::+ x y)` — so both carry `x\u{1}<scope>`. Bind used `"x"`;
/// lookup used `"x\u{1}<scope>"` → UnboundSymbol.
///
/// The fix (Stone 249.5b defclause path): `scoped_arg_names` re-walks the
/// `WatAST::Vector` and applies `env_key` to each name symbol, producing the
/// same key as the body lookup.
#[test]
fn macro_generated_defclause_resolves_params() {
    let result = call_beside_value(file!(), ":test::compute-1").expect(
        "macro-generated defclause must evaluate without UnboundSymbol; \
         failure means bind key (bare) ≠ lookup key (scoped)"
    );
    assert_eq!(
        result,
        Value::i64(7),
        "DEFCLAUSE HYGIENE: macro-generated defclause body must resolve \
         params correctly (3 + 4 = 7). Got {:?}",
        result
    );
}

/// HYGIENE REGRESSION GUARD — proves wat's macro expansion is hygienic: a
/// macro-introduced binding does NOT capture a caller's same-named variable.
///
/// History: this was the RED contract for Stone 249.5b. `walk_template` already
/// TAGGED template symbols with a fresh macro scope (expand.rs:681), but the
/// runtime resolved names string-only (`Environment` = `HashMap<String, _>`), so
/// the tag was inert and the macro's `tmp` CAPTURED the caller's `tmp` — 200, not
/// 105. Stone 249.5b closed it by routing every Identifier-keyed bind/lookup
/// through `scope::resolution::env_key` (the scope set is now load-bearing). This
/// test went 200 → 105; `mod.rs`'s "variable capture is structurally impossible"
/// claim is now TRUE, and this guard keeps it true.
#[test]
fn classic_macro_capture_is_prevented() {
    let result = call_beside_value(file!(), ":test::compute-2").expect("expansion + eval should succeed");
    assert_eq!(
        result,
        Value::i64(105),
        "HYGIENE: the macro's `let [tmp 100]` must NOT capture the caller's `tmp` (=5). \
         105 = hygienic (user tmp distinct); 200 = captured. Got {:?}",
        result
    );
}

/// 2-SCOPE END-TO-END PROBE — proves that an identifier that accumulates TWO
/// scopes (one from an outer macro-generating-macro pass, one from the inner
/// macro invocation) still resolves correctly at runtime.
///
/// Shape: outer defmacro `make-add-inner` defines inner defmacro `inner-add`.
/// Call outer → registers `inner-add`. Call inner with 7 → result 17.
#[test]
fn two_scope_identifier_resolves_correctly_end_to_end() {
    let result = call_beside_value(file!(), ":test::compute-3").expect(
        "2-scope identifier must resolve correctly; failure means \
         bind-key (2-scope env_key) ≠ lookup-key or env_key encoding is broken"
    );
    assert_eq!(
        result,
        Value::i64(17),
        "2-SCOPE: inner macro's let [tmp 10] + caller arg 7 must = 17. \
         tmp carries {{outer_scope, inner_scope}}; env_key encodes both; \
         bind-key == lookup-key is the contract. Got {:?}",
        result
    );
}
