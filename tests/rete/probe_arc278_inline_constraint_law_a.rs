//! GREEN gate for `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-inline-constraint-admits-non-rete.md`.
//!
//! A fact pattern may carry an inline constraint clause beside its bindings:
//!
//! ```clojure
//! (:probe::Reading (?loc <- :location) (:wat::core::> :value 10))
//! ```
//!
//! Freeze classifies CoreGeneric as NonReteConstraint. Native `compile_condition_local` returns
//! false on CoreGeneric. `compile-condition` lives at `wat/rete/compile.wat:364`;
//! `classify_rete_clause` lives at `clause.rs:173`. The untyped-ordering fixture already names
//! the freeze wall.
//!
//! The rete surface is per-type: the user names the type they are comparing at. Monomorphising
//! *deletes* the domain hole rather than handling it — `i64::>` has no incomparable case. Zero
//! generic rete comparators exist, by ruling ("the rete surface is per-type, period").
//!
//! **WHAT EACH ROW PROVES** — `expect_err` is the GREEN gate for rows 1, 2, and 4:
//!
//! | row | asserts | gate |
//! |---|---|---|
//! | `untyped_ordering_constraint_is_refused` | generic `>` in a pattern is refused | `expect_err` (NonReteConstraint) |
//! | `untyped_equality_constraint_is_refused` | generic `=` likewise (the discrimination-tree op) | `expect_err` |
//! | `per_type_constraint_is_admitted_and_discriminates` | `i64::>` compiles, fires, prunes | count == 1 |
//! | `cross_type_constraint_is_refused_at_compile` | `i64::>` on a `String` field is a COMPILE error | `expect_err` (ConstraintTypeMismatch) |
//!
//! Row 4 makes the unmeasured runtime question (does a cross-type compare raise mid-fire, or
//! silently answer false?) **stop existing** rather than get answered.
//!
//! **This probe does not cover site 4 of the four** — `alpha_tree.rs:243`'s equality fan-out —
//! because it does not need to. `alpha_tree_discriminates_candidates_to_about_one_at_50_100`
//! (`kernel/tests.rs`) already gates it and is MUTATION-PROVEN: breaking that literal takes mean
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
/// that it name the head and point at the per-type twin). The head-name check is the load-bearing half.
fn assert_refusal_names_head(msg: &str, head: &str) {
    assert!(
        msg.contains(head),
        "the refusal must NAME the offending head {head:?} so the diagnostic teaches; got:\n{msg}"
    );
}

/// Generic `>` inside a fact pattern is refused — it is a non-rete head on the LHS
/// (NonReteConstraint / Law A freeze wall).
#[test]
fn untyped_ordering_constraint_is_refused() {
    let r = run_fixture(UNTYPED_ORDERING);
    let msg = r.expect_err(
        "an untyped generic `>` inline constraint must be REFUSED — it is a non-rete head \
         (NonReteConstraint / Law A freeze wall)",
    );
    assert_refusal_names_head(&msg, ":wat::core::>");
}

/// Generic `=` likewise. Called out separately because `=` is the op the alpha
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

/// The per-type spelling is ADMITTED and actually DISCRIMINATEs.
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

/// The payoff row. `i64::>` against a `String` field is a COMPILE error.
///
/// This is what makes the fix more than a rename: the cross-type question (raise mid-fire, or
/// silently answer false — the NaN-shaped mask) stops existing, because the clause cannot be
/// written: the lhs is a field keyword of a declared `defrecord`, so its type is in hand at
/// compile time.
#[test]
fn cross_type_constraint_is_refused_at_compile() {
    let r = run_fixture(CROSS_TYPE);
    let msg = r.expect_err(
        "`(:wat::rete::i64::> :location 10)` on a String-typed field must be a COMPILE error \
         — the per-type surface is what deletes the incomparable-operands domain hole",
    );
    // ⛔ NON-VACUITY. The refusal must be a TYPE check, not a shape error. A `MalformedClause:
    // … not a recognized :when shape` refusal would go green the moment the grammar was widened
    // and stay green even if no type were ever checked — `[[feedback_a_green_test_can_prove_nothing]]`.
    // So: the refusal must NOT be the shape error. Name what would have to break.
    // rune:lint(loose-assert) — targeted ABSENCE of MalformedClause over a ReteCheckErrors blob
    // that still embeds an absolute Span path.
    assert!(
        !msg.contains("not a recognized :when shape") && !msg.contains("MalformedClause"),
        "VACUOUS: refused as a malformed SHAPE, not as a type mismatch. The grammar must first \
         ACCEPT `:wat::rete::i64::>`, and the checker must then reject it against a \
         String-typed field. Got:\n{msg}"
    );
    // rune:lint(loose-assert) — ConstraintTypeMismatch Display embeds a Span path; pin the
    // teaching fields (field / op_type / field_type), not the whole blob.
    assert!(
        msg.contains("ConstraintTypeMismatch")
            || (msg.contains(":location") && msg.contains("i64") && msg.contains("String")),
        "STOP-3: refusal must name the type mismatch (field :location, op i64, declared String). Got:\n{msg}"
    );
}
