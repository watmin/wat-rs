//! strike-match-arm-is-not-a-call (arc 278, work-list **D5**) — **A MATCH ARM'S PATTERN IS NOT A
//! CONSTRUCTOR CALL.**
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

/// ★ THE CURE — and it is stated as AGREEMENT between the two spellings, not as "it compiles".
///
/// PRE (driven at HEAD `d10ae67c4`): the bare fixture died at startup with two `RhsArityMismatch`
/// findings, on `:mac::E::A` and `:mac::E::B`, each "expects 0 positional argument(s); got 1". The
/// wrapped fixture printed `true=2 false=1`. Same expression, different verdict, decided by
/// parentheses.
///
/// Both must now fire and produce the SAME counts: `?ok` is `true` for the two `:mac::E::A` facts
/// and `false` for the one `:mac::E::B` fact, so the match is asserted to have discriminated,
/// per-fact, in the derived value — not merely to have been tolerated by the wall.
#[test]
fn the_bare_and_wrapped_then_spellings_compile_and_agree() {
    let (bare_ok, bare_out, bare_err) = run("tests/rete/probe_arc278_match_arm_then_rete_bare.wat");
    assert!(
        bare_ok,
        "a bare enum-variant match arm in `:then` is a legal program — a refusal here means the \
         walker is reading an arm PATTERN as a constructor call again\n{bare_out}{bare_err}"
    );
    let (wrapped_ok, wrapped_out, wrapped_err) =
        run("tests/rete/probe_arc278_match_arm_then_wrapped.wat");
    assert!(
        wrapped_ok,
        "the WRAPPED spelling compiled before the cure and must still — if this reddens, the fix \
         broke the path that already worked\n{wrapped_out}{wrapped_err}"
    );
    assert_eq!(
        bare_out.trim(),
        "\"true=2 false=1\"",
        "the match must still DISCRIMINATE: two `:mac::E::A` facts derive `:ok true`, one \
         `:mac::E::B` derives `:ok false`. A cure that stopped walking match forms would also make \
         this program compile — this counts what it evaluated to\n{bare_out}{bare_err}"
    );
    assert_eq!(
        bare_out.trim(),
        wrapped_out.trim(),
        "AGREEMENT IS THE CURE. A fix that makes one spelling compile and not the other has moved \
         the coincidence, not removed it"
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

/// MUTATION 2's CONTROL. A CORRECT constructor nested in a match arm BODY must still compile and
/// fire with the right values.
///
/// Without this, its twin below proves only that something refuses — not that the arm body is
/// reachable at all. Two `:macb::E::A` facts take the `n 10` arm and one `:macb::E::B` takes
/// `n 20`, so the counts also re-assert that the match discriminated.
#[test]
fn a_correct_constructor_in_a_match_arm_body_still_fires() {
    let (ok, out, err) = run("tests/rete/probe_arc278_match_arm_body_ok.wat");
    assert!(ok, "a correctly-spelled nested constructor in an arm body is legal\n{out}{err}");
    assert_eq!(
        out.trim(),
        "\"n10=2 n20=1\"",
        "the arm bodies must still be EVALUATED and must still discriminate\n{out}{err}"
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
