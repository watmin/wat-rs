//! Arc 278 #87 — rete-defn may not recurse. eBPF-shaped LOAD refusal.
//!
//! A user expression may not fault the fire loop. `pure?`/`total?` still admit
//! a cycle (a cycle is not impure). The wall is the declaration: a
//! `(:wat::rete::core::defn …)` whose call graph has a back-edge is refused
//! with `ReteDefnRecursive`, not a lying axis.
//!
//! Three fixtures:
//!   self    — one fn calls itself          → RED
//!   mutual  — a calls b, b calls a         → RED
//!   dag     — wrap calls leaf, no back-edge → GREEN (non-vacuity)

use wat::freeze::{startup_from_file, StartupError};

const SELF: &str = "tests/rete/probe_arc278_rete_defn_recurse_self.wat.bad";
const MUTUAL: &str = "tests/rete/probe_arc278_rete_defn_recurse_mutual.wat.bad";
const DAG: &str = "tests/rete/probe_arc278_rete_defn_recurse_dag.wat";

fn assert_recursive(path: &str, helper: &str) {
    let err = startup_from_file(path).expect_err("recursive rete-defn must refuse at load");
    let StartupError::Runtime(re) = &err else {
        panic!("expected StartupError::Runtime(ReteDefnRecursive), got {err:?}");
    };
    let rendered = format!("{re:?}");
    assert!(
        // rune:lint(loose-assert) — EDN carries an absolute path and live span;
        // presence of the kind tag and the helper FQDN is the claim.
        rendered.contains("#wat.runtime/ReteDefnRecursive"),
        "expected ReteDefnRecursive, got: {rendered}"
    );
    assert!(
        rendered.contains(helper),
        "diagnostic must name the helper {helper}; got: {rendered}"
    );
}

#[test]
fn self_recursive_rete_defn_refused_at_load() {
    assert_recursive(SELF, ":probe::countdown");
}

#[test]
fn mutual_recursive_rete_defns_refused_at_load() {
    let err = startup_from_file(MUTUAL).expect_err("mutual rete-defn cycle must refuse at load");
    let StartupError::Runtime(re) = &err else {
        panic!("expected StartupError::Runtime(ReteDefnRecursive), got {err:?}");
    };
    let rendered = format!("{re:?}");
    assert!(
        rendered.contains("#wat.runtime/ReteDefnRecursive"), // rune:lint(loose-assert) — Debug of RuntimeError wraps span; tag is the contract
        "expected ReteDefnRecursive, got: {rendered}"
    );
    assert!(
        rendered.contains(":probe::a") || rendered.contains(":probe::b"), // rune:lint(loose-assert) — cycle names either helper; Debug wrap varies
        "diagnostic must name a helper on the cycle; got: {rendered}"
    );
}

#[test]
fn acyclic_rete_defn_dag_still_loads() {
    startup_from_file(DAG).unwrap_or_else(|e| {
        panic!(
            "an acyclic wrap→leaf rete-defn DAG must load — if it does not, the \
             cycle walk is treating any named call as recursion. Got: {e:?}"
        )
    });
}
