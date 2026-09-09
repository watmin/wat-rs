//! PROBE — the `:wat::*` reserved-prefix blanket lets a PHANTOM head through `--check`, and
//! that acceptance is currently the ONLY thing keeping two corpus files legal.
//!
//! ## Why this probe exists
//!
//! Arc 255 Stone ④ (`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-4-the-corpus-blanket-
//! dependents.md`) found two `wat-scripts/` files that type-check ONLY because the blanket
//! accepts any `:wat::*` head, unconditionally, without asking whether it is a registered
//! builtin — `--check` never validates a `:wat::*` head at all. Both files were moved here, out
//! from under the loader gate's scan root (`tests/lint/wat_scripts_fixes_load.rs`, which requires
//! every `.wat` beneath `wat-scripts/` to `startup_from_source` cleanly), because their
//! containment premise IS the blanket: the moment it dies, both would turn the gate red in a way
//! that reads as an unrelated floor failure rather than as the blanket's own removal.
//!
//! ## What these rows pin, and what they do NOT pin
//!
//! ★★ Both rows below assert TODAY's measured behaviour — the blanket's defect, not its cure.
//! They are RATCHETS AIMED AT THE BLANKET: when it dies, `--check` starts exiting 1 on both
//! fixtures and these rows go RED, at exactly the moment they should, in tests whose names say
//! why. The blanket's own removal stone owns updating them; writing the post-blanket verdict now
//! would rot silently instead of ratcheting. `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`
//!
//! - `bogus_rete_head` — `:wat::rete::f64::>X` was never minted. `--check` exits 0 anyway (the
//!   blanket is opaque to the checker); running it raises a located `UnknownFunction` naming the
//!   phantom head, proving the acceptance is not a silent no-op.
//! - `dot_spelling_reaches_the_accessor` — `(:wat::core::Option.Some {:value 7})` is not the
//!   `Option::Some` constructor; the DOT spelling fails to resolve as a call head and falls
//!   through to the keyword-as-accessor path (`runtime.rs:3655`), which is total (a miss is
//!   `None`, never an error). The blanket is what lets a `:wat::core::` head reach that
//!   fall-through at all — a `:usr::` head is refused by resolve first. `--check` exits 0 and all
//!   five `println`s in the fixture succeed at `run`, today.

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

/// ★★ RATCHET — a bogus `:wat::rete::*` head type-checks today (the blanket does not validate
/// `:wat::*` heads at all) but raises a located `UnknownFunction` at runtime, naming the phantom
/// head. When the blanket dies, `--check` must start refusing this file, and this row goes red.
#[test]
fn bogus_rete_head_type_checks_today_but_raises_unknown_function_at_runtime() {
    assert_eq!(
        check("bogus_rete_head"),
        0,
        "the blanket accepts any :wat::* head at --check time, including a phantom one"
    );
    let (code, stdout, stderr) = run("bogus_rete_head");
    assert_ne!(code, 0, "expected the phantom head to fail at runtime.\nstdout:\n{stdout}");
    assert!(
        stdout.is_empty(),
        "the raise happens before the println completes, so nothing should reach stdout:\n{stdout}"
    );
    // Structure-exact against a golden, not a `contains` probe: the span is machine-independent
    // because `run` invokes the binary from CARGO_MANIFEST_DIR with a RELATIVE fixture path.
    wat::assert_edn_eq!(
        stderr,
        include_str!("probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head_stderr.edn")
    );
}

/// ★★ RATCHET — the DOT spelling of `Option.Some` does not resolve as the `Option::Some`
/// constructor; it falls through to the keyword-as-accessor path, which is total. `--check`
/// exits 0 and every `println` in the fixture succeeds. When the blanket dies, the two
/// `:wat::core::Option.Some` lines stop reaching that fall-through and are refused at resolve
/// instead — this row goes red at exactly that moment.
#[test]
fn dot_spelling_reaches_the_keyword_accessor_fallthrough_today() {
    assert_eq!(
        check("dot_spelling_reaches_the_accessor"),
        0,
        "the blanket lets the :wat::core:: head reach the accessor fall-through at --check time"
    );
    let (code, stdout, stderr) = run("dot_spelling_reaches_the_accessor");
    assert_eq!(code, 0, "today all five printlns must succeed.\nstderr:\n{stderr}");
    // The fixture prints FIVE EDN values, one per line; bracketing both sides makes one EDN
    // value so the comparison is STRUCTURAL against a golden rather than a string literal.
    // The bracket pair is STRUCTURAL scaffolding, not golden content — the golden itself is the
    // co-located `.edn` file. Kept as lone delimiters so the assembled value is EDN without a
    // literal that carries any of it.
    let open = "[";
    let close = "]";
    wat::assert_edn_eq!(
        format!("{open}\n{stdout}{close}\n"),
        include_str!("probe_arc255_the_blanket_hides_a_phantom_head__dot_spelling_stdout.edn")
    );
}
