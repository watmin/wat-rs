//! Arc 277.1d — the concat fix picks the right head by POSITION (defmacro-body → interpolate intrinsic,
//! defn-body → format macro).
//!
//! **The wat source is the co-located sibling fixture** `probe_arc277_1d_concat_fix_position_gate.wat`,
//! slurped via `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust
//! string, never `format!`-assembled). The fixture's `:t::fix` lints+fixes a SourceFile carrying BOTH a
//! defmacro-body concat (expand-time → must become `interpolate`) and a defn-body concat (runtime → must
//! stay `format`); the probe fetches `:t::fix` from the frozen world + `apply_function`s it
//! (just-eval, no inline driver) and asserts each head was picked by position.
//!
//! Run: cargo test --release -p wat --test probe_arc277_1d_concat_fix_position_gate

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn concat_fix_picks_head_by_position() {
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(":t::fix")
        .expect("no :t::fix in fixture")
        .clone();
    let fixed = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("lint-fix-file raised: {e:?}"))
    {
        Value::String(ref s) => s.to_string(),
        other => panic!("expected String; got {other:?}"),
    };
    assert_eq!(
        fixed,
        concat!(
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::defmacro :u::m [x <- :wat::WatAST] -> :wat::core::String ",
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::let [s (:wat::core::ast-name x) nm ",
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::string::interpolate ",
            "\"{s}::Op\" :s s)] nm)) ",
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::defn :u::f [a <- :wat::core::String] -> :wat::core::String ",
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact fixed-source output of the lint autofix; the golden holds double-colon namespace forms that are not reader-parseable, so a structural whitespace-blind compare cannot apply here
            "(:wat::core::format ",
            "\"x: {a}\" :a a))"
        ),
        "defmacro-body → interpolate, defn-body → format; must match position-gate golden"
    );
}
