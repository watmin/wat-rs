//! Arc 277 — disconfirming probe: `wat-lint` has no rule engine yet (RED at HEAD).
//!
//! 277.1 builds the lint framework (`wat/lint.wat`): `(:wat::lint::lint-source files) ->
//! Vector<Finding>`, where a rule is `(form -> Vector<Finding>)`. The first structural rule is the
//! `nested-if-=-ladder` — a chain of `(if (= VAR LIT) true (if (= VAR LIT) true … false))` comparing
//! ONE var against literals, all returning `true` on match: a `HashSet/contains?` membership in
//! disguise (the exact bad form that `deporder`/`fix.wat` carried, arc 275). The rule detects it and
//! carries a `fix` edit toward `(:wat::core::contains? (:wat::core::HashSet :T LIT…) VAR)` — the cleaned
//! `deporder.wat` shape is the worked-reference output.
//!
//! At HEAD `:wat::lint::lint-source` is undefined → this eval errors → RED. GREEN when 277.1 ships the
//! framework + the ladder rule and the fixture's 3-deep ladder surfaces ≥1 finding.
//!
//! Reuses `:wat::deporder::SourceFile` (shipped arc 275) — the pure-function-of-sources input shape.
//!
//! Run: cargo test --release -p wat --test probe_arc277_lint_if_ladder -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A SourceFile whose body is a 3-deep nested-if-=-ladder over one var — a set membership in disguise.
// (Inner quotes escaped for the wat string lexer; the raw Rust string preserves the backslashes.)
const LINT_LADDER: &str = r#"
(:wat::lint::lint-source
  (:wat::core::Vector :wat::deporder::SourceFile
    (:wat::deporder::SourceFile "t.wat"
      "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))")))
"#;

#[test]
fn lint_finds_the_nested_if_ladder() {
    let world = startup_from_source("(:wat::core::defn :user::main [] -> :wat::core::nil nil)", None,
        Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed once wat/lint.wat is in the stdlib");
    let ast = wat::parse_one!(LINT_LADDER).expect("parse the lint-source call");
    let findings = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("lint-source raised (undefined at HEAD): {e:?}"))
        .value_owned();
    let n = match findings {
        Value::Vec(ref v) => v.len(),
        other => panic!("lint-source must return Vector<Finding>; got {other:?}"),
    };
    assert!(
        n >= 1,
        "the 3-deep nested-if-=-ladder must surface >=1 finding from the nested-if-=-ladder rule; got {n}"
    );
}
