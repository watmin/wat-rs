//! Arc 277.1 — the linter's `nested-if-=-ladder` rule surfaces a finding.
//!
//! **The wat source is the co-located sibling fixture** `probe_arc277_lint_if_ladder.wat`, slurped via
//! `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust string). The
//! fixture's `:t::lint` wraps a `lint-source` over a 3-deep nested-if-=-ladder (a set membership in
//! disguise); the probe fetches `:t::lint` from the frozen world + `apply_function`s it (just-eval,
//! no inline driver) and asserts the finding count.
//!
//! Reuses `:wat::source::File` (shipped arc 275) — the pure-function-of-sources input shape.
//!
//! Run: cargo test --release -p wat --test probe_arc277_lint_if_ladder

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn lint_finds_the_nested_if_ladder() {
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(":t::lint")
        .expect("no :t::lint in fixture")
        .clone();
    let findings = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("lint-source raised: {e:?}"));
    let n = match findings {
        Value::Vec(ref v) => v.len(),
        other => panic!("lint-source must return Vector<Finding>; got {other:?}"),
    };
    assert!(
        n >= 1,
        "the 3-deep nested-if-=-ladder must surface >=1 finding from the nested-if-=-ladder rule; got {n}"
    );
}
