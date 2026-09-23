//! Probe (excursus 003, stone A) — **A NON-POSITIVE LRU CAPACITY IS A WAT VALUE, NOT A RUST PANIC.**
//!
//! Cures the-little-wat **F-084**. At `3155207ac` a wat program that called
//! `:wat::cache::Lru/new` with `0` got a Rust `panic!` through the dispatch shim:
//!
//! ```text
//!   thread 'main' panicked at src/rust_deps/cache.rs:103:13:
//!   :rust::cache::Lru/new: capacity must be positive; got 0
//!   note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! Three separate injuries in four lines: a **Rust backtrace note** offered to someone who never
//! wrote Rust, the **internal `:rust::` shim name** instead of the verb they typed, and **no span
//! at all** — nothing pointing at the line that passed the `0`.
//!
//! ## The mandate, and why the old ruling's axis was wrong
//!
//! `docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md`
//! split these panics on *programming-error vs fallible input*. The substrate does not use that
//! axis: `:wat::i64::/` is annotated `@Totality Partial` — a divide-by-zero is a caller bug by any
//! reading — and it still refuses INSIDE the language, carrying the user's file, line and column.
//! Partiality never licensed a panic. The line is **a refusal must arrive as a wat value**, and
//! the builder's mandate (2026-09-22, *"lru-new can return a result - panic inducing behavior is
//! bad"*) is recorded in
//! `docs/excursus/2026/09/003-the-little-wat-findings/DESIGN-stone-A-lru-new-returns-a-result.md`.
//!
//! ## What the four fixtures divide between them
//!
//! | fixture | the question it answers |
//! |---|---|
//! | `lru_new_zero_is_a_fault_value` | is the refusal a VALUE a program can match? does it name the verb the user typed? |
//! | `holographic_lru_new_zero_is_a_fault_value` | does the `Result` PROPAGATE through the composite, carrying the same fault? |
//! | `lru_new_zero_unhandled` | when nobody handles it, what face does the death wear? |
//! | `holographic_lru_new_zero_unhandled` | the same, one level up |
//!
//! ⭐ The two `*_is_a_fault_value` fixtures are the load-bearing pair. The two `*_unhandled` ones
//! pin the *diagnostic*, and they matter because F-084 was never about liveness — the program died
//! before and dies now. What the stone changed is what the person reading stderr is told.
//!
//! ## ⛔ The driver is the binary, twice — the same contract decision as the sibling probe
//!
//! `probe_ex003_silent_failure_pair.rs`'s header states it in full and it is not re-derived here:
//! every the-little-wat finding is a claim about what a user experiences running `wat foo.wat`,
//! and the in-process `startup_from_file` driver does not evaluate, so it cannot see the half of
//! this probe that lives at run time.
//!
//! ## Mutation (this gate has been driven RED)
//!
//! `EXPECTATIONS` row 7 asks for it and this is the one that was run: with the `Result` SIGNATURE
//! left intact, `src/rust_deps/cache.rs`'s `new` was made to `panic!` on `capacity <= 0` again —
//! i.e. exactly the F-084 defect restored, with none of the surrounding plumbing disturbed.
//! ⚠ **One mutation, and it reaches ALL FIVE tests in this file** — measured, 5 passed -> 0
//! passed, 5 failed — because every fixture routes through that one guard: both
//! `*_is_a_fault_value` tests lose their stdout and their zero exit; both stderr goldens are
//! replaced by a face that is not even EDN (`thread 'main' panicked at src/rust_deps/cache.rs`,
//! which the golden macro refuses to parse); and `no_rust_panic_face_reaches_the_user` catches
//! all four of its needles at once.
//! The blunter mutation — deleting the `Result` from the signature — is NOT the one to use: it
//! stops `wat/cache.wat` compiling and reddens the whole floor, which proves the files are wired
//! together and nothing about this gate.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(case: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/diagnostics")
        .join(format!("probe_ex003_lru_new_refuses_as_a_value__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

/// One invocation of the binary against a fixture. Returns `(exit code, stdout, stderr)`.
fn run(case: &str) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// The whole stderr of a death, framed as ONE EDN document so a golden can pin it.
///
/// ⚠ **Why a frame is needed at all, and why it alters nothing.** A wat death writes TWO
/// top-level EDN documents on separate lines — the `#wat.kernel/AssertionFailure` the raise
/// emitted, then the `#wat.kernel/LociDiedError.Panic` the death-carrier wrapped it in — and
/// `wat_edn::parse_owned` reads exactly one document, so the raw face is not parseable as a
/// golden (it fails at the second document's `[`). The sibling probe never hit this: a
/// `RuntimeError` face is a single document.
///
/// This joins the lines into a vector. Every byte of both documents is preserved and compared,
/// in order; the brackets are the only thing added, and a third line appearing later would be
/// compared too rather than silently dropped. ⛔ It is NOT a filter — nothing is selected out,
/// which is the failure mode a `.lines().next()` here would have.
fn stderr_as_one_edn_doc(case: &str) -> String {
    let (_rc, _stdout, stderr) = run(case);
    let docs: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        !docs.is_empty(),
        "fixture `{case}` died silently — no stderr to pin. \
         An empty face would make this probe's golden vacuous."
    );
    // ⛔ The brackets are pushed, not written as a `format!` template. `tests/lint/no_inlined_edn.rs`
    // reads any string literal whose trimmed content opens `#`/`{`/`[`/`(` as an inlined EDN
    // golden, and a `[{}]` template is exactly that shape — it made the floor RED here. The lint's
    // own rubric says to restructure the code rather than rune a false positive, and the gate
    // beside this one already does it this way (`every_recorded_migration_replays.rs`'s
    // `default_path_vector`: *"Built from chars so the default stdin is not an inlined EDN string
    // literal."*). A lone `[` is excluded by the detector's `is_lone_delimiter` arm — no EDN
    // reader can complete it — so it is not a suppression, it is a literal that cannot be a golden.
    let mut framed = String::new();
    framed.push('[');
    framed.push_str(&docs.join("\n"));
    framed.push(']');
    framed
}

/// The refusal is a `Result.Err` the program matches, and its `diagnostic` names the verb the
/// USER typed — `:wat::cache::Lru/new`, never the `:rust::cache::Lru/new` shim behind it.
///
/// The exit code is half the assertion: a program that handles the refusal runs to completion.
/// Before the cure this same fixture could not exist — there was no `Err` arm to write.
#[test]
fn lru_new_zero_is_a_wat_value_naming_the_user_facing_verb() {
    let (rc, stdout, stderr) = run("lru_new_zero_is_a_fault_value");
    assert_eq!(
        rc, 0,
        "a HANDLED capacity refusal must not end the program. stderr:\n{stderr}"
    );
    assert_eq!(
        stdout, "\":wat::cache::Lru/new | capacity must be positive; got 0\"\n",
        "the Err payload changed shape or the diagnostic stopped naming the user-facing verb"
    );
}

/// The ONE contract decision, driven: the `Result` propagates through `HolographicLru/new`
/// instead of stopping there, and the fault it carries is `Lru/new`'s own — unaltered.
///
/// ⭐ The assertion is byte-identical to the sibling above, and that identity IS the content:
/// `HolographicLru/new` has no guard of its own, so a fault naming `HolographicLru/new` would
/// mean a second guard had been added — the thing EXPECTATIONS trap 3 forbids.
#[test]
fn holographic_lru_new_zero_propagates_lru_news_own_fault_unaltered() {
    let (rc, stdout, stderr) = run("holographic_lru_new_zero_is_a_fault_value");
    assert_eq!(
        rc, 0,
        "a HANDLED capacity refusal must not end the program. stderr:\n{stderr}"
    );
    assert_eq!(
        stdout, "\":wat::cache::Lru/new | capacity must be positive; got 0\"\n",
        "HolographicLru/new stopped propagating Lru/new's fault verbatim — either it swallowed \
         the Result or it grew a second guard of its own"
    );
}

/// An UNHANDLED refusal still ends the program — and now does it as a wat error whose
/// `:location` is the user's `.wat`, at the line and column that passed the `0`.
///
/// Captured, never hand-authored:
/// `UPDATE_EDN=1 cargo nextest run --release -E 'test(/lru_new_refuses/)'`.
/// The golden is safe against unrelated Rust edits: `assert_edn_eq!` blanks the `:line` of any
/// span whose `:file` is a `src/**.rs` path on BOTH sides (`blank_rust_source_lines`), so the
/// `src/freeze.rs` frame in this face cannot rot the gate.
#[test]
fn an_unhandled_lru_new_refusal_dies_as_a_wat_error_carrying_the_users_span() {
    let (rc, _stdout, _stderr) = run("lru_new_zero_unhandled");
    assert_eq!(rc, 2, "an unhandled refusal must still end the program");
    wat::assert_edn_matches_file!(
        stderr_as_one_edn_doc("lru_new_zero_unhandled"),
        "probe_ex003_lru_new_refuses_as_a_value__lru_new_zero_unhandled_stderr.edn",
        "the face of an unhandled capacity refusal changed — check it still carries the user's \
         file in :location and still names :wat::cache::Lru/new"
    );
}

/// The same, one level up — EXPECTATIONS row 4.
#[test]
fn an_unhandled_holographic_lru_new_refusal_dies_as_a_wat_error_carrying_the_users_span() {
    let (rc, _stdout, _stderr) = run("holographic_lru_new_zero_unhandled");
    assert_eq!(rc, 2, "an unhandled refusal must still end the program");
    wat::assert_edn_matches_file!(
        stderr_as_one_edn_doc("holographic_lru_new_zero_unhandled"),
        "probe_ex003_lru_new_refuses_as_a_value__holographic_lru_new_zero_unhandled_stderr.edn",
        "the face of an unhandled capacity refusal through HolographicLru/new changed"
    );
}

/// ⭐ **F-084's own complaint, asserted directly: none of the Rust panic's face reaches the user.**
///
/// The goldens above already pin the whole stderr, so this test cannot fail while they pass —
/// and it is here anyway, because a golden says *what the face is* and this says *what it must
/// never be*. When someone re-captures the goldens after a legitimate change, this is the
/// assertion that refuses a re-capture of a panic.
///
/// The four needles are the four things F-084 quotes: the backtrace invitation, the panic banner,
/// the Rust source file, and the internal shim name.
#[test]
fn no_rust_panic_face_reaches_the_user() {
    for case in ["lru_new_zero_unhandled", "holographic_lru_new_zero_unhandled"] {
        let (_rc, _stdout, stderr) = run(case);
        for needle in [
            "RUST_BACKTRACE",
            "panicked at",
            "src/rust_deps/cache.rs",
            ":rust::cache::Lru/new",
        ] {
            // rune:lint(loose-assert) — a TARGETED ABSENCE over a large output, the rubric's own
            // named exemption. The positive shape is pinned exactly by the two goldens above;
            // what is asserted here is that four specific strings are NOWHERE in the face, which
            // an equality check on a face that does not contain them cannot express.
            assert!(
                !stderr.contains(needle),
                "F-084 is back: `{needle}` reached a wat user's stderr from fixture `{case}`.\n\
                 stderr:\n{stderr}"
            );
        }
    }
}
