//! Stone 255.12, site 2 — the macro registry's identity question, MEASURED.
//!
//! ## ⛔ THE BRIEF'S LEAD IS REFUTED
//!
//! The 255.12 brief listed `macros/registry.rs::ast_same_identity` as one of
//! four sites asking one identity question with the wrong comparator, and named
//! the mechanism: *"the converted body differs only by 3 `<-` + 2 `->` → 5 `:-`
//! — a Symbol→Keyword flip — and `ast_same_identity`'s cross arm requires
//! `id.is_reference()`, which `<-` and `->` are not. Confirm or refute."*
//!
//! **Refuted, three ways:**
//!
//! 1. [`a_macro_re_declared_in_the_other_dialect_is_a_no_op`] — a macro declared
//!    twice, once per dialect, every head and annotation re-spelled, registers
//!    as a NO-OP **on the pre-255.12 binary already**. The cross-spelling arm
//!    was never broken.
//! 2. [`an_arrow_flipped_template_is_still_a_duplicate`] — add ONE `<-`/`->` →
//!    `:-` flip inside the quasiquoted template to that same pair and it is
//!    `DuplicateMacro`. That is `wat/holon/Ngram.wat`'s actual shape: restoring
//!    exactly the two body arrows in the converted file (and nothing else) makes
//!    it check clean.
//! 3. The unit half, beside the function itself (`src/macros/registry.rs`'s
//!    `the_annotation_markers_are_three_identities_not_one_name`):
//!    `canonical_identity` answers `"<-"`, `"->"` and `":-"` — three DIFFERENT
//!    identities. The cross arm compares those, so `is_reference()` never gets a
//!    say. Relaxing it could not have equated them.
//!
//! ## Why site 2 was REPORTED and not cured
//!
//! `<-` → `:-` and `->` → `:-` is a GRAMMAR migration, not a namespace
//! re-spelling, and the codemod folds TWO markers onto ONE. Teaching a blind
//! structural walker that they are interchangeable would say `<-` ≡ `->` ≡ `:-`
//! inside a template — so two macros whose templates EMIT different surface
//! forms would be one macro. And it would have to be done by relaxing the
//! `is_reference()` guard that [`a_binder_is_not_a_keyword`] shows is load-bearing.
//!
//! The real cure is 8d-ii's: convert the baked stdlib sources at the same time
//! as the on-disk ones. `wat/holon/Ngram.wat` is `include_str!`'d into the
//! binary AND read off disk, so under a partial conversion the two copies meet
//! at the registry in two dialects. **Builder's call**, per the brief's own
//! cure/report line.

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::MacroErrorKind;

fn freeze_ok(path: &str, why: &str) {
    if let Err(e) = startup_from_file(path) {
        panic!("{why}\n  freeze should have succeeded; got: {e}");
    }
}

/// Assert STRUCTURALLY that the freeze failed with the macro registry's own
/// duplicate verdict, on the name the fixture declares twice.
///
/// Not a `contains()` over the rendered Debug: that would pass on a
/// `DuplicateMacro` about some OTHER name, so it could not tell "the registry
/// refused the re-declaration" from "the fixture has a typo that happens to
/// collide with a stdlib macro".
fn assert_duplicate_macro(path: &str, name: &str, why: &str) {
    let err = match startup_from_file(path) {
        Ok(_) => panic!("{why}\n  ⛔ ACCEPTED; this row has stopped measuring"),
        Err(e) => e,
    };
    let StartupError::Macro(m) = &err else {
        panic!("{why}\n  expected a MACRO-registry failure; got a different StartupError: {err:?}")
    };
    match &m.kind {
        MacroErrorKind::DuplicateMacro(got) => assert_eq!(
            got, name,
            "{why}\n  the registry refused a duplicate, but of the wrong name — this row has \
             stopped measuring the fixture it names"
        ),
        other => panic!(
            "{why}\n  expected DuplicateMacro({name}); the freeze DID fail, but on {other:?}"
        ),
    }
}

/// ⭐ The refutation's positive half. Pre-cure AND post-cure: rc 0.
#[test]
fn a_macro_re_declared_in_the_other_dialect_is_a_no_op() {
    freeze_ok(
        "tests/macros/probe_arc255_12_macro_cross_spelling.wat",
        "`canonical_identity` already equates a reference symbol and its keyword",
    );
}

/// NON-VACUITY for the row above — a genuinely different template in the other
/// dialect is still refused.
#[test]
fn a_divergent_template_is_still_a_duplicate() {
    assert_duplicate_macro(
        "tests/macros/probe_arc255_12_macro_divergent_template.wat.bad",
        ":p255_12::MD",
        "`+ ~x 1` and `+ ~x 2` are two macros in any dialect",
    );
}

/// ⭐ `wat/holon/Ngram.wat`'s actual shape, pinned. This is the row that says
/// what the identity door CANNOT express.
#[test]
fn an_arrow_flipped_template_is_still_a_duplicate() {
    assert_duplicate_macro(
        "tests/macros/probe_arc255_12_macro_arrow_template_diverges.wat.bad",
        ":p255_12::MA",
        "an annotation-marker flip inside a template is a grammar change, not a \
         spelling — no identity door equates `<-`, `->` and `:-`",
    );
}

/// ⛔ Why the `is_reference()` guard must STAY. A let-binder `v` is not the
/// keyword `:v`; relaxing the guard — the cure the brief proposed for site 2 —
/// would make this file freeze clean and silently discard the second template.
#[test]
fn a_binder_is_not_a_keyword() {
    assert_duplicate_macro(
        "tests/macros/probe_arc255_12_macro_binder_is_not_a_keyword.wat.bad",
        ":p255_12::MB",
        "a binder symbol and a data keyword of the same name are different nodes",
    );
}
