//! The alpha-tree and compiled-cond CONTRACT — candidate sets and binding equivalence.
//!
//! These test modules OUTSIDE `kernel/` (`rete/alpha_tree.rs`, `rete/compiled_cond.rs`); they
//! live here only because they need a fired session to build a realistic tree from.


use super::*;

/// Every `Value::Aggregate` (non-`Struct`) fact in a fired session's final fact set —
/// `merge_facts` accumulates seed + every derived fact there across the whole fire pass.
fn all_facts_of(fired: &Value) -> Vec<Value> {
    match session_facts(fired) {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    }
}

/// The one committed instrument for row 1 and row 2 of the EXPECTATIONS scorecard: fires
/// the `[50 100]` cascade, rebuilds P8's alpha index (`build_alpha_index` — the SAME
/// function `fire_fixpoint_delta` uses, not a hand-rolled duplicate) from that fired
/// session's own network, and builds the `AlphaTree` from that index. Returns everything a
/// caller needs to compare the tree's candidate set against the matcher's true set, fact by
/// fact, without re-firing or diverging from what actually ran.
///
/// Returned as a NAMED struct rather than a 5-tuple: `clippy::type_complexity` flagged the
/// tuple, and an alias would have quieted the signature while leaving both call sites
/// destructuring by POSITION — one of them underscoring two fields purely to hold their slots.
/// Cast `perspicere` on it; its verdict was a struct over an alias, on exactly that ground
/// (a name here is better than the tuple, not merely equivalent to it).
struct AlphaTreeFixture {
    world: crate::freeze::FrozenWorld,
    tree: AlphaTree,
    alpha_by_type: AlphasByType,
    alpha_cond: HashMap<i64, WatAST>,
    facts: Vec<Value>,
}

fn alpha_tree_fixture_50_100() -> AlphaTreeFixture {
    let (world, fired) = fire_cascade(50, 100);
    let wm = to_transient(&fired).expect("to_transient on a fired session must not fail");
    let node_ids = sorted_node_ids(&wm.network);
    let (alpha_by_type, alpha_cond) = build_alpha_index(&wm.network, &node_ids);
    let tree = AlphaTree::build(&alpha_by_type, &alpha_cond, world.symbols());
    let facts = all_facts_of(&fired);
    AlphaTreeFixture {
        world,
        tree,
        alpha_by_type,
        alpha_cond,
        facts,
    }
}

/// Row 1 / STOP-2 — the ONE contract decision, as a test: for every fact the `[50 100]`
/// cascade ever held (seed + every derived fact), the tree's candidate set must be a
/// SUPERSET of the set `alpha_match_inner` actually accepts. A subset anywhere is a hard
/// fail — reported with the fact, the tree's candidate set, and the matcher's true set, per
/// STOP-2, rather than relaxed or special-cased.
#[test]
fn alpha_tree_candidate_set_is_superset_of_true_matches_at_50_100() {
    let AlphaTreeFixture {
        world,
        tree,
        alpha_by_type,
        alpha_cond,
        facts,
    } = alpha_tree_fixture_50_100();
    let sym = world.symbols();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts — the invariant would hold vacuously"
    );

    // rune:perspicere(read-once) — STOP-2 field-name cache; one probe, not a domain noun.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut checked = 0usize;
    for fact in &facts {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        let field_names = field_names_cache
            .entry(fact_class.to_string())
            .or_insert_with(|| class_field_names(sym, fact_class));

        // The oracle: alpha_match_inner run over EVERY alpha of this fact's type — exactly
        // the pre-stone linear scan, kept here as ground truth for what "actually matches"
        // means. The tree must never drop any id this set contains.
        let true_set: std::collections::HashSet<i64> = alpha_by_type
            .get(fact_class)
            .into_iter()
            .flatten()
            .filter(|aid| {
                let cond = &alpha_cond[aid];
                crate::rete::matcher::alpha_match_inner(Some(crate::rete::compiled_cond::test_sym()), cond, fact_class, fact_fields, field_names)
                    .is_some()
            })
            .copied()
            .collect();

        let candidate_set: std::collections::HashSet<i64> = tree
            .candidates(fact_class, fact_fields)
            .into_iter()
            .collect();

        let missing: Vec<i64> = true_set.difference(&candidate_set).copied().collect();
        assert!(
            missing.is_empty(),
            "STOP-2: superset invariant failed.\n  fact: {fact:?}\n  class: {fact_class}\n  \
                 tree's candidate set: {candidate_set:?}\n  matcher's true set: {true_set:?}\n  \
                 missing (dropped) alpha ids: {missing:?}"
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "no Aggregate (non-Struct) facts were checked — the invariant test measured nothing"
    );
    println!(
        "alpha_tree_candidate_set_is_superset_of_true_matches_at_50_100: checked {checked} facts, \
             superset invariant held for all of them"
    );
}

/// Row 2 / STOP-3 — the tree must actually discriminate, not just be correct. Reports mean
/// candidates/fact WITH the tree at `[50 100]` (expected ~1) alongside the SAME measurement
/// with the tree bypassed (`alpha_by_type[class].len()` — the pre-stone "every alpha of this
/// type," expected ~D=50), so a tree that wildcards everything (perfectly correct, buys
/// nothing — the trap-door row 1/5/6 would not catch) cannot read as success.
#[test]
fn alpha_tree_discriminates_candidates_to_about_one_at_50_100() {
    let AlphaTreeFixture {
        tree,
        alpha_by_type,
        facts,
        ..
    } = alpha_tree_fixture_50_100();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts"
    );

    let mut n = 0u64;
    let mut with_tree_total = 0u64;
    let mut without_tree_total = 0u64;
    let mut with_tree_hist: HashMap<usize, u64> = HashMap::new();

    for fact in &facts {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        let with_tree = tree.candidates(fact_class, fact_fields).len();
        let without_tree = alpha_by_type.get(fact_class).map(|v| v.len()).unwrap_or(0);

        with_tree_total += with_tree as u64;
        without_tree_total += without_tree as u64;
        *with_tree_hist.entry(with_tree).or_default() += 1;
        n += 1;
    }
    assert!(
        n > 0,
        "no Aggregate (non-Struct) facts were checked — the test measured nothing"
    );

    let mean_with = with_tree_total as f64 / n as f64;
    let mean_without = without_tree_total as f64 / n as f64;

    let mut hist_keys: Vec<&usize> = with_tree_hist.keys().collect();
    hist_keys.sort();
    let hist_str: String = hist_keys
        .iter()
        .map(|k| format!("{k} candidates × {} facts", with_tree_hist[*k]))
        .collect::<Vec<_>>()
        .join(", ");

    println!(
            "\n  ALPHA TREE candidate distribution at [50 100]  (n = {n} facts)\n  \
             mean candidates/fact WITH the tree:      {mean_with:.3}\n  \
             mean candidates/fact WITHOUT (bypassed): {mean_without:.3}   (the pre-stone linear scan)\n  \
             WITH-tree histogram: {hist_str}\n"
        );

    assert!(
        mean_with < 2.0,
        "STOP-3: mean candidates/fact WITH the tree is {mean_with:.3} at [50 100], not ~1 — \
             the tree is correct but discriminates nothing. Distribution: {hist_str}"
    );
    assert!(
        mean_without > 10.0,
        "the bypassed (no-tree) comparison itself collapsed — mean {mean_without:.3} \
             candidates/fact without the tree, expected ~D=50; this fixture no longer exercises \
             the depth the row-2 assertion depends on, so the row-2 pass above would be vacuous"
    );
}

// ── Compiled conditions (DESIGN-STONE-compiled-conditions.md) ────────────────────────────

/// Build every alpha's `CompiledCond`, exactly as `fire_fixpoint_delta`'s setup does — one
/// reader of `(alpha_by_type, alpha_cond)` for compilation, not a hand-rolled duplicate.
fn compile_all(
    alpha_by_type: &AlphasByType,
    alpha_cond: &HashMap<i64, WatAST>,
    sym: &crate::runtime::SymbolTable,
) -> HashMap<i64, crate::rete::compiled_cond::CompiledCond> {
    let mut compiled = HashMap::with_capacity(alpha_cond.len());
    for (class, ids) in alpha_by_type {
        let field_names = class_field_names(sym, class);
        for aid in ids {
            let cond = &alpha_cond[aid];
            let c = crate::rete::compiled_cond::compile_alpha_ops(cond, &field_names, crate::rete::compiled_cond::test_sym())
                .unwrap_or_else(|| {
                    panic!(
                        "STOP-2: compile_alpha_ops returned None for a condition \
                             build_alpha_index already accepted: {cond:?}"
                    )
                });
            compiled.insert(*aid, c);
        }
    }
    compiled
}

/// Row 1 / STOP-1 — the ONE contract decision, as a test: for every (fact, alpha) pair the
/// `[50 100]` cascade's own network+facts can form, the compiled executor's verdict AND
/// bindings array must be IDENTICAL to `alpha_match_inner`'s. A "both matched" comparison
/// would pass while producing wrong joins downstream (EXPECTATIONS row 1's named trap-door)
/// — so this asserts array equality (`Arc<[(Value, Value)]>`'s `PartialEq`, which compares
/// length, then each pair in order), never just `is_some()`.
#[test]
fn compiled_cond_bindings_identical_to_interpreter_at_50_100() {
    use crate::rete::compiled_cond::exec_compiled;

    let AlphaTreeFixture {
        world,
        alpha_by_type,
        alpha_cond,
        facts,
        ..
    } = alpha_tree_fixture_50_100();
    let sym = world.symbols();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts"
    );

    let compiled = compile_all(&alpha_by_type, &alpha_cond, sym);
    // rune:perspicere(read-once) — STOP-2 field-name cache; one probe, not a domain noun.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut scratch: SlotFrame = Vec::new();
    let mut checked = 0usize;
    let mut matched_checked = 0usize;

    for fact in &facts {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        let field_names = field_names_cache
            .entry(fact_class.to_string())
            .or_insert_with(|| class_field_names(sym, fact_class));

        // EVERY alpha of this fact's type (not just the tree's candidate set) — the
        // differential is about the executor, not the tree, so it must cover the alphas the
        // tree would have pruned too.
        for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
            let cond = &alpha_cond[aid];
            let interpreted =
                crate::rete::matcher::alpha_match_inner(Some(crate::rete::compiled_cond::test_sym()), cond, fact_class, fact_fields, field_names);
            let mut pool = Vec::new();
            let mut bkeys = Vec::new();
            let mut bvals = Vec::new();
            let mut bids = crate::rete::compiled_cond::ValIntern::default();
            let mut intern = crate::rete::compiled_cond::BindIntern {
                keys: &mut bkeys,
                vals: &mut bvals,
                ids: &mut bids,
                pool: &mut pool,
            };
            let via_compiled =
                exec_compiled(crate::rete::compiled_cond::test_sym(), &compiled[aid], fact_fields, &mut scratch, &mut intern, fact);

            match (interpreted.as_ref(), via_compiled.as_ref()) {
                (None, None) => {}
                (Some(i), Some((off, len))) => {
                    matched_checked += 1;
                    let span = super::BindSpan {
                        off: *off,
                        len: *len,
                    };
                    let c: Vec<(Value, Value)> = super::bind_view(&bkeys, &bvals, &pool, span)
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    assert_eq!(
                        i.as_ref(),
                        c.as_slice(),
                        "STOP-1: bindings array diverged.\n  fact: {fact:?}\n  alpha id: {aid}\n  \
                             interpreted: {i:?}\n  compiled: {c:?}"
                    );
                }
                _ => panic!(
                    "STOP-1: verdict diverged (one side matched, the other didn't).\n  \
                         fact: {fact:?}\n  alpha id: {aid}\n  interpreted: {interpreted:?}\n  \
                         compiled: {via_compiled:?}"
                ),
            }
            checked += 1;
        }
    }

    assert!(
        checked > 0,
        "no (fact, alpha) pairs were checked — the differential measured nothing"
    );
    assert!(
        matched_checked > 0,
        "every pair agreed None/None — the array-equality assertion (the actual STOP-1 \
             requirement) never ran once. Need at least one Some/Some comparison."
    );
    println!(
        "compiled_cond_bindings_identical_to_interpreter_at_50_100: checked {checked} \
             (fact, alpha) pairs; {matched_checked} matched on both sides with IDENTICAL bindings \
             arrays (same pairs, same order, same values)."
    );
}

/// Row 2 / STOP-3 — the load-bearing row: the failure path allocates NOTHING. Asserted via
/// the `match:key-alloc` census counter (armed at the two `Value::String(Arc::new(..))` call
/// sites in `matcher.rs` that rebuild the constant `"?var"` key on every call), with the SAME
/// measure taken against the interpreter over the IDENTICAL corpus — so a compiled path that
/// happens to read zero simply because the counter is never wired to anything live cannot
/// pass vacuously (EXPECTATIONS' named trap-door for this row).
#[test]
fn compiled_cond_failure_path_allocates_no_binding_keys_at_50_100() {
    use crate::rete::compiled_cond::exec_compiled;

    let AlphaTreeFixture {
        world,
        alpha_by_type,
        alpha_cond,
        facts,
        ..
    } = alpha_tree_fixture_50_100();
    let sym = world.symbols();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts"
    );

    let compiled = compile_all(&alpha_by_type, &alpha_cond, sym);

    let (mut calls, mut fails) = (0u64, 0u64);
    let mut scratch: SlotFrame = Vec::new();
    let (_out, compiled_rows) = super::with_count_census(|| {
        for fact in &facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_ref(), a.fields.as_slice())
                }
                _ => continue,
            };
            for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
                calls += 1;
                let mut pool = Vec::new();
                let mut keys = Vec::new();
                let mut vals = Vec::new();
                let mut ids = crate::rete::compiled_cond::ValIntern::default();
                let mut intern = crate::rete::compiled_cond::BindIntern {
                    keys: &mut keys,
                    vals: &mut vals,
                    ids: &mut ids,
                    pool: &mut pool,
                };
                if exec_compiled(crate::rete::compiled_cond::test_sym(), &compiled[aid], fact_fields, &mut scratch, &mut intern, fact)
                    .is_none()
                {
                    fails += 1;
                }
            }
        }
    });

    // rune:perspicere(read-once) — STOP-2 field-name cache; one probe, not a domain noun.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut interp_calls = 0u64;
    let (_out, interp_rows) = super::with_count_census(|| {
        for fact in &facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_ref(), a.fields.as_slice())
                }
                _ => continue,
            };
            let field_names = field_names_cache
                .entry(fact_class.to_string())
                .or_insert_with(|| class_field_names(sym, fact_class));
            for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
                interp_calls += 1;
                let _ = crate::rete::matcher::alpha_match_inner(
                    Some(crate::rete::compiled_cond::test_sym()),
                    &alpha_cond[aid],
                    fact_class,
                    fact_fields,
                    field_names,
                );
            }
        }
    });

    let get = |rows: &[(&'static str, u64)], name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    let compiled_key_allocs = get(&compiled_rows, "match:key-alloc");
    let interp_key_allocs = get(&interp_rows, "match:key-alloc");

    println!(
        "\n  ROW 2 — failure-path binding-key allocation, [50 100] cascade\n  \
             compiled calls:    {calls} ({fails} failed, {:.1}% failure rate)\n  \
             compiled path    match:key-alloc = {compiled_key_allocs}\n  \
             interpreter      match:key-alloc = {interp_key_allocs}   (over {interp_calls} calls, \
             the SAME corpus)\n",
        100.0 * fails as f64 / calls.max(1) as f64
    );

    assert!(
        calls > 0 && fails > 0,
        "the corpus produced no failing calls — row 2 would be vacuous"
    );
    assert_eq!(
        compiled_key_allocs, 0,
        "STOP-3: the compiled path allocated {compiled_key_allocs} binding key(s) on this \
             corpus — the failure path is supposed to allocate NOTHING"
    );
    assert!(
        interp_key_allocs > 0,
        "the interpreter comparison itself allocated ZERO keys over {interp_calls} calls — \
             the counter is not wired to a live call path, so compiled's zero above would prove \
             nothing"
    );
}
