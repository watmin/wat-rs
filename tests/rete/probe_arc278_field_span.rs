//! strike-field-span — **THE CARET MUST LAND ON THE KEYWORD THE AUTHOR MISTYPED.**
//!
//! `ReteCheckErrorKind::UnknownField` had four producers in the freeze-time rete wall and they
//! pointed at four different things. Three live ones passed an ENCLOSING form's span — the whole
//! comparison, the whole nested constructor, the whole fact — while three doc comments promised
//! the field's own. The only producer that got it right (`reorder_then_kwargs`'s `Err(bad)` arm,
//! carrying `bad.span`) could not run, and its doc was the clearest statement of the contract in
//! the file. A `Span` parameter accepts the clause's, the fact's and the field's with equal ease,
//! so nothing could tell a kept promise from a broken one.
//!
//! The cure is type-level, not per-site: there is now ONE producer, `check_field_kw`, and it takes
//! the **keyword AST node**. A bare `Span` no longer compiles at the call.
//!
//! ## Why the extent and not the file
//!
//! Every assertion here is an `.edn` golden carrying the whole `Span` — `:line`, `:col`, and
//! `:end`. The old caret and the new one live in the SAME `.wat` file, so a probe asserting the
//! filename, or asserting that the error is "located", passes on the exact defect it was written
//! to catch. Only the extent separates them. Measured at HEAD `9c4748b4d` on
//! `probe_arc278_enum_variant_typo_tagged.wat:26`: the caret spanned **cols 31–76**, 46 characters,
//! the entire `(:wat::rete::core::enum::= :grade :tg::P::Hi)`, while the offending keyword
//! `:tg::P::Hi` sits at **col 65, length 10**.
//!
//! ## The paths, and what each one is worth
//!
//! Four call paths reach the producer, and they are separate: a mutation at one leaves the others
//! green. All four are driven here (inline constraint, bind clause, kwargs fact, nested
//! constructor).
//!
//! ⚠ The fourth was **NOT REACHABLE** when this file was written, and its arm said so as a
//! disconfirming pin rather than asserting a caret that could not be produced. The type had been
//! moved out from under `walk_nested_constructors` by `defrecord`'s pre-freeze lowering — the wall
//! was orphaned, not untaught. strike-nested-wall re-pointed it and this file's fourth arm was
//! re-pointed with it, from asserting the acceptance to asserting the refusal. See
//! `a_nested_constructor_names_the_field_keyword_not_the_whole_form`, and
//! `probe_arc278_nested_wall.rs` for the four error kinds that branch produces.

use std::path::Path;
use std::process::{Command, Stdio};

/// ⚠ RUNS FROM THE MANIFEST DIR WITH A RELATIVE PATH, deliberately — the same reason
/// `probe_arc278_enum_variant_typo.rs` states: a refusal's `Span` carries `:file` verbatim, so an
/// absolute path would make the diagnostic machine-dependent and no `.edn` golden over it could
/// ever be checked in.
fn run(rel: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel} in {}: {e}", manifest.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// THE CONTROL, and it is load-bearing for all four arms below. Each fixture differs from this
/// one by a single mistyped keyword; without it, "the fixture refused" is indistinguishable from
/// "the fixture was malformed in some way I did not intend", and the nested arm's "it was
/// ACCEPTED" is indistinguishable from "rete never looked at this program at all".
///
/// It spells every shape the others misspell — an inline constraint on a real field, a bind of a
/// real field, a kwargs `:then`, and a nested constructor — and must COMPILE, FIRE, and derive
/// exactly one fact.
#[test]
fn every_shape_spelled_correctly_compiles_and_fires() {
    let (ok, out, err) = run("tests/rete/probe_arc278_field_span_ok.wat");
    assert!(ok, "the control must run — every field name in it is real\n{out}{err}");
    assert_eq!(
        out.trim(),
        "1",
        "one seeded `:fso::Src` matches and derives one `:fso::Outer` — a count that is not 1 \
         means the control drifted and the arms below prove nothing\n{out}{err}"
    );
}

/// ROW 1 — the INLINE-CONSTRAINT path (`check_operand_field_ref` → `check_field_kw`).
///
/// PRE: the producer was handed `clause.span()`, so the caret covered the whole
/// `(:wat::rete::core::i64::= :nofield 5)`. POST: `:nofield` alone — line 12, col 58, end col 66,
/// which is exactly its 8 characters at its hand-counted column in the fixture.
#[test]
fn an_inline_constraint_names_the_field_keyword_not_the_comparison() {
    let (ok, out, err) = run("tests/rete/probe_arc278_field_span_inline.wat");
    assert!(!ok, "an operand naming no declared field is a freeze refusal\n{out}{err}");
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_field_span__inline.edn",
        "the caret must be the `:nofield` KEYWORD's own extent, not the comparison's — both live \
         on the same line of the same file, so only `:col`/`:end` tell the fix from the defect"
    );
}

/// The BIND-CLAUSE path (`ReteClauseShape::Bind` → `check_field_kw`).
///
/// This path is why `ReteClauseShape::Bind` grew a `field_kw` member. The classifier resolved
/// `(?b <- :nofield)` through `keyword_payload(&items[2])`, which returns only the TEXT — so the
/// keyword node, and with it the only correct span, was dropped one line before the wall needed
/// it. POST: line 12, col 39, end col 47.
///
/// It is not one of the scorecard's three rows and it is not optional: it is the second caller of
/// the producer, and leaving it passing `clause.span()` would have left the ★ half-done — the
/// wrong span unwritable at three sites and writable at the fourth.
#[test]
fn a_bind_clause_names_the_field_keyword_not_the_whole_bind() {
    let (ok, out, err) = run("tests/rete/probe_arc278_field_span_bind.wat");
    assert!(!ok, "a bind of an undeclared field is a freeze refusal\n{out}{err}");
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_field_span__bind.edn",
        "the caret must be `:nofield`, not the whole `(?b <- :nofield)` clause"
    );
}

/// ROW 3 — the KWARGS-FACT path (`validate_then_form` → `check_field_kw`).
///
/// PRE: `fact_span`, the whole `(:fsk::Hit :nope ?k)`. The mechanism was structural rather than
/// careless — the branch built `kv_pairs` (field TEXT + value node) in one loop and checked the
/// names off it in a second, by which point the key keyword's span no longer existed. The two
/// loops are now one, so the node is still in hand at the check. POST: `:nope` at line 14, col 21,
/// end col 26.
///
/// `RhsMissingFields` rides along in this golden and KEEPS the fact form's span. That is correct
/// for it and is asserted here on purpose: "missing `k`" is a property of the whole form, not of
/// any one keyword in it, so a strike that moved every span to a field would have been wrong.
#[test]
fn a_kwargs_then_fact_names_the_field_keyword_not_the_whole_form() {
    let (ok, out, err) = run("tests/rete/probe_arc278_field_span_kwargs.wat");
    assert!(!ok, "a `:then` kwarg naming no declared field is a freeze refusal\n{out}{err}");
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_field_span__kwargs.edn",
        "`UnknownField` must carry `:nope`'s own extent while `RhsMissingFields`, which is about \
         the whole form, keeps the form's"
    );
}

/// ROW 2 — the NESTED-CONSTRUCTOR path (`walk_nested_constructors` → `check_field_kw`), and the
/// arm this file's header used to call **NOT REACHABLE**.
///
/// **RE-POINTED, NOT REPLACED (strike-nested-wall).** It was written as a DISCONFIRMING pin: it
/// asserted the wrong behaviour on purpose and said so, because the finding it carried — that the
/// producer could not fire at all — belonged to a wall-reachability strike, not to this span
/// strike. It said in its own body what must be asserted in its place the day someone wired that
/// branch. That day came; this is that assertion.
///
/// PRE, measured twice and printed by the fixture itself: **`"ACCEPTED-UNVALIDATED"`**.
/// `walk_nested_constructors` looked for the record type as the HEAD of the nested form, but by
/// freeze time `defrecord`'s companion macro has lowered every record-constructor call to
/// `(:wat::core::kwargs-construct :fsn::Inner :nope ?k)` — the type is ARGUMENT 0. `types.get` on
/// the macro's head was `None`, so the aggregate branch never opened and `UnknownField`,
/// `RhsMissingFields`, `RhsArityMismatch` and `RhsPositionalConstructionRetired` were all
/// unreachable there. The walker's sibling enum-variant branch IS live — an enum variant is not
/// lowered — which is why the walk looked exercised. The wall now reads the lowered head and takes
/// the type from index 1, the way `purity.rs`, `stratify.rs` and `expr_ir` were all re-pointed.
///
/// POST, and this is the row: `:nope` carries **its own** extent, not the nested form's. The
/// lowering preserves source spans, so the caret lands on the text the author typed even though
/// the form the wall inspected was synthesised by a macro — which is the whole reason a span
/// golden, and not a "the error is located" assertion, is what proves this.
///
/// `RhsMissingFields` rides along (the nested form leaves `x` unsupplied) and keeps the NESTED
/// FORM's span, asserted here on purpose for the same reason ROW 3 asserts it: "missing `x`" is a
/// property of the whole form, not of any one keyword in it.
///
/// ⚠ ANTI-VACUITY. The old guard was `stdout == "ACCEPTED-UNVALIDATED"`, proving the fixture
/// reached `main`; a refusal now means `main` never runs, so that guard cannot survive. Two things
/// replace it, and neither is weaker: the golden pins the exact `Span` — so a fixture that broke
/// for an unrelated reason produces a different error class, a different file, or different
/// columns, and fails — and `every_shape_spelled_correctly_compiles_and_fires` above is the control
/// that proves this file's shapes still compile when spelled correctly.
///
/// The four error kinds each get their own driving fixture in `probe_arc278_nested_wall.rs`; this
/// arm's job is the CARET, which is this file's subject.
#[test]
fn a_nested_constructor_names_the_field_keyword_not_the_whole_form() {
    let (ok, out, err) = run("tests/rete/probe_arc278_field_span_nested.wat");
    assert!(
        !ok,
        "a nested constructor naming an undeclared field is a freeze refusal. If this program \
         printed `ACCEPTED-UNVALIDATED` and exited 0, the nested-constructor wall has been \
         orphaned again: some later lowering moved the form out from under \
         `walk_nested_constructors` the same way `kwargs-construct` did, and all four of its error \
         kinds are unreachable once more\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_field_span__nested.edn",
        "`UnknownField` must carry `:nope`'s own extent — preserved through the macro lowering — \
         while `RhsMissingFields`, which is about the whole form, keeps the nested form's"
    );
}
