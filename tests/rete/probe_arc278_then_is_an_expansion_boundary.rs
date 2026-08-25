//! Arc 278 — a macro in a `:then` expands, exactly as one in a `:when`'s `where` does.
//!
//! FILED AS A BUG, and the report's FACTS were right while its MECHANISM was not.
//! `NOTE-rete-cond-lowers-on-the-lhs-but-not-the-rhs.md` (2026-08-24) reproduced
//! `cond` working on the LHS and failing in a `:then` with "call head must be a
//! keyword", and diagnosed it as "one missing arm in one lowerer" — proposing a
//! `cond` arm in `expr_ir::lower_list`.
//!
//! That fix would have been wrong. `cond` is not an op the lowerer should know:
//! `vocabulary.rs` records that it is its OWN `defmacro` (`wat/rete/syntax.wat`)
//! expanding to rete `if`, and that its `RETE_OPS` row "does no expansion work
//! itself". A `cond` arm in the lowerer would be a second, Rust-side copy of a wat
//! macro — the report's own table would have gained a fifth reader instead of
//! losing one.
//!
//! THE REAL CAUSE was one line in `expand_make_rule`: "items[3..]: quoted :then
//! vector + any trailing args — untouched data". `:then` was not an expansion
//! boundary; `:when`'s `where` bodies are. `boundary.rs` states the principle —
//! `where` is "the one place inside a `MakeRule` call's quoted `:when` vector where
//! a condition's DATA gives way to a live-code BODY" — and a `:then` fact-form's
//! field VALUES are that same live code (`resolve_rhs_value` evaluates fenced
//! expressions there). The principle applied to the RHS all along.
//!
//! SO IT WAS NEVER `cond`-SPECIFIC: every rete macro was unusable in a `:then`.
//! `cond` is merely the one with a vocabulary row and a purity arm loud enough to
//! notice.
//!
//! WHAT THE ROWS GUARD. Slots 0-2 are the reported case, asserting the SELECTED
//! BRANCH rather than merely that it compiled — including `:else`. Slot 3 is the
//! positional fact-form, the other shape `rete_is_kwargs` distinguishes. Slots 4-5
//! are the sharp one: a nested constructor as a VALUE. A fact-form's head and field
//! keywords are DATA — a record's registered kwargs companion macro shares the
//! record's name, so expanding the form itself would rewrite it into something the
//! RHS never meant (the `:when` side documents this as STOP-2 and avoids it by never
//! expanding fact patterns). Slot 4 proves the nested value expanded; slot 5 proves
//! its sibling field survived, i.e. the nested head stayed data. Slot 6 is the LHS
//! control, so a regression on the path that already worked is visible here too.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_then_is_an_expansion_boundary

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn witness() -> Vec<String> {
    let out = call_beside_value(file!(), ":user::witness").unwrap_or_else(|e| {
        panic!("a macro in a `:then` failed to expand — the boundary regressed: {e:?}")
    });
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    items
        .into_iter()
        .map(|v| match v {
            Value::String(s) => s.as_ref().clone(),
            other => panic!("expected String; got {other:?}"),
        })
        .collect()
}

#[test]
fn cond_in_a_then_expands_and_selects_the_right_branch() {
    let w = witness();
    assert_eq!(w.len(), 7, "witness shape changed: {w:?}");
    assert_eq!(
        (&w[0][..], &w[1][..], &w[2][..]),
        ("was-a", "was-b", "other"),
        "the reported case: `cond` in a `:then` must expand AND pick the right branch, \
         `:else` included. Compiling is not enough — a wrong branch here would be a \
         silently wrong derived fact."
    );
}

#[test]
fn both_fact_form_shapes_expand_their_values() {
    let w = witness();
    assert_eq!(
        &w[3][..], "pos-a",
        "a POSITIONAL fact-form `(:Type v …)` must expand its values too. Which \
         positions are values comes from `rete_is_kwargs` — the same predicate \
         `build_insert_fact` / `compile_rhs` / `lower_construct` use; if the expander \
         re-derived it and disagreed, it would expand a field keyword or skip a value."
    );
}

#[test]
fn a_nested_constructors_value_expands_while_its_head_stays_data() {
    let w = witness();
    assert_eq!(
        &w[4][..], "nest-a",
        "a constructor nested in a VALUE position must have its own values expanded"
    );
    assert_eq!(
        &w[5][..], "a",
        "…and its sibling field must survive intact, which is what proves the nested \
         HEAD was left as data. Expanding the form itself would invite a record's \
         registered kwargs companion macro to rewrite the fact-form (STOP-2)."
    );
}

#[test]
fn the_lhs_path_that_already_worked_still_works() {
    let w = witness();
    assert_eq!(
        &w[6][..], "1",
        "the `where`-body boundary is the one this fix mirrors; a regression there \
         would otherwise only show up somewhere far away"
    );
}
