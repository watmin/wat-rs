//! Arc 277.1c — disconfirming probe: the linter has no `concat-abuse` rule (RED at HEAD).
//!
//! The self-fixing-toolchain RULE-half for `format` (SELF-FIXING-TOOLCHAIN.md): a `string::concat`
//! chain that interleaves string literals with non-literal args is a hand-rolled template — the rule
//! detects it and suggests `format`. Report-only (the auto-fix needs `ast-end-span`, deferred to the
//! 277.1b keystone).
//!
//! At HEAD `lint-source` runs only the `nested-if-=-ladder` rule; a concat-abuse form yields ZERO
//! findings with rule "concat-abuse" → RED. GREEN when 277.1c adds the rule.
//!
//! Run: cargo test --release -p wat --test probe_arc277_lint_concat_abuse -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A SourceFile whose body is a concat chain mixing literals ("x: ", " of ") with values (a, b) —
// the textbook hand-rolled template that `format` cures.
const LINT_CONCAT: &str = r#"
(:wat::lint::lint-source
  (:wat::core::Vector :wat::deporder::SourceFile
    (:wat::deporder::SourceFile "t.wat"
      "(:wat::core::defn :t::g [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"x: \" a \" of \" b))")))
"#;

// The fixture contains NO nested-if-=-ladder, so at HEAD (only the ladder rule runs) lint-source
// returns ZERO findings. The concat-abuse rule (277.1c) makes the concat chain surface >=1 finding.
// The wat deftest (wat-tests/lint.wat) asserts the finding's rule == "concat-abuse" precisely; this
// Rust probe is the coarse RED/GREEN gate on the count.
#[test]
#[ignore = "arc 277.1c — RED until the concat-abuse rule ships; un-ignore on green"]
fn concat_abuse_surfaces_a_finding() {
    let world = startup_from_source("(:wat::core::defn :user::main [] -> :wat::core::nil nil)", None,
        Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!(LINT_CONCAT).expect("parse the lint-source call");
    let findings = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("lint-source raised: {e:?}"))
        .value_owned();
    let n = match findings {
        Value::Vec(ref v) => v.len(),
        other => panic!("lint-source must return Vector<Finding>; got {other:?}"),
    };
    assert!(
        n >= 1,
        "a concat chain mixing literals and values (and no if-ladder) must surface >=1 concat-abuse \
         finding; got {n}"
    );
}
