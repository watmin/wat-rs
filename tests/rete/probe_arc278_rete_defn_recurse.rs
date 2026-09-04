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
    // C20 (arc 278) — THIS USED TO READ `contains(":probe::a") || contains(":probe::b")`, an
    // assertion written AROUND a defect instead of at it: the blamed member was a per-process
    // coin flip, so the only assertion that could stay green was one that accepted either
    // answer. `declared_rete_defns` is a `BTreeSet` now, so the entry point is `:probe::a`
    // (the lesser name) on every run and the `||` is no longer needed to survive.
    //
    // ⚠ THIS TEST CANNOT SEE THE DEFECT IT DOCUMENTS. One process is one hash seed, so it
    // reads the same answer twice however broken the ordering is. It pins the IDENTITY;
    // `mutual_rete_defn_cycle_blames_the_same_member_every_run` below is what pins the
    // STABILITY, and it needs 24 fresh processes to do it.
    assert_eq!(
        blamed_identity(&rendered),
        (MUTUAL_BLAMED_NAME.to_string(), MUTUAL_BLAMED_LINE),
        "the mutual cycle must blame the lesser-named member at its call site; got: {rendered}"
    );
}

/// The member `apply_rete_defn_contracts` enters the cycle walk from, and therefore blames.
/// `:probe::a` sorts before `:probe::b` in the `BTreeSet` — that ordering IS the contract
/// (C20), not an accident of this fixture.
const MUTUAL_BLAMED_NAME: &str = ":probe::a";

/// Where the back-edge closes when the walk starts at `:probe::a`: line 8 is `(:probe::a n)`
/// inside `:probe::b`'s body. Enter from `:probe::b` instead and it would be line 5 — which is
/// exactly the flip C20 cured, so this number is half the assertion, not decoration.
const MUTUAL_BLAMED_LINE: u32 = 8;

/// Pull `(name, line)` out of a rendered `ReteDefnRecursive`.
///
/// Extraction, not `contains`: it PANICS with the whole diagnostic if the shape moves, so it
/// cannot decay into a check that passes over nothing (the idiom
/// `tests/lint/diagnostic_output_is_deterministic.rs` uses for `:got`). Both fields matter —
/// the name says WHO is blamed, the line says WHERE the caret lands, and C20 moved both.
fn blamed_identity(rendered: &str) -> (String, u32) {
    fn field<'a>(hay: &'a str, key: &str) -> &'a str {
        let start = hay.find(key).unwrap_or_else(|| {
            panic!("no `{key}` field in the diagnostic; the whole output was:\n{hay}")
        }) + key.len();
        let rest = &hay[start..];
        let end = rest.find(['"', ' ', ',', '}']).unwrap_or_else(|| {
            panic!("unterminated `{key}` field; the whole output was:\n{hay}")
        });
        &rest[..end]
    }
    let name = field(rendered, ":name \"").to_string();
    let line: u32 = field(rendered, ":line ")
        .parse()
        .unwrap_or_else(|e| panic!("`:line` is not a number ({e}); the whole output was:\n{rendered}"));
    (name, line)
}

/// C20 (arc 278) — ★ THE BLAMED FUNCTION MAY NOT BE CHOSEN BY HASH ORDER.
///
/// ## What was broken
///
/// `apply_rete_defn_contracts` loops over `declared_rete_defns` and `rete_defn_cycle` returns on
/// the FIRST failure, so the loop's ENTRY POINT decides which member of a mutual cycle is named
/// and which line the caret lands on. That set was a `HashSet<String>`; under Rust's per-process
/// random hasher its iteration order is a fresh coin flip every process. Driven at `c6bfe2fbb`,
/// 24 runs of the same release binary over this same file:
///
/// ```text
///   16  :probe::a  … :line 8
///    8  :probe::b  … :line 5
/// ```
///
/// Both reports are truthful — walking from `a` closes the cycle at the call to `b`, walking from
/// `b` closes it at the call to `a`. The arbitrary thing was WHERE THE WALK STARTED. A user
/// following the caret was sent to a different function next time.
///
/// ## Why 24 runs, and why 2 would have been worse than nothing
///
/// This is the whole difficulty of the test, so the number is DERIVED, not picked.
///
/// The defect's signature is a per-process Bernoulli draw. Measured at `c6bfe2fbb` over 224 runs
/// of this fixture, `:probe::a` came up 130 times — `p̂ = 0.58`, upper 95% bound `p ≤ 0.65`. An
/// N-run test that asserts the pinned identity is a FALSE GREEN on a surviving bug exactly when
/// all N draws land on the pinned side, i.e. with probability `p^N`:
///
/// ```text
///   N =  2 …  0.42        ← a "run it twice" test is a coin flip about a coin flip
///   N =  4 …  0.18
///   N = 12 …  5.7e-3      ← 1 in 176
///   N = 24 …  3.3e-5      ← 1 in 31,000   (6.0e-8 at the fair-coin p = 0.5)
/// ```
///
/// 24 is also the count C19's own corpus sweep needed to close its set of three, after a 2-run
/// scan reported two and missed the third on a flip
/// (`tests/lint/diagnostic_output_is_deterministic.rs`, header). A regression test that ran this
/// fixture twice would go green over a live defect 42% of the time — worse than no test at all,
/// because the next hand reads green as proof.
///
/// Cost: measured 0.31s per `wat` run on this fixture → ~7.5s. This binary is `wat::rete`, which
/// `.config/nextest.toml` budgets at 90s warn / 180s kill for exactly this class of work, so no
/// deadline is touched.
///
/// ## Two assertions, because one of them cannot see mutation 2
///
/// 1. **All 24 runs byte-identical** (stdout, stderr, exit code). Reverting the type to `HashSet`
///    fails HERE.
/// 2. **The identity is the pinned one.** Iterating the `BTreeSet` in reverse is perfectly
///    deterministic and blames `:probe::b` at line 5 instead — assertion 1 stays green through
///    it. Only pinning `(name, line)` proves this test reads WHO IS BLAMED rather than merely
///    "some error was produced".
///
/// A FRESH PROCESS PER RUN IS THE POINT: the variance is a per-process hash seed, so 24 calls
/// inside one process would share one seed and this test would be green by construction.
const MUTUAL_RUNS: usize = 24;

#[test]
fn mutual_rete_defn_cycle_blames_the_same_member_every_run() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut runs: Vec<(Option<i32>, Vec<u8>, Vec<u8>)> = Vec::with_capacity(MUTUAL_RUNS);
    for _ in 0..MUTUAL_RUNS {
        let out = std::process::Command::new(bin)
            .arg(MUTUAL)
            .output()
            .unwrap_or_else(|e| panic!("spawn {bin} {MUTUAL}: {e}"));
        runs.push((out.status.code(), out.stdout, out.stderr));
    }

    // NON-VACUITY: the refusal must actually have happened. A `wat` that started exiting 0 on
    // this file would be byte-stable and identity-free, and both assertions below would need
    // this one to notice.
    assert_ne!(
        runs[0].0,
        Some(0),
        "{MUTUAL} must be REFUSED at load; it exited 0, so this test is asserting determinism \
         over a program that no longer produces the diagnostic at all"
    );

    // 1. STABILITY.
    let distinct: std::collections::BTreeSet<_> = runs.iter().collect();
    assert_eq!(
        distinct.len(),
        1,
        "{MUTUAL} produced {} DISTINCT outputs over {MUTUAL_RUNS} runs of the same binary. The \
         rete-defn cycle walk is choosing its entry point by an unordered iteration again \
         (C20) — `declared_rete_defns` must be a `BTreeSet` at EVERY site. Every distinct \
         output follows IN FULL; do NOT summarise them when reporting:\n\n{}",
        distinct.len(),
        distinct
            .iter()
            .map(|(code, out, err)| format!(
                "    exit: {code:?}\n    --- stdout ---\n{}\n    --- stderr ---\n{}",
                String::from_utf8_lossy(out),
                String::from_utf8_lossy(err),
            ))
            .collect::<Vec<_>>()
            .join("\n\n")
    );

    // 2. IDENTITY.
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&runs[0].1),
        String::from_utf8_lossy(&runs[0].2)
    );
    assert_eq!(
        blamed_identity(&text),
        (MUTUAL_BLAMED_NAME.to_string(), MUTUAL_BLAMED_LINE),
        "the cycle diagnostic named a different member (or a different line). Stability alone \
         does not make this right: iterating the declaration set in a fixed but REVERSED order \
         is equally deterministic and equally wrong for a reader. Whole diagnostic:\n{text}"
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
