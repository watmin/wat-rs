//! Arc 277.1c-fix — disconfirming probe: the concat-abuse rule has no AUTO-FIX yet (RED at HEAD).
//!
//! 277.1c shipped concat-abuse REPORT-ONLY. The keystone (ast-end-span) + 277.1b's FixEdit/apply-fixes
//! machinery now let it carry a real fix → rewrite a `string::concat` of literals + values into a
//! `format` call. Per the four-questions (a naming heuristic fails Simple/Obvious; `pure in -> pure
//! out`), the fix fires ONLY when every value slot is a bare symbol/keyword (name = the symbol);
//! a COMPOUND slot has no honest derivable name, so that concat stays report-only (fix = None) — its
//! naming is deferred to the arc-278 RETE map-consumer (a judgment, not a fact).
//!
//! At HEAD `lint-fix-file` leaves concat-abuse untouched → RED. GREEN when the bare-symbol fix ships.
//!
//! Run: cargo test --release -p wat --test probe_arc277_1c_concat_format_autofix -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn lint_fix(body_src: &str) -> String {
    let prog = format!(
        "(:wat::lint::lint-fix-file (:wat::source::File \"t.wat\" {body_src}))"
    );
    let world = startup_from_source("(:wat::core::defn :user::main [] -> :wat::core::nil nil)", None,
        Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!(&prog).expect("parse lint-fix-file call");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("lint-fix-file raised: {e:?}")).value_owned() {
        Value::String(ref s) => s.to_string(),
        other => panic!("lint-fix-file must return String; got {other:?}"),
    }
}

// BARE-SYMBOL slots → auto-fix to a self-documenting format call.
#[test]
fn bare_symbol_concat_rewrites_to_format() {
    // a defn body: (string::concat "x: " a " y: " b) — a,b bare symbols.
    let src = r#""(:wat::core::defn :u::g [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"x: \" a \" y: \" b))""#;
    let fixed = lint_fix(src);
    assert!(
        fixed.contains("(:wat::core::format") && fixed.contains("{a}") && fixed.contains("{b}")
            && fixed.contains(":a a") && fixed.contains(":b b"),
        "bare-symbol concat must rewrite to (format \"x: {{a}} y: {{b}}\" :a a :b b); got: {fixed}"
    );
    assert!(!fixed.contains("string::concat"), "the concat must be gone; got: {fixed}");
}

// COMPOUND slot → NO auto-fix (report-only; naming is a judgment deferred to the RETE map).
#[test]
fn compound_slot_concat_is_left_untouched() {
    // (string::concat "n=" (i64::to-string n)) — the value slot is a compound expr.
    let src = r#""(:wat::core::defn :u::h [n <- :wat::core::i64] -> :wat::core::String (:wat::core::string::concat \"n=\" (:wat::core::i64::to-string n)))""#;
    let fixed = lint_fix(src);
    assert!(
        fixed.contains("string::concat") && !fixed.contains("(:wat::core::format"),
        "a compound-value concat must stay report-only (no fix); got: {fixed}"
    );
}
