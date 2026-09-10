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
//! - `dot_spelling_reaches_the_keyword_accessor_fallthrough_today` (renamed
//!   [`dot_spelling_is_refused_at_resolve_not_silently_none`]) — `(:wat::core::Option.Some
//!   {:value 7})`, the DOT-spelled non-constructor, used to reach the keyword-as-accessor
//!   fall-through and silently yield `#wat.core/Option.None {}` — a wrong answer with no error.
//!   It is now refused at resolve instead: `--check` exits 1, and running the file no longer
//!   reaches ANY of its five `println`s (stdout is empty; the file never gets far enough to run).
//!   That silent-wrong-answer defect is exactly why the seam ordered the blanket's death BEFORE
//!   the dot flip (a follow-on stone) — its disappearance here is the evidence the ordering was
//!   right.
//!
//! Both fixtures' `.wat` header comments (unchanged — outside this stone's blast radius) still
//! narrate the PRE-deletion premise ("this file TYPE-CHECKS... despite never having been
//! minted"); that narration is now STALE prose describing a defect this stone retired. The
//! authority for current behaviour is this test file and its two `.edn` goldens, not the fixture
//! headers — a follow-on housekeeping pass owns refreshing that prose.
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

/// ★★ RATCHET FIRED — arc 255 Stone ⑤-F+G. Was
/// `dot_spelling_reaches_the_keyword_accessor_fallthrough_today`, pinning the blanket's defect:
/// the DOT spelling of `Option.Some` does not resolve as the `Option::Some` constructor, so it
/// used to fall through to the keyword-as-accessor path (total: a miss is `None`, never an
/// error) — `--check` exited 0 and all five `println`s in the fixture succeeded, silently
/// yielding `#wat.core/Option.None {}` for a real ctor name typo'd with a dot. The blanket is
/// what let a `:wat::core::` head reach that fall-through at all. It is dead: the two
/// `:wat::core::Option.Some` lines are now refused AT RESOLVE, before the accessor fall-through
/// is ever consulted. `--check` now exits 1, and running the file (no `--check`) fails identically
/// at startup — none of the five `println`s execute; stdout is empty.
#[test]
fn dot_spelling_is_refused_at_resolve_not_silently_none() {
    assert_eq!(
        check("dot_spelling_reaches_the_accessor"),
        1,
        "the blanket is dead: the dot-spelled head must now be refused at --check time instead \
         of reaching the keyword-accessor fall-through"
    );
    let (code, stdout, stderr) = run("dot_spelling_reaches_the_accessor");
    assert_ne!(
        code, 0,
        "expected startup to refuse the file before any println runs.\nstdout:\n{stdout}"
    );
    assert!(
        stdout.is_empty(),
        "resolve refuses the file before main ever runs, so none of the five printlns should \
         reach stdout any more:\n{stdout}"
    );
    // Recaptured 2026-09-09: was a stdout golden of five successful EDN prints (the second and
    // fourth silently `#wat.core/Option.None {}` — the wrong-answer defect this stone retires);
    // now a startup-time `UnresolvedReferences` (resolve, TWO refused references — one per
    // dot-spelled line), wrapped in `LociDiedError.StartupError`, on stderr.
    wat::assert_edn_eq!(
        stderr,
        include_str!(
            "probe_arc255_the_blanket_hides_a_phantom_head__dot_spelling_reaches_the_accessor_stderr.edn"
        )
    );
}
