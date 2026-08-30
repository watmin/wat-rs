//! Arc 277.1b — the `nested-if-=-ladder` rule carries a real AUTO-FIX (rewrites the ladder).
//!
//! **The wat source is the co-located sibling fixture** `probe_arc277_1b_ladder_autofix.wat`, slurped
//! via `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust string).
//! `:wat::lint::lint-fix-file` lints a SourceFile + applies its findings' fixes, returning the fixed
//! source. The fixture's `:t::fix` runs it over a 3-deep ladder; the probe fetches `:t::fix` from the
//! frozen world + `apply_function`s it (just-eval, no inline driver) and asserts the ladder became a
//! `(contains? (HashSet …) x)` call.
//!
//! Run: cargo test --release -p wat --test probe_arc277_1b_ladder_autofix

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn ladder_autofix_rewrites_to_contains() {
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(":t::fix")
        .expect("no :t::fix in fixture")
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("lint-fix-file raised: {e:?}"));
    let fixed = match got {
        Value::String(ref s) => s.to_string(),
        other => panic!("lint-fix-file must return the fixed source String; got {other:?}"),
    };
    assert_eq!(
        fixed,
        concat!(
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool ",
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::contains? (:wat::core::HashSet :- [:wat::type::Infer] ",
            "\"a\" \"b\" \"c\") x))"
        ),
        "the ladder must be rewritten to a (contains? (HashSet …) x) golden"
    );
}
