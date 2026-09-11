//! PROBE — the `:wat::*` reserved-prefix blanket let a PHANTOM head through `--check`, and that
//! acceptance was, until this stone, the ONLY thing keeping two corpus files legal.
//!
//! ## Why this probe exists
//!
//! Arc 255 Stone ④ (`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-4-the-corpus-blanket-
//! dependents.md`) found two `wat-scripts/` files that type-checked ONLY because the blanket
//! accepted any `:wat::*` head, unconditionally, without asking whether it is a registered
//! builtin — `--check` never validated a `:wat::*` head at all. Both files were moved here, out
//! from under the loader gate's scan root (`tests/lint/wat_scripts_fixes_load.rs`, which requires
//! every `.wat` beneath `wat-scripts/` to `startup_from_source` cleanly), because their
//! containment premise WAS the blanket: the moment it died, both would turn the gate red in a way
//! that reads as an unrelated floor failure rather than as the blanket's own removal.
//!
//! ## ★★★ THE RATCHET FIRED, 2026-09-09 (arc 255 Stone ⑤-F+G, "the blanket dies")
//!
//! Both rows below used to assert the blanket's DEFECT, not its cure — pinned in stone ④,
//! verified non-vacuous THEN by applying stone ⑤-F+G's own deletion (`src/resolve/walk.rs`,
//! `is_resolvable_call_head`: `if is_reserved_prefix(head) { return true; }` replaced by a lookup
//! into `crate::intrinsic::registry()`). That deletion has now LANDED, and both rows have flipped
//! from green to red exactly as designed — this file is the recapture of that flip, which IS the
//! stone's evidence, not incidental maintenance:
//!
//! - `bogus_rete_head_type_checks_today_but_raises_unknown_function_at_runtime` (renamed
//!   [`bogus_rete_head_is_refused_at_check_the_blanket_is_dead`]) — `:wat::rete::f64::>X`, a head
//!   that was never minted and has type-checked clean for as long as it has existed, now fails
//!   `--check` (exit 0 → exit 1). It no longer reaches runtime to raise `UnknownFunction` at all —
//!   `resolve` refuses it BEFORE `main` ever runs, one pass earlier than the blanket ever let this
//!   probe look.
//! - `dot_spelling_reaches_the_keyword_accessor_fallthrough_today` → `dot_spelling_is_refused_at_
//!   resolve_not_silently_none` → ⛔ **RETIRED 2026-09-10 — and the retirement IS the finding.**
//!   `(:wat::core::Option.Some {:value 7})` was a PHANTOM head reaching the keyword-accessor
//!   fall-through, silently yielding `#wat.core/Option.None {}`. The DOT FLIP then made that
//!   spelling CANONICAL: `Option.Some` IS the constructor, and `Option::Some` is now the name that
//!   does not resolve. The assertion did not go slightly stale — **its subject changed sides**, one
//!   day after it was written. It is replaced by a PAIR, because asserting only that the new
//!   spelling WORKS would pass just as well in a world where BOTH spellings work, and that is
//!   precisely the pre-flip world this arc ordered the blanket's death to escape:
//!   replaced by the PAIR [`the_dot_spelling_is_the_constructor_and_resolves`] (check 0,
//!   runs, structural stdout golden) and [`the_colon_spelling_no_longer_resolves`]
//!   (check 1, refused at RESOLVE, stderr golden).
//!
//! ⚠ **It did not fail the way the retired assertion predicted.** The fixture stopped failing at
//! resolve altogether and began failing at CHECK (`MalformedForm`), on a line the flip made illegal
//! for an unrelated reason: a ctor payload key that had been an ordinary map key while the head was
//! a phantom. Re-capturing the golden would have pinned that accident and called it the ruling — so
//! the fixture was REWRITTEN, and the dead line's property (the accessor finds a key that is there)
//! is carried by `(:value {:value 7})`, which needs no phantom to make its point.
//!
//! The `dot_spelling_is_the_constructor` / `colon_spelling_is_refused` fixtures carry this story in
//! their own headers. The `bogus_rete_head` fixture's header still narrates its PRE-deletion
//! premise and remains stale prose; this test file and its goldens are the authority.
//!
//! `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The fixture's path RELATIVE to the crate root, run with `current_dir` set there — so any span
/// captured in an assertion records exactly this relative string on every machine.
fn rel(case: &str) -> String {
    format!("tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__{case}.wat")
}

fn run_check(case: &str) -> (i32, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(rel(case)).exists(), "fixture missing: {}", rel(case));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg("--check")
        .arg(rel(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn check(case: &str) -> i32 {
    run_check(case).0
}

/// Actually RUNS the fixture (no `--check`) and returns (exit code, stdout, stderr) separately —
/// `dot_spelling_reaches_the_accessor` is asserted on stdout content, `bogus_rete_head` on
/// stderr content, and conflating the two streams would hide which one carried the proof.
fn run(case: &str) -> (i32, String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(rel(case)).exists(), "fixture missing: {}", rel(case));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg(rel(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// ★★ RATCHET FIRED — arc 255 Stone ⑤-F+G. Was
/// `bogus_rete_head_type_checks_today_but_raises_unknown_function_at_runtime`, pinning the
/// blanket's defect: a bogus `:wat::rete::*` head type-checked (`--check` exit 0) because the
/// blanket never validates `:wat::*` heads at all, and only raised `UnknownFunction` once it
/// actually ran. The blanket is dead: `is_resolvable_call_head` now asks
/// `crate::intrinsic::registry()` instead of accepting any reserved prefix, and `:wat::rete::f64::>X`
/// was never minted into that registry. `--check` now REFUSES the file (exit 1), and running it
/// (no `--check`) fails identically at STARTUP — before `main` ever executes, so no
/// `UnknownFunction` raise happens any more; there is no runtime to reach.
#[test]
fn bogus_rete_head_is_refused_at_check_the_blanket_is_dead() {
    assert_eq!(
        check("bogus_rete_head"),
        1,
        "the blanket is dead: a bogus :wat::* head must now be refused at --check time"
    );
    let (code, stdout, stderr) = run("bogus_rete_head");
    assert_ne!(
        code, 0,
        "expected the phantom head to be refused at startup, before main runs.\nstdout:\n{stdout}"
    );
    assert!(
        stdout.is_empty(),
        "resolve refuses the file before main ever runs, so nothing should reach stdout:\n{stdout}"
    );
    // Structure-exact against a golden, not a `contains` probe: the span is machine-independent
    // because `run` invokes the binary from CARGO_MANIFEST_DIR with a RELATIVE fixture path.
    // Recaptured 2026-09-09: was a runtime `UnknownFunction` raise; now a startup-time
    // `UnresolvedReferences` (resolve), wrapped in the same `LociDiedError.StartupError` shape.
    wat::assert_edn_eq!(
        stderr,
        include_str!("probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head_stderr.edn")
    );
}

/// ★★ THE RATCHET INVERTED — and it is the inversion, not the red, that is the finding.
///
/// This was `dot_spelling_is_refused_at_resolve_not_silently_none`, pinning arc 255 Stone ⑤-F+G:
/// the DOT spelling of `Option.Some` was a PHANTOM head that fell through to the keyword-accessor
/// path and silently answered `#wat.core/Option.None {}`. Then the dot flip made that spelling
/// CANONICAL. The assertion did not become slightly wrong; its subject changed sides.
///
/// ⚠ **And it did not fail the way the retired assertion predicted.** The fixture stopped failing
/// at RESOLVE altogether and began failing at CHECK — `MalformedForm`, on a line the flip made
/// illegal for an unrelated reason (a ctor payload key that used to be a plain map key). A stale
/// assertion does not merely go red; it goes red for a reason that is not the one it names. That
/// is why the fixture was REWRITTEN rather than re-goldened: re-capturing the golden would have
/// pinned an accident and called it the ruling.
#[test]
fn the_dot_spelling_is_the_constructor_and_resolves() {
    assert_eq!(
        check("dot_spelling_is_the_constructor"),
        0,
        "post-flip `Option.Some` IS the constructor — it must type-check"
    );
    let (code, stdout, stderr) = run("dot_spelling_is_the_constructor");
    assert_eq!(code, 0, "the file must RUN; stderr:\n{stderr}");
    // Three deterministic lines, asserted byte-identical (not `contains`): the ctor's payload
    // survives, the accessor misses TOTALLY (None, not an error), and it hits when the key is
    // there. The middle line is the one that used to be a silent wrong ANSWER for a real name.
    // ⛔ NOT an inline EDN string literal — `tests/lint/no_inlined_edn.rs` bans that, and
    // it caught this exact site when the first cut of this test spelled the three reads
    // as a byte-exact literal. The FIXTURE was restructured to print ONE vector so the
    // whole answer is a single EDN value, compared STRUCTURALLY against a golden.
    wat::assert_edn_matches_file!(
        stdout,
        "probe_arc255_the_blanket_hides_a_phantom_head__dot_spelling_is_the_constructor_stdout.edn"
    );
}

/// ★★★ THE OTHER HALF, and without it the pair proves nothing. A test asserting only that the new
/// spelling WORKS passes identically in a world where BOTH spellings work — which is the world
/// that existed before the flip, and exactly the silent-wrong-answer hazard arc 255 ordered the
/// blanket's death to prevent. `Option::Some` must now be the REFUSED one, at RESOLVE, before the
/// keyword-accessor fall-through is consulted, so `main` never runs and stdout stays empty.
#[test]
fn the_colon_spelling_no_longer_resolves() {
    assert_eq!(
        check("colon_spelling_is_refused"),
        1,
        "post-flip `Option::Some` is not a name any more — it must be refused at --check"
    );
    let (code, stdout, stderr) = run("colon_spelling_is_refused");
    assert_ne!(code, 0, "startup must refuse before any println runs.\nstdout:\n{stdout}");
    assert!(
        stdout.is_empty(),
        "refused at resolve, so main never runs and nothing reaches stdout:\n{stdout}"
    );
    wat::assert_edn_matches_file!(
        stderr,
        "probe_arc255_the_blanket_hides_a_phantom_head__colon_spelling_is_refused_stderr.edn"
    );
}
