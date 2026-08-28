//! Arc 277.1c-fix — the concat-abuse rule carries an AUTO-FIX (concat-of-literals+values → `format`).
//!
//! **The wat source is the co-located sibling fixture** `probe_arc277_1c_concat_format_autofix.wat`,
//! slurped via `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust
//! string, never `format!`-assembled). The two cases are named `defn`s the probe calls by name.
//!
//! Per the four-questions (`pure in -> pure out`), the fix fires ONLY when every value slot is a bare
//! symbol/keyword (name = the symbol); a COMPOUND slot has no honest derivable name, so that concat
//! stays report-only — its naming is deferred to the arc-278 RETE map-consumer (a judgment, not a fact).
//!
//! Run: cargo test --release -p wat --test probe_arc277_1c_concat_format_autofix

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): fetch the fixture's named zero-arg entry fn by name and apply_function it —
// no inline driver expression. Returns its String result.
fn fix(entry: &str) -> String {
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(entry)
        .unwrap_or_else(|| panic!("no {entry} in fixture"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("lint-fix-file raised: {e:?}"))
    {
        Value::String(ref s) => s.to_string(),
        other => panic!("lint-fix-file must return String; got {other:?}"),
    }
}

// BARE-SYMBOL slots → auto-fix to a self-documenting format call.
#[test]
fn bare_symbol_concat_rewrites_to_format() {
    let fixed = fix(":t::fix-bare");
    assert_eq!(
        fixed,
        concat!(
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::defn :u::g [a <- :wat::core::String b <- :wat::core::String] ",
            "-> :wat::core::String (:wat::core::format ",
            "\"x: {a} y: {b}\" :a a :b b))"
        ),
        "bare-symbol concat must match format-rewrite golden"
    );
}

// COMPOUND slot → NO auto-fix (report-only; naming is a judgment deferred to the RETE map).
#[test]
fn compound_slot_concat_is_left_untouched() {
    let fixed = fix(":t::fix-compound");
    assert_eq!(
        fixed,
        concat!(
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::defn :u::h [n <- :wat::core::i64] -> :wat::core::String ",
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::string::concat ",
            "\"n=\" (:wat::core::i64::to-string n)))"
        ),
        "compound-value concat must stay untouched (report-only) golden"
    );
}
