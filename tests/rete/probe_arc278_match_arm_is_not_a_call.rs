//! strike-match-arm-is-not-a-call (arc 278, work-list **D5**) — **A MATCH ARM'S PATTERN IS NOT A
//! CONSTRUCTOR CALL.**
//!
//! ⛔ **ON THIS TREE, TWO OF THE FIVE TESTS BELOW ARE INVERTED FROM GROK'S OWN VERSION** —
//! `the_bare_and_wrapped_then_spellings_compile_and_agree` and
//! `a_correct_constructor_in_a_match_arm_body_still_fires` now assert REFUSAL, not compilation.
//! This is a builder-ruled divergence (`BRIEF-7j-ADDENDUM-324-the-then-match-collision.md`,
//! 2026-09-16, 4-YES OPTION A), not a re-reading of D5's own intent: main's own `250162a0e
//! SCORE(277)` (ACCEPTED, ancestor of this batch's own start point, absent from
//! `origin/grok-rete`) makes the then-item fence refuse `:wat::rete::core::match` anywhere inside
//! a `:then` item, for any spelling, permanently. grok's branch never carries that ruling, so
//! grok's own five fixtures below were written assuming the `:wat::rete::core::` spelling
//! compiles in a `:then` — true there, false here. grok agrees the `:wat::core::` spelling is
//! illegal (`the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error`, unmodified).
//! See each inverted test's own doc for the full citation and for how it still proves the walker
//! fix below rather than merely agreeing with the fence.
//!
//! `walk_nested_constructors` (`src/rete/validate/mod.rs`) recognised one head —
//! `:wat::core::kwargs-construct` — and otherwise *"recursed into every item anyway"*. So it
//! descended into a `match` form's **arm patterns** as if they were value expressions. An arm
//! `(:mac::E::A true)` has an enum-variant keyword at `items[0]`, `matcher::enum_variant_ctor`
//! resolved it, and the arity branch fired the variant's **0** declared fields against the arm's
//! **1** item. The diagnostic was `RhsArityMismatch` naming a `:then` insert of `:mac::E::A` — **an
//! insert that appears nowhere in the source.**
//!
//! It survived only by coincidence of spelling. `((:mac::E::A) true)` puts a *List* at `items[0]`,
//! keyword extraction fails, and the form falls to the generic recursion untouched. So whether a
//! legal `match` compiled in `:then` depended on which of two equivalent spellings the author
//! picked — and the byte-identical expression was accepted unchanged in the `where` fence.
//!
//! ## Why this file, and why FIVE fixtures
//!
//! Because the cheapest wrong cure — *stop walking `match` forms at all* — makes the first two
//! tests below green and silently retires four error kinds inside every arm BODY. That is not a
//! hypothetical: `strike-nested-wall`, one strike earlier in this same arc, found exactly that
//! shape at exactly this function, with `UnknownField`, `RhsMissingFields`, `RhsArityMismatch` and
//! `RhsPositionalConstructionRetired` all unreachable and every gate green.
//! [`a_misspelled_constructor_in_a_match_arm_body_is_still_refused`] is the only test here that
//! separates the two cures, and [`a_correct_constructor_in_a_match_arm_body_still_fires`] is its
//! control — without it, "refused" is indistinguishable from "refuses everything".
//!
//! ## AGREEMENT, not compilation, is the assertion
//!
//! Three fixtures carry the identical rule in three spellings and print the FIRED VALUES
//! (`true=2 false=1`) rather than merely loading. "All three compile" is satisfied by a cure that
//! throws the match away; only agreement on what the match *evaluated to* is not.
//!
//! ## The banked repro is retired here
//!
//! `docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-then-match.wat` was banked
//! 2026-08-30 carrying `;; rune:lint(red-by-design)` and its own disposal condition: *"If this file
//! ever loads, D5 is cured and the rune must go with it."* The rune is gone, which hands the pair
//! to `tests/lint/docs_wat_loads_or_declares_why_not.rs` as a standing regression gate — a rune
//! EXEMPTS a file from that gate's load check, so removing it is what puts the file back under it.
//! [`the_banked_d5_repro_pair_both_load`] drives the pair here as well, because that lint asks only
//! "does it load" of the whole `docs/arc/` tree and would not say WHICH file regressed or why.

use std::path::Path;
use std::process::{Command, Stdio};

/// ⚠ RUNS FROM THE MANIFEST DIR WITH A RELATIVE PATH, deliberately — a refusal's `Span` carries
/// `:file` verbatim, so an absolute path would make the diagnostic machine-dependent and no `.edn`
/// golden over it could ever be checked in. Same reason `probe_arc278_nested_wall.rs` states.
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

/// ★ INVERTED BY A BUILDER RULING (`BRIEF-7j-ADDENDUM-324-the-then-match-collision.md`,
/// 2026-09-16, 4-YES OPTION A). grok's own version of this test asserted the bare and wrapped
/// spellings COMPILE and agree. On THIS tree that premise is false by a permanent, ACCEPTED,
/// main-side design decision this replay's batch anchor already carries: `250162a0e SCORE(277)`
/// (`git merge-base --is-ancestor 250162a0e 665b17b60` succeeds; the commit is entirely absent
/// from `origin/grok-rete` and never revisited by any of grok's own remaining ~300 commits) makes
/// `wat/rete/compile.wat`'s then-item fence (`then-item-contains-match?`, walked recursively over
/// the WHOLE `:then` item subtree, not just its own top level) refuse `:wat::rete::core::match`
/// outright, for ANY spelling, exhaustive or not. Its own reasoning, verbatim: *"a `:then` admits
/// only what the fence can prove total, `total?`'s contract is head-level, match is a total HEAD,
/// and a match's exhaustiveness belongs to ITS ARMS. The axis cannot tell an exhaustive match from
/// a partial one, so admitting the good one means admitting both. It refuses both."*
///
/// grok agrees `:wat::core::match` is illegal in a `:then`
/// (`the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error`, unmodified, already
/// passes here) — only the `:wat::rete::core::` spelling collides, because grok's own branch never
/// carries `250162a0e` at all.
///
/// ★ THIS TEST MUST STILL PROVE THE D5 WALKER FIX, not merely that the fence refuses — asserting
/// only `!ok` would pass whether or not the fix landed, since BOTH the pre-fix walker (a freeze-time
/// refusal) and the post-fix fence (a runtime refusal) produce `!ok`. What discriminates them is
/// the diagnostic's CONTENT: PRE the fix, `walk_nested_constructors` read the arm PATTERN
/// `(:mac::E::A true)` as a constructor CALL and died at FREEZE with a fabricated
/// `RhsArityMismatch` naming a `:then` insert of `:mac::E::A` that appears nowhere in the source.
/// POST the fix (confirmed separately: both fixtures pass `./target/release/wat --check`, rc 0 —
/// freeze succeeds), the SAME expression reaches `compile-all` at runtime and is refused by the
/// fence, naming the TRUE Stone-C axis (form-level arm exhaustiveness vs. head-level fence
/// totality) instead. Asserting the absence of `RhsArityMismatch` plus the presence of that axis
/// text is exactly what `the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error`
/// already does for the core spelling; this is the same shape for the rete spelling.
///
/// AGREEMENT survives the inversion too: after `convert.sh`'s `match-arm-to-bracket-map-pattern`,
/// grok's "bare" and "wrapped" arm-pattern spellings become BYTE-IDENTICAL past the header comment
/// (main's syntax has exactly one match-arm-pattern form) — confirmed by diff. So both fixtures
/// must refuse with the identical message, differing only in the fixture's own path inside the
/// frame trace.
#[test]
fn the_bare_and_wrapped_then_spellings_compile_and_agree() {
    let (bare_ok, bare_out, bare_err) = run("tests/rete/probe_arc278_match_arm_then_rete_bare.wat");
    assert!(
        !bare_ok,
        "`:wat::rete::core::match` anywhere inside a `:then` item's subtree is refused outright by \
         `250162a0e`'s then-item fence, exhaustive or not — a permanent main-side ruling this batch's \
         own anchor already carries, not a landing defect\n{bare_out}{bare_err}"
    );
    let (wrapped_ok, wrapped_out, wrapped_err) =
        run("tests/rete/probe_arc278_match_arm_then_wrapped.wat");
    assert!(
        !wrapped_ok,
        "the WRAPPED spelling collides with the identical fence — after conversion to main's \
         bracket-map match-arm syntax the bare and wrapped bodies are byte-identical past the \
         header comment, so they must refuse identically\n{wrapped_out}{wrapped_err}"
    );
    let bare_face = format!("{bare_out}{bare_err}");
    let wrapped_face = format!("{wrapped_out}{wrapped_err}");
    // rune:lint(loose-assert) — a targeted ABSENCE on a large output, matching the exemption
    // `the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error` already uses for
    // the identical check. This is the whole point of the test: the phantom kind must be gone.
    assert!(
        !bare_face.contains("RhsArityMismatch") && !wrapped_face.contains("RhsArityMismatch"),
        "the D5 walker fix's whole point: neither refusal may be the phantom arity error the \
         un-fixed walker fabricated by reading an arm PATTERN as a constructor CALL\n\
         bare: {bare_face}\nwrapped: {wrapped_face}"
    );
    // rune:lint(loose-assert) — the face embeds `wat/rete/compile.wat` line numbers and per-run
    // frame paths that move with any edit to that file, so an exact golden could not be
    // deterministic. Asserts the load-bearing sentence: the axis named is form-level vs
    // head-level, never a fabricated arity error.
    assert!(
        bare_face.contains("form-level") && bare_face.contains("head-level"),
        "the refusal must name the TRUE Stone-C reason (form-level arm exhaustiveness vs. \
         head-level fence totality), not a fabricated arity error\n{bare_face}"
    );
    assert_eq!(
        bare_face.replace("probe_arc278_match_arm_then_rete_bare", "probe_arc278_match_arm_then_X"),
        wrapped_face.replace("probe_arc278_match_arm_then_wrapped", "probe_arc278_match_arm_then_X"),
        "the two spellings' refusals must be identical once each fixture's own path is normalised \
         out of the frame trace — this is what AGREEMENT means after the inversion"
    );
}

/// The `:wat::core::match` spelling — measured to reach the walker VERBATIM, and gated here because
/// nothing else proves the walker's `resolve_core_name` indirection is load-bearing.
///
/// Measured by instrumenting `walk_nested_constructors` to `eprintln!` `items[0]` and driving both
/// fixtures: a `:then` operand delivers `:wat::rete::core::match` and `:wat::core::match`
/// un-lowered, and at HEAD each produced the SAME phantom `RhsArityMismatch` pair, at freeze.
///
/// The core spelling is nonetheless illegal in a `:then` — `wat/rete/compile.wat`'s then-item fence
/// admits only `:wat::rete::` ops — so this fixture still FAILS. What changed is WHICH wall refuses
/// it and what it says. Keyed on `:wat::rete::core::match` alone, the walker would still fabricate
/// an insert of an enum variant nobody constructed, and that mutation reddens exactly here.
#[test]
fn the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error() {
    let (ok, out, err) = run("tests/rete/probe_arc278_match_arm_then_core_bare.wat");
    assert!(
        !ok,
        "`:wat::core::match` is not admissible in a `:then` — the then-item fence must refuse \
         it\n{out}{err}"
    );
    let face = format!("{out}{err}");
    // rune:lint(loose-assert) — a targeted ABSENCE on a large output, which this lint's own message
    // names as the exemption. This is the whole point of the test: the phantom kind must be gone.
    assert!(
        !face.contains("RhsArityMismatch"),
        "the freeze wall fabricated an arity error about an enum variant that is a PATTERN here, \
         not a call — this is the D5 defect itself\n{face}"
    );
    // rune:lint(loose-assert) — the face embeds `wat/rete/compile.wat` line numbers that move with
    // any edit to that file, so an exact golden could not be deterministic. Asserts the load-bearing
    // sentence: the refusal comes from the fence and NAMES the head it will not admit.
    assert!(
        face.contains("is not a rete primitive; a then admits only :wat::rete:: ops"),
        "the refusal must come from the then-item fence, naming the head\n{face}"
    );
}

/// MUTATION 2's CONTROL, INVERTED BY THE SAME BUILDER RULING as
/// [`the_bare_and_wrapped_then_spellings_compile_and_agree`] (see its doc for the full citation of
/// `250162a0e SCORE(277)`). grok's `:then` item here nests `:wat::rete::core::match` as the VALUE
/// of a constructor field (`:inner (match …)`), not at the item's own top level — but
/// `then-item-contains-match?` (`wat/rete/compile.wat:737`) walks every list/vector in the item's
/// subtree looking for the head, not merely the item's own call position, so nesting the match one
/// level down inside an otherwise-correct constructor does not escape the identical fence.
///
/// Still proves the D5 walker fix, the same way: `--check` on this fixture is rc 0 (freeze
/// succeeds — the arm PATTERN `:macb::E::A` is no longer misread as a constructor call), so the
/// refusal that DOES occur is purely the runtime fence at `compile-all`, never the walker's old
/// fabricated `RhsArityMismatch`. Asserting the diagnostic's content (axis text present, phantom
/// kind absent) is what distinguishes "the fix landed" from "nothing changed and it still dies
/// some other way" — a bare `!ok` cannot.
#[test]
fn a_correct_constructor_in_a_match_arm_body_still_fires() {
    let (ok, out, err) = run("tests/rete/probe_arc278_match_arm_body_ok.wat");
    assert!(
        !ok,
        "a `match` nested inside a `:then` constructor's field value still collides with \
         `250162a0e`'s then-item fence — the fence walks the whole item subtree, not only its own \
         top level, so nesting one level down inside a correct constructor does not escape it\n\
         {out}{err}"
    );
    let face = format!("{out}{err}");
    // rune:lint(loose-assert) — a targeted ABSENCE on a large output, same exemption as the
    // identical check above. The phantom kind must be gone regardless of where the match sits.
    assert!(
        !face.contains("RhsArityMismatch"),
        "the D5 walker fix's whole point: this must not be the phantom arity error the un-fixed \
         walker fabricated by reading the arm PATTERN as a constructor CALL\n{face}"
    );
    // rune:lint(loose-assert) — the face embeds `wat/rete/compile.wat` line numbers and per-run
    // frame paths that move with any edit to that file, so an exact golden could not be
    // deterministic. Asserts the load-bearing axis text, never a fabricated arity error.
    assert!(
        face.contains("form-level") && face.contains("head-level"),
        "the refusal must name the TRUE Stone-C reason (form-level arm exhaustiveness vs. \
         head-level fence totality), not a fabricated arity error\n{face}"
    );
}

/// ★★ MUTATION 2's GATE — **the only test here that separates the correct cure from "stop walking
/// `match` forms".**
///
/// Skipping the whole form makes every other test in this file green and darkens four error kinds
/// inside every arm body. `:macb::Inner` declares exactly `n`; the A-arm's body supplies `n` (so
/// `RhsMissingFields` cannot fire and this kind stands alone, per `strike-nested-wall`'s
/// one-kind-per-fixture rule) and also the undeclared `:nope`.
///
/// The golden carries the whole `Span`: `check_field_kw` takes the keyword NODE, so the caret must
/// be `:nope`'s own extent. A span over the enclosing form would mean the body was reached by some
/// other route than the arm-body recursion this strike added, and the kind alone would not say so.
#[test]
fn a_misspelled_constructor_in_a_match_arm_body_is_still_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_match_arm_body_bad.wat.bad");
    assert!(
        !ok,
        "a constructor naming an undeclared field is a freeze refusal wherever it sits — if this \
         program RAN, the cure stopped walking match forms instead of skipping their patterns, and \
         four error kinds are now dark inside every arm body\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_match_arm_body_bad.edn",
        "`UnknownField` must be the ONLY finding, and its caret must be `:nope`'s own extent"
    );
}

/// THE BANKED REPRO, retired into a gate.
///
/// `experiri-then-match.wat` refused at HEAD and `experiri-when-match.wat` — the byte-identical
/// expression in a `where` fence — loaded. That asymmetry WAS the finding. Both must now load, and
/// the `:then` file's `rune:lint(red-by-design)` is removed, which is what returns it to
/// `tests/lint/docs_wat_loads_or_declares_why_not.rs`'s load check.
///
/// Driven here as well as there because that lint asks one question of the whole `docs/arc/` tree
/// and would report only that *a* file stopped loading.
#[test]
fn the_banked_d5_repro_pair_both_load() {
    const THEN: &str = "docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-then-match.wat";
    const WHEN: &str = "docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-when-match.wat";
    let (then_ok, then_out, then_err) = run(THEN);
    assert!(
        then_ok,
        "the D5 repro must now load — it is the file whose own header says the rune goes with the \
         cure\n{then_out}{then_err}"
    );
    let (when_ok, when_out, when_err) = run(WHEN);
    assert!(when_ok, "the `where` fence was never affected and must be untouched\n{when_out}{when_err}");
    assert_eq!(
        then_out.trim(),
        when_out.trim(),
        "the pair exists to be compared: the same expression in `:then` and in the `where` fence \
         must no longer disagree about whether it is legal"
    );
    assert_eq!(then_out.trim(), "\"loaded\"", "both files print the same single word");
}
