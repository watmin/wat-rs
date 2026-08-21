//! RED probe for `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-inline-constraint-admits-non-rete.md`.
//!
//! **THE GAP.** A fact pattern may carry an inline constraint clause beside its bindings:
//!
//! ```clojure
//! (:probe::Reading (?loc <- :location) (:wat::core::> :value 10))
//! ```
//!
//! `compile-condition` (`wat/rete.wat:679`) branches on four shapes — `where` / `not` / `exists` /
//! `accumulate` — and a keyword-headed constraint matches none of them, so it falls to the
//! fact-pattern branch. The pattern's children are then classified by a **separate grammar in
//! Rust**, `classify_rete_clause` (`clause.rs`), which matches six literal strings:
//! `:wat::core::{= not= < > <= >=}`. Nothing on that path consults `pure?` / `deterministic?` /
//! `total?` / `primitive?`. **Law A never sees it.**
//!
//! Four of the six route through `compare_values` (`matcher.rs:453-456`), whose `?` propagates the
//! incomparable-operands error — the generic comparator's domain hole, live on the LHS.
//!
//! **THE FIX** is the builder's, and it is the same act as round 1c (per-type equality): force the
//! per-type rete spelling, so the user names the type they are comparing at. Monomorphising
//! *deletes* the domain hole rather than handling it — `i64::>` has no incomparable case. Zero
//! generic rete comparators exist, by ruling ("the rete surface is per-type, period").
//!
//! **WHAT EACH ROW PROVES** — rows 1-3 are RED today, row 4 is the payoff:
//!
//! | row | asserts | today |
//! |---|---|---|
//! | `untyped_ordering_constraint_is_refused` | generic `>` in a pattern is refused | **RED** — compiles + fires |
//! | `untyped_equality_constraint_is_refused` | generic `=` likewise (the discrimination-tree op) | **RED** |
//! | `per_type_constraint_is_admitted_and_discriminates` | `i64::>` compiles, fires, prunes | **RED** — grammar does not match it |
//! | `cross_type_constraint_is_refused_at_compile` | `i64::>` on a `String` field is a COMPILE error | **RED** |
//!
//! Row 4 is why the fix is worth more than a rename: it makes the *unmeasured* runtime question
//! (does a cross-type compare raise mid-fire, or silently answer false?) **stop existing** rather
//! than get answered. Today that clause is admitted and its semantics are unproven — two earlier
//! runs failed to distinguish the two outcomes because the harness could not tell them apart
//! (`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`).
//!
//! **This probe does not cover site 4 of the four** — `alpha_tree.rs:243`'s equality fan-out —
//! because it does not need to. `alpha_tree_discriminates_candidates_to_about_one_at_50_100`
//! (`kernel.rs:6775`) already gates it and is MUTATION-PROVEN: breaking that literal takes mean
//! candidates/fact from 1.000 to 50.000 and the row fails naming the exact mode ("the tree is
//! correct but discriminates nothing").
//!
//! An earlier draft of this comment called site 4 SILENT. It is not — that was asserted from
//! reading, and the mutation refuted it. Kept visible rather than quietly deleted. Note what the
//! same run also showed: the SUPERSET row passed under that mutation, so correctness cannot detect
//! a discrimination regression — which is why the two rows are separate, and why STOP-1 says to
//! read the discrimination row BY NAME at the weigh.
//!
//! Run: cargo nextest run --release -E 'test(probe_arc278_inline_constraint_law_a)'

use wat::freeze::{startup_from_file, FrozenWorld};
use wat::runtime::{apply_function, Value};

const UNTYPED_ORDERING: &str = "tests/rete/probe_arc278_inline_constraint_untyped_ordering.wat";
const UNTYPED_EQUALITY: &str = "tests/rete/probe_arc278_inline_constraint_untyped_equality.wat";
const PER_TYPE: &str = "tests/rete/probe_arc278_inline_constraint_per_type.wat";
const CROSS_TYPE: &str = "tests/rete/probe_arc278_inline_constraint_cross_type.wat";

/// Load a fixture and run its `:probe::run` entry, returning the derived-fact count.
///
/// A refusal can land at EITHER boundary and both count as "refused": rule validation runs at
/// freeze (`startup_from_file` → `Err`), while the compile fence raises at rule-compile time
/// (`Option/expect` → `panic_any`). The caller must not care which — it cares only that the form
/// did not silently become a live rule. Both are folded into `Err(message)`.
fn run_fixture(path: &str) -> Result<i64, String> {
    let world: FrozenWorld = match startup_from_file(path) {
        Ok(w) => w,
        Err(e) => return Err(format!("{e:?}")),
    };
    let func = match world.symbols().get(":probe::run") {
        Some(f) => f.clone(),
        None => return Err("no entry fn :probe::run".to_string()),
    };
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(Ok(Value::i64(n))) => Ok(n),
        Ok(Ok(other)) => Err(format!("expected i64 count; got {other:?}")),
        Ok(Err(e)) => Err(format!("{e:?}")),
        Err(payload) => {
            if let Some(p) = payload.downcast_ref::<wat::assertion::AssertionPayload>() {
                Err(p.message.clone())
            } else if let Some(s) = payload.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                Err((*s).to_string())
            } else {
                Err("panic-opaque".to_string())
            }
        }
    }
}

/// A refusal must NAME the offending head — R29 `RVINA ERVDIT`: the ruin is the lesson, so a
/// diagnostic that merely says "no" teaches nothing. Substring rather than `assert_eq!` here
/// *deliberately*: the exact wording is the implementer's to choose (stone STOP-3 requires only
/// that it name the head and point at the per-type twin), and pinning prose that does not exist yet
/// would be writing the fix's diagnostic for it. The head-name check is the load-bearing half.
fn assert_refusal_names_head(msg: &str, head: &str) {
    assert!(
        msg.contains(head),
        "the refusal must NAME the offending head {head:?} so the diagnostic teaches; got:\n{msg}"
    );
}

/// RED. Generic `>` inside a fact pattern must be refused — it is a non-rete head on the LHS,
/// and it is PARTIAL (`compare_values`'s `?`).
#[test]
fn untyped_ordering_constraint_is_refused() {
    let r = run_fixture(UNTYPED_ORDERING);
    let msg = r.expect_err(
        "an untyped generic `>` inline constraint must be REFUSED — it is a non-rete, partial head \
         on the LHS, and law A does not reach it (matcher.rs:374-380)",
    );
    assert_refusal_names_head(&msg, ":wat::core::>");
}

/// RED. Generic `=` likewise. Called out separately because `=` is the op the alpha
/// discrimination tree keys on (`alpha_tree.rs:243`) — migrating it is the half with the silent
/// failure mode, so it gets its own row rather than riding on the ordering one.
#[test]
fn untyped_equality_constraint_is_refused() {
    let r = run_fixture(UNTYPED_EQUALITY);
    let msg = r.expect_err(
        "an untyped generic `=` inline constraint must be REFUSED (and see alpha_tree.rs:243 — \
         its migration is the SILENT one)",
    );
    assert_refusal_names_head(&msg, ":wat::core::=");
}

/// RED. The per-type spelling must be ADMITTED and must actually DISCRIMINATE.
///
/// The count is the non-vacuity guard, and it is what makes this row worth more than "it compiled":
/// two facts are staged (`value 42`, `value 3`) and exactly ONE satisfies `> 10`. A constraint that
/// parsed but did not filter yields 2; one that filtered everything yields 0.
#[test]
fn per_type_constraint_is_admitted_and_discriminates() {
    let n = run_fixture(PER_TYPE).expect(
        "the per-type rete spelling must be ADMITTED — it is the form the fix forces users onto",
    );
    assert_eq!(
        n, 1,
        "the constraint must DISCRIMINATE: of two staged facts (value 42, value 3) exactly one is \
         > 10. Got {n} — 2 means the constraint parsed but never filtered; 0 means it filtered all."
    );
}

/// RED, and this is the payoff row. `i64::>` against a `String` field must be a COMPILE error.
///
/// This is what makes the fix more than a rename: today the cross-type case is *admitted* and its
/// runtime semantics are UNPROVEN (raise mid-fire, or silently answer false — the NaN-shaped mask).
/// After the fix the question stops existing, because the clause cannot be written: the lhs is a
/// field keyword of a declared `defrecord`, so its type is in hand at compile time.
#[test]
fn cross_type_constraint_is_refused_at_compile() {
    let r = run_fixture(CROSS_TYPE);
    let msg = r.expect_err(
        "`(:wat::rete::core::i64::> :location 10)` on a String-typed field must be a COMPILE error \
         — the per-type surface is what deletes the incomparable-operands domain hole",
    );
    // ⛔ NON-VACUITY. This row PASSED on its first run — for the wrong reason. Today the rete
    // spelling is not in the grammar at all, so the clause is `Unrecognized` and validation emits
    // `MalformedClause: … not a recognized :when shape`. That is a refusal, but it is not a TYPE
    // check, and a row that accepts it would go green the moment the grammar was widened and stay
    // green even if no type were ever checked — `[[feedback_a_green_test_can_prove_nothing]]`.
    // So: the refusal must NOT be the shape error. Name what would have to break.
    // rune:lint(loose-assert) — a targeted ABSENCE over a large output (the whole ReteCheckErrors
    // EDN blob), which is the rubric's own named exempt shape. There is no exact form available:
    // pinning the message would require the fix's diagnostic to exist, and it does not. This is
    // deliberately the NEGATIVE half only — see the note below for why the positive half is absent.
    assert!(
        !msg.contains("not a recognized :when shape") && !msg.contains("MalformedClause"),
        "VACUOUS: refused as a malformed SHAPE, not as a type mismatch. The grammar must first \
         ACCEPT `:wat::rete::core::i64::>`, and the checker must then reject it against a \
         String-typed field. Got:\n{msg}"
    );
    // ⚠ NO POSITIVE ASSERTION HERE, ON PURPOSE. A first draft added
    // `msg.contains(":location") || msg.contains("i64") || msg.contains("String")` and it was
    // WRONG to write: it asserts the presence of prose nobody has authored, so it could only ever
    // be satisfied by guessing the implementer's wording — and a `rune:lint(loose-assert)` over a
    // guess is the launder the rune exists to prevent
    // (`[[feedback_wat_stdio_is_edn_assert_structure_not_loose_contains]]`).
    // rune:exigere(attested-arc) — STOP-3 teaching diagnostic (name the offending side) is
    // tracked in docs/arc/2026/06/278-rules-engine/DESIGN-STONE-inline-constraint-admits-non-rete.md;
    // this row pins absence-of-malformed-shape only until that diagnostic ships an exact assert_eq.
}
