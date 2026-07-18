//! Arc 277.1c — the linter's `concat-abuse` rule surfaces a finding.
//!
//! **The wat source is the co-located sibling fixture** `probe_arc277_lint_concat_abuse.wat`,
//! slurped via `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust
//! string; the `.wat` is `cargo wat`-runnable + fix-wat-migratable). The fixture's `:t::lint` wraps a
//! `lint-source` over a `string::concat` chain that interleaves literals with values — the textbook
//! hand-rolled template `format` cures; the probe fetches `:t::lint` from the frozen world +
//! `apply_function`s it (just-eval, no inline driver) and asserts the count.
//!
//! The wat deftest (`wat-tests/lint.wat`) asserts the finding's rule == "concat-abuse" precisely; this
//! Rust probe is the coarse RED/GREEN gate on the count.
//!
//! Run: cargo test --release -p wat --test probe_arc277_lint_concat_abuse

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn concat_abuse_surfaces_a_finding() {
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
        "a concat chain mixing literals and values (and no if-ladder) must surface >=1 concat-abuse \
         finding; got {n}"
    );
}
