//! ★ THE FILTER BRANCH PAIR — the where-tree fast path vs the `exec_where` reference path.
//!
//! `dispatch_where_tests` (`fire/mod.rs`) holds TWO implementations of ONE semantics and picks
//! between them with `let use_tree = tids.iter().any(|id| sink.where_tree.covers(*id))`:
//!
//! | branch | behaviour |
//! |---|---|
//! | `else` | `exec_stashed_where` for EVERY (tid, token). This is the DEFINITION of the filter. |
//! | `if use_tree` | **skips** on `covers && !proven && !maybe`; **pushes without evaluating** on `proven && is_pure_cmp`; evaluates otherwise |
//!
//! The tree branch must produce an identical derived fact set. Its two proof obligations were
//! discharged by construction and by nothing else:
//!
//! 1. `covers(tid) && !proven && !maybe` ⟹ `exec_stashed_where(tid, binds) == false`
//!    — wrong here **drops** a derived fact.
//! 2. `proven && is_pure_cmp(tid)` ⟹ `exec_stashed_where(tid, binds) == true`
//!    — wrong here **invents** one.
//!
//! ## The lever
//!
//! `WhereTree::empty()` makes `covers` false for every tid, so `use_tree` is false and the
//! dispatch takes the reference branch. `fire_fixpoint_delta_armed` already accepts a prebuilt
//! arm (stratify passes slice arms through it), so the SAME staged session can be fired twice —
//! once through the real arm, once through an arm identical in every field except a
//! `WhereTree::empty()` — with **no change to `src/` of any kind**.
//!
//! ## ⛔ SETS, NEVER COUNTS
//!
//! D7 (`strike-two-writers-one-alpha`) produced a right-sized WRONG answer; a cardinality check
//! passes it. The comparison here is over the multiset of rendered derived facts, and a mismatch
//! prints both sides plus the symmetric difference.
//!
//! ## ⛔ THE COVERAGE CRITERION IS NOT `filter:test-reuse > 0`
//!
//! The strike brief said a fixture exercises this branch pair only if `filter:test-reuse > 0`.
//! **That measures obligation 2 alone.** Obligation 1's arm is a bare `continue` — it emits no
//! census counter at all, so a fixture can route every (tid, token) pair through the SKIP and
//! still read `reuse == 0`. Measured on `node-share [50 200]`: `reuse 200`, `evals 0`, and
//! **9,800 pairs skipped** — 98% of the dispatch, invisible to the brief's criterion.
//!
//! The skip count is recoverable without touching the engine, because the reference run
//! evaluates every pair exactly once:
//!
//! ```text
//!   skips = evals(reference run) − evals(tree run) − reuse(tree run)
//! ```
//!
//! Each fixture below reports all three, and the gate asserts the corpus reaches BOTH arms.
//!
//! Measured over this corpus, 2026-09-04: **115 fixtures fire the branch pair; 39 reach
//! obligation 1 and 34 reach obligation 2, and FIVE reach obligation 1 while reading
//! `reuse == 0`** — `where-boolean`, `where-collection`, `where-multivar` and two
//! `where-string` rows, 528 skipped pairs between them. A population selected by
//! `filter:test-reuse > 0` drops all five and keeps 34 of 115.
//!
//! ## ⛔ DO NOT PIN THE reuse/evals SPLIT — IT IS PROCESS-NONDETERMINISTIC
//!
//! `WhereTree::build` collects its dimension levels as
//! `let dims: Vec<DimKey> = dim_set.into_iter().collect()` out of a `std::collections::HashSet`,
//! whose iteration order is randomised per process. A different level order builds a different
//! discrimination tree, which lands the same TestNode in `proven` on one run and in `maybe` on
//! the next — so the SAME fixture reports `evals 14 / reuse 29` and `evals 29 / reuse 14` across
//! two runs of this very test. Measured 2026-09-04 over five runs: `where-join-order row 6`
//! flips, corpus reuse totals came out 2426 / 2441 / 2456.
//!
//! What does NOT move: the derived multisets (9,576 facts), the skip count (13,982) and the pair
//! count (34,368). **The answer is stable; only the route to it is not.** Every assertion here is
//! therefore a `> 0` reach or a set equality, never a pinned count — a count would be a flake by
//! construction, and its green would say nothing anyway.

use super::*;

use crate::rete::where_tree::WhereTree;
use crate::SymbolTable;

/// One fixture's dispatch census, in the units the two obligations are counted in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Dispatch {
    /// `filter:test-evals` — an `exec_stashed_where` call.
    evals: u64,
    /// `filter:test-reuse` — obligation 2's arm: a fact pushed WITHOUT evaluating.
    reuse: u64,
    /// `filter:test-pass` — a token that reached a beta/d_beta push.
    pass: u64,
}

fn dispatch_of(rows: &[(&'static str, u64)]) -> Dispatch {
    let get = |k: &str| {
        rows.iter()
            .find(|(n, _)| *n == k)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    };
    Dispatch {
        evals: get("filter:test-evals"),
        reuse: get("filter:test-reuse"),
        pass: get("filter:test-pass"),
    }
}

/// Every derived fact in a fired session's production memory, rendered, as a SORTED MULTISET.
///
/// Multiset, not set: a dropped DUPLICATE is a dropped fact, and dedup would hide it.
/// `production-memory` is `PersistentMap<node-id, PV<Record>>` — the same walk
/// `collect_derived` (`fire/rules.rs`) performs.
fn derived_multiset(fired: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(pm) = session_named_field(fired, "production-memory") else {
        panic!("fired session has no production-memory field");
    };
    if let Value::wat__core__PersistentMap(m) = pm {
        for (_node, v) in m.iter() {
            if let Value::wat__core__PersistentVector(pv) = v {
                for fact in pv.iter() {
                    out.push(crate::edn_shim::value_to_edn_string_lossy(fact, None));
                }
            }
        }
    }
    out.sort();
    out
}

/// The multiset symmetric difference of two sorted multisets, as (only-in-a, only-in-b).
fn multiset_symdiff(a: &[String], b: &[String]) -> (Vec<String>, Vec<String>) {
    let mut counts: std::collections::BTreeMap<&str, i64> = std::collections::BTreeMap::new();
    for x in a {
        *counts.entry(x.as_str()).or_insert(0) += 1;
    }
    for x in b {
        *counts.entry(x.as_str()).or_insert(0) -= 1;
    }
    let mut only_a = Vec::new();
    let mut only_b = Vec::new();
    for (k, n) in counts {
        if n > 0 {
            for _ in 0..n {
                only_a.push(k.to_string());
            }
        } else if n < 0 {
            for _ in 0..(-n) {
                only_b.push(k.to_string());
            }
        }
    }
    (only_a, only_b)
}

/// A bounded rendering of a multiset for a failure message — the WHOLE thing when it is small,
/// and an explicit, counted truncation when it is not (never a silent window).
fn show(label: &str, v: &[String]) -> String {
    const CAP: usize = 40;
    if v.len() <= CAP {
        format!("{label} ({}): {v:#?}", v.len())
    } else {
        format!(
            "{label} ({}, first {CAP} shown — {} NOT shown): {:#?}",
            v.len(),
            v.len() - CAP,
            &v[..CAP]
        )
    }
}

/// The arm this network would normally get, with its where-tree replaced by an EMPTY one.
///
/// Rebuilt from the network rather than cloned, so every other field is derived by the
/// production recipe (`build_rete_arm`) and the ONLY difference from the interned arm is the
/// tree. An empty tree covers no tid, so `use_tree` is false and `dispatch_where_tests` takes
/// the reference branch for every (tid, token) pair.
fn reference_arm(staged: &Value, sym: &SymbolTable) -> (Arc<InternedNetwork>, usize) {
    let network = session_network(staged).expect("staged session must carry a network");
    let rules = session_rules(staged);
    let mut arm = build_rete_arm(network, &rules, sym).expect("arm must build for the reference run");
    arm.where_tree = WhereTree::empty();
    // ⛔ THE SWAP IS PROVEN STRUCTURALLY, NOT ASSUMED — and not merely inferred from a census
    // afterwards. `covers` is `ids.contains`, and `ids` is every key of `compiled_wheres`, so
    // this says exactly: the arm handed to the reference run covers NO TestNode it could be
    // asked about, hence `use_tree` is false for every dispatch it sees.
    let n_wheres = arm.compiled_wheres.len();
    assert!(
        arm.compiled_wheres
            .keys()
            .all(|id| !arm.where_tree.covers(*id)),
        "the reference arm still covers a TestNode — the empty-tree swap did not take, and the \
         differential below would be the tree branch against ITSELF"
    );
    (Arc::new(arm), n_wheres)
}

/// Fire ONE staged session down BOTH branches and return (tree run, reference run).
///
/// Both runs go through `fire_fixpoint_delta_armed` with the same `FireKind`, so the ONLY
/// difference between them is which `WhereTree` the dispatch consults.
struct BranchPair {
    tree: (Vec<String>, Dispatch),
    reference: (Vec<String>, Dispatch),
    /// How many TestNodes this network compiled a `where` for — 0 means the filter phase has
    /// nothing to dispatch and the fixture cannot exercise the branch pair at all.
    wheres: usize,
}

fn fire_both_branches(staged: &Value, sym: &SymbolTable, what: &str) -> BranchPair {
    let (ref_arm, wheres) = reference_arm(staged, sym);

    let (tree_fired, tree_rows) = super::with_count_census(|| {
        fire_fixpoint_delta_armed(staged, sym, None, None, FireKind::Rules)
            .unwrap_or_else(|e| panic!("{what}: tree-branch fire raised: {e:?}"))
    });
    let (ref_fired, ref_rows) = super::with_count_census(|| {
        fire_fixpoint_delta_armed(staged, sym, None, Some(ref_arm), FireKind::Rules)
            .unwrap_or_else(|e| panic!("{what}: reference-branch fire raised: {e:?}"))
    });

    BranchPair {
        tree: (derived_multiset(&tree_fired), dispatch_of(&tree_rows)),
        reference: (derived_multiset(&ref_fired), dispatch_of(&ref_rows)),
        wheres,
    }
}

/// One measured row of the corpus — what the fixture is, and which arms it reached.
#[derive(Debug, Clone)]
struct Measured {
    what: String,
    derived: usize,
    wheres: usize,
    tree: Dispatch,
    reference: Dispatch,
    /// Obligation 1's arm: pairs the tree SKIPPED. Not a counter — derived, see the header.
    skips: i64,
}

impl Measured {
    /// Did this fixture actually route (tid, token) pairs through `dispatch_where_tests`?
    ///
    /// The reference run evaluates every pair exactly once, so `reference.evals` IS the pair
    /// count. Zero pairs means the fixture's rule carried its constraint in the ALPHA (an inline
    /// condition) and no TestNode was ever dispatched — it compares one derived set against an
    /// identical derived set and proves nothing about the branch pair.
    ///
    /// A dispatched tid always has a compiled `where` (`exec_stashed_where` refuses otherwise),
    /// and `covers` is `ids.contains` over exactly the compiled-`where` keys — so a fixture with
    /// pairs is a fixture where `use_tree` was TRUE. That is why this is the tree-firing test,
    /// and why it is `reference.evals`, not `filter:test-reuse`.
    fn exercises_the_branch_pair(&self) -> bool {
        self.wheres > 0 && self.reference.evals > 0
    }
}

/// Drive one staged session, compare the two branches, and return the measurement.
///
/// ⛔ Panics (RED) on ANY disagreement — that is a live soundness bug in the filter, and it
/// outranks every other verdict this file can produce.
fn differential(staged: &Value, sym: &SymbolTable, what: &str) -> Measured {
    let pair = fire_both_branches(staged, sym, what);
    let wheres = pair.wheres;
    let (tree_set, tree_c) = pair.tree;
    let (ref_set, ref_c) = pair.reference;

    // The reference branch must actually BE the reference branch: an empty tree can reuse
    // nothing, so a non-zero reuse here means the swap did not take and the run below would be
    // `X == X`. This is the C9 hole, asserted rather than assumed.
    assert_eq!(
        ref_c.reuse, 0,
        "{what}: the reference run reused {} tree decisions — the empty-tree arm did not take, \
         so this comparison is the tree branch against ITSELF",
        ref_c.reuse
    );

    if tree_set != ref_set {
        let (only_tree, only_ref) = multiset_symdiff(&tree_set, &ref_set);
        panic!(
            "\n⛔ FILTER BRANCH PAIR DISAGREES — {what}\n\
             \n\
             This is a LIVE SOUNDNESS BUG in `dispatch_where_tests`, not a test problem.\n\
             The where-tree branch and the `exec_where` reference branch derived different facts\n\
             from the SAME staged session.\n\
             \n\
             tree branch     : evals {} reuse {} pass {}\n\
             reference branch: evals {} reuse {} pass {}\n\
             \n\
             {}\n\
             {}\n\
             \n\
             INVENTED by the tree branch (obligation 2 — `proven && is_pure_cmp` pushed a fact\n\
             whose test would have FAILED):\n{:#?}\n\
             \n\
             DROPPED by the tree branch (obligation 1 — `covers && !proven && !maybe` skipped an\n\
             evaluation that would have PASSED):\n{:#?}\n",
            tree_c.evals,
            tree_c.reuse,
            tree_c.pass,
            ref_c.evals,
            ref_c.reuse,
            ref_c.pass,
            show("tree branch derived", &tree_set),
            show("reference branch derived", &ref_set),
            only_tree,
            only_ref,
        );
    }

    // Every (tid, token) pair the reference branch evaluated is, in the tree branch, exactly one
    // of: skipped (obligation 1), reused (obligation 2), or evaluated. The pair populations are
    // identical — the two runs fired the same staged session and just agreed on the answer — so
    // the skip count is the remainder.
    let skips = ref_c.evals as i64 - tree_c.evals as i64 - tree_c.reuse as i64;

    Measured {
        what: what.to_string(),
        derived: tree_set.len(),
        wheres,
        tree: tree_c,
        reference: ref_c,
        skips,
    }
}

// ── The corpus ────────────────────────────────────────────────────────────────────────────

/// The `where-*` expressivity corpus lives on disk, outside this file.
fn grid_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wat-scripts/perf/grid")
}

/// A `where-*.wat` file that follows the corpus's UNIFORM row protocol, as discovered by
/// reading the file itself — never a hand-kept list of namespaces that could rot against it.
#[derive(Debug, Clone)]
struct UniformAxis {
    stem: String,
    ns: String,
    /// The axis's own rule-builder verb — `build-rules` in ten files, `rules-for` in two.
    /// READ OFF THE FILE, never assumed: assuming `build-rules` here silently excluded the two
    /// `rules-for` axes on the first drive.
    rules_verb: String,
    rows: i64,
}

/// The row-driver shape every uniform `where-*.wat` shares, modulo its namespace:
///
/// ```text
///   staged (:NS::seed (match (compile-all rules (PersistentVector (:NS::q-Hit))) …) (:NS::items))
/// ```
///
/// A file is UNIFORM iff that exact skeleton is present and a `row-count` can be read. Anything
/// else is excluded BY NAME below with the reason — the excluded set is asserted, so a file that
/// silently changes protocol reds instead of quietly leaving the population.
fn classify(stem: &str, src: &str) -> Option<UniformAxis> {
    const MID: &str =
        "::seed (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:";
    let at = src.find(MID)?;
    let head = &src[..at];
    let ns_start = head.rfind("(:")? + 2;
    let ns = &head[ns_start..];
    if ns.is_empty() || !ns.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        return None;
    }
    // The query and the items call must both be this same namespace's.
    let tail = &src[at + MID.len()..];
    if !tail.starts_with(&format!("{ns}::q-Hit)))")) {
        return None;
    }
    if !src.contains(&format!("(:{ns}::items))")) {
        return None;
    }
    let marker = format!("defn :{ns}::row-count [] -> :wat::core::i64 ");
    let rc_at = src.find(&marker)? + marker.len();
    let digits: String = src[rc_at..].chars().take_while(|c| c.is_ascii_digit()).collect();
    let rows: i64 = digits.parse().ok()?;
    if rows <= 0 {
        return None;
    }
    // The `rules` binding that feeds `compile-all`, immediately above `staged` in the same
    // `let`: `rules (:NS::<verb> row)`. Walk back from the `(:NS` that opens `staged`.
    let before = &head[..head.rfind("(:")?];
    let close = before.rfind(" row)")?;
    let open = before[..close].rfind("(:")? + 2;
    let called = &before[open..close];
    let verb = called.strip_prefix(&format!("{ns}::"))?;
    if verb.is_empty() || !verb.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
        return None;
    }
    Some(UniformAxis {
        stem: stem.to_string(),
        ns: ns.to_string(),
        rules_verb: verb.to_string(),
        rows,
    })
}

/// The `where-*.wat` stems that do NOT follow the uniform row protocol, and therefore cannot be
/// driven by the generic row driver above. Asserted by EXACT set equality against what the walk
/// finds, so a new axis cannot land unnoticed and an existing one cannot quietly change shape.
///
/// None of these is excluded for its RESULT — the classification reads only the file's driver
/// skeleton, and it was written before any of them was fired.
const NON_UNIFORM: &[&str] = &[
    "where-accum-from-left",
    "where-accum-group",
    "where-accum-lead",
    "where-accum-lead-cascade",
    "where-accum-where",
    "where-accum-where-chain",
    "where-exists",
    "where-fact-bind",
    "where-join-left",
    "where-nested-combinators",
    "where-not-and",
    "where-not-and-bound",
    "where-not-and-not",
    "where-not-bound",
    "where-not-derived-in-query",
    "where-not-fact",
    "where-not-not",
    "where-not-or",
    "where-not-where",
    "where-not-windy",
    "where-or-and",
    "where-or-conditions",
    "where-or-inline",
    "where-query-compat",
    "where-query-params",
    "where-test-chain",
];

fn walk_corpus() -> (Vec<UniformAxis>, Vec<String>) {
    let dir = grid_dir();
    let mut uniform = Vec::new();
    let mut other = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("grid dir {dir:?} must be readable: {e}"))
        .map(|e| e.expect("dir entry").path())
        .collect();
    entries.sort();
    for path in entries {
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".wat") else {
            continue;
        };
        if !stem.starts_with("where-") {
            continue;
        }
        let src = std::fs::read_to_string(&path).expect("axis source must be readable");
        match classify(stem, &src) {
            Some(a) => uniform.push(a),
            None => other.push(stem.to_string()),
        }
    }
    (uniform, other)
}

/// Stage ONE row of a uniform axis: compile that row's single rule, seed the axis's own facts,
/// and stop BEFORE the fire. Byte-for-byte the same expression the axis's own `run-row` builds.
fn stage_row(world: &crate::freeze::FrozenWorld, axis: &UniformAxis, row: i64) -> Value {
    let UniformAxis { ns, rules_verb, .. } = axis;
    let src = format!(
        "(:{ns}::seed (:wat::core::match (:wat::rete::compile-all (:{ns}::{rules_verb} {row}) (:wat::core::PersistentVector (:{ns}::q-Hit))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) (:{ns}::items))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the staging driver");
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("staging {ns} row {row} raised: {e:?}"))
        .value_owned()
}

/// ★ THE DIFFERENTIAL — the fast filter must derive exactly what the reference filter derives.
///
/// Corpus: every `where-*.wat` axis in `wat-scripts/perf/grid/` that follows the uniform row
/// protocol, every row of it, plus `node-share [50 200]` from `NODE_SHARE_WORLD` (the fixture
/// the strike was drawn on, and the one measured to take the tree branch hardest).
///
/// Four things are asserted, and the ORDER matters:
///
/// 1. **The corpus walk is whole** — uniform ∪ non-uniform is every `where-*.wat` on disk, and
///    the non-uniform half matches `NON_UNIFORM` exactly.
/// 2. **The reference run really is the reference run** (`reuse == 0`) — per fixture, inside
///    `differential`. Without it the comparison could be the tree branch against itself.
/// 3. **The two branches derive the same multiset** — the correctness verdict, per fixture.
/// 4. **The population is not vacuous and reaches BOTH arms** — some fixture derives facts, some
///    fixture skips (obligation 1), some fixture reuses (obligation 2). Checked LAST, so an
///    anti-vacuity message can never pre-empt a correctness failure.
#[test]
fn where_tree_branch_agrees_with_the_reference_filter() {
    let (uniform, other) = walk_corpus();

    let mut expected_other: Vec<String> = NON_UNIFORM.iter().map(|s| s.to_string()).collect();
    expected_other.sort();
    let mut found_other = other.clone();
    found_other.sort();
    assert_eq!(
        found_other, expected_other,
        "the non-uniform `where-*` axes must match NON_UNIFORM exactly — a new axis, a deleted \
         one, or one that changed its row-driver shape all land here"
    );
    assert!(
        !uniform.is_empty(),
        "no `where-*` axis followed the uniform row protocol — the corpus walk found nothing to drive"
    );

    let mut measured: Vec<Measured> = Vec::new();

    for axis in &uniform {
        let path = grid_dir().join(format!("{}.wat", axis.stem));
        let src = std::fs::read_to_string(&path).expect("axis source must be readable");
        let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
            .unwrap_or_else(|e| panic!("{} must freeze: {e:?}", axis.stem));
        for row in 1..=axis.rows {
            let staged = stage_row(&world, axis, row);
            let what = format!("{} row {row}", axis.stem);
            measured.push(differential(&staged, world.symbols(), &what));
        }
    }

    // `node-share [50 200]` — 50 TestNodes over 200 tokens, the axis the strike was drawn on.
    {
        let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("node-share world should freeze");
        let src = "(:nsh::seed (:wat::core::match (:wat::rete::compile (:nsh::build-rules 50)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200)";
        let ast = crate::parse_one!(src).expect("parse the node-share staging driver");
        let staged = eval_in_frozen(&ast, &world, &Environment::new())
            .expect("node-share staging")
            .value_owned();
        measured.push(differential(&staged, world.symbols(), "node-share [50 200]"));
    }

    // ── the measured corpus, printed whole ────────────────────────────────────────────────
    let mut report = String::from(
        "\n  FILTER BRANCH-PAIR DIFFERENTIAL — tree branch vs `exec_where` reference branch\n\
         \x20 skips = obligation 1's arm (`covers && !proven && !maybe` -> continue). It emits NO\n\
         \x20 census counter, so it is DERIVED: ref-evals − tree-evals − reuse. `filter:test-reuse`\n\
         \x20 alone cannot see it. `fires` = the fixture dispatched >= 1 (tid, token) pair.\n\n\
         \x20 fixture                             fires  wheres  derived  tree-evals  reuse    skips  ref-evals\n\
         \x20 ----------------------------------  -----  ------  -------  ----------  ------  -------  ---------\n",
    );
    for m in &measured {
        report.push_str(&format!(
            "\x20 {:<34}  {:>5}  {:>6}  {:>7}  {:>10}  {:>6}  {:>7}  {:>9}\n",
            m.what,
            if m.exercises_the_branch_pair() { "yes" } else { "NO" },
            m.wheres,
            m.derived,
            m.tree.evals,
            m.tree.reuse,
            m.skips,
            m.reference.evals
        ));
    }
    let firing: Vec<&Measured> = measured
        .iter()
        .filter(|m| m.exercises_the_branch_pair())
        .collect();
    let total_derived: usize = firing.iter().map(|m| m.derived).sum();
    let total_skips: i64 = firing.iter().map(|m| m.skips).sum();
    let total_reuse: u64 = firing.iter().map(|m| m.tree.reuse).sum();
    let obl1_fixtures = firing.iter().filter(|m| m.skips > 0).count();
    let obl2_fixtures = firing.iter().filter(|m| m.tree.reuse > 0).count();
    report.push_str(&format!(
        "\x20 {:<34}  {:>5}  {:>6}  {:>7}  {:>10}  {:>6}  {:>7}  {:>9}\n\
         \n\
         \x20 {} fixtures walked, {} FIRE the tree branch (the gated population).\n\
         \x20 obligation 1 (skip)          reached by {} of them, {} pairs total.\n\
         \x20 obligation 2 (unevald push)  reached by {} of them, {} pairs total.\n",
        format!("TOTAL (firing only)"),
        firing.len(),
        firing.iter().map(|m| m.wheres).sum::<usize>(),
        total_derived,
        firing.iter().map(|m| m.tree.evals).sum::<u64>(),
        total_reuse,
        total_skips,
        firing.iter().map(|m| m.reference.evals).sum::<u64>(),
        measured.len(),
        firing.len(),
        obl1_fixtures,
        total_skips,
        obl2_fixtures,
        total_reuse,
    ));
    println!("{report}");

    // ── the population must exist (STOP-2 / C9's hole) ────────────────────────────────────
    //
    // A fixture where no (tid, token) pair reaches `dispatch_where_tests` compares the reference
    // branch against itself. If the WHOLE corpus were like that, every agreement above would be
    // `X == X` and the gate would be green over a population that cannot express the defect.
    assert!(
        !firing.is_empty(),
        "NO FIXTURE FIRES THE TREE BRANCH. Every comparison above is the reference branch against \
         itself — the gate is green over a population that cannot express the defect (C9's \
         hole).\n{report}"
    );

    // ── not vacuous: an empty derived multiset equals an empty derived multiset ────────────
    let empty: Vec<&str> = firing
        .iter()
        .filter(|m| m.derived == 0)
        .map(|m| m.what.as_str())
        .collect();
    assert!(
        empty.is_empty(),
        "VACUOUS FIXTURE(S) IN THE FIRING POPULATION — these derived NOTHING, so their two \
         branches agreed about an empty set: {empty:?}\n{report}"
    );

    // ── both obligations, or this is half a gate ──────────────────────────────────────────
    assert!(
        total_skips > 0,
        "OBLIGATION 1 UNREACHED: no firing fixture took the `covers && !proven && !maybe` SKIP \
         arm, so a skip that should have evaluated could not have been observed. Half a \
         gate.\n{report}"
    );
    assert!(
        total_reuse > 0,
        "OBLIGATION 2 UNREACHED: no firing fixture took the `proven && is_pure_cmp` \
         unevaluated-push arm, so an invented fact could not have been observed. Half a \
         gate.\n{report}"
    );
}
