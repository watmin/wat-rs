//! Arc 277.1b — the `nested-if-=-ladder` rule carries a real AUTO-FIX (rewrites the ladder).
//!
//! **The wat source is the co-located sibling fixture** `probe_arc277_1b_ladder_autofix.wat`, slurped
//! via `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust string).
//! `:wat::lint::lint-fix-file` lints a SourceFile + applies its findings' fixes, returning the fixed
//! source. The fixture's `:t::fix` runs it over a 3-deep ladder; the probe eval_in_frozen's `(:t::fix)`
//! and asserts the ladder became a `(contains? (HashSet …) x)` call.
//!
//! Run: cargo test --release -p wat --test probe_arc277_1b_ladder_autofix

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn ladder_autofix_rewrites_to_contains() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::fix)").expect("parse the fix call");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("lint-fix-file raised: {e:?}"))
        .value_owned();
    let fixed = match got {
        Value::String(ref s) => s.to_string(),
        other => panic!("lint-fix-file must return the fixed source String; got {other:?}"),
    };
    assert!(
        fixed.contains("contains?") && fixed.contains("HashSet"),
        "the ladder must be rewritten to a (contains? (HashSet …) x) call; got: {fixed}"
    );
    assert!(
        !fixed.contains("(:wat::core::if (:wat::core::= x"),
        "the nested-if-=-ladder must be GONE after the fix; got: {fixed}"
    );
}
