//! GATE — A DIAGNOSTIC MUST SAY THE SAME THING TWICE.
//!
//! Every `.wat.bad` in the tree is run through the `wat` binary **twice, in two fresh processes**,
//! and the two runs must produce byte-identical stdout, byte-identical stderr, and the same exit
//! code. That is the whole property, and it is a PROPERTY, not a fixture: it does not know what a
//! type variable is, so it catches the next source of run-to-run variance as readily as the one it
//! was written for.
//!
//! ## WHY (C19, arc 278)
//!
//! `InferCtx::fresh` (`src/check.rs`, ~line 485) hands out unification-variable ids from a
//! monotonic counter, and three renderers printed that id straight into user-facing diagnostics
//! (`check::format_type`, `check::format_type_inner`, `freeze::format_type_expr`). The counter
//! encodes HOW MANY variables had been allocated before this one — and some traversal upstream is
//! `HashMap`-ordered, so under Rust's per-process random hasher that count differs every run:
//!
//! ```text
//! $ for i in 1 2 3 4 5; do ./target/release/wat tests/function/probe_arc247_hof_coll_first.wat.bad 2>&1 | md5sum; done
//! 70421fe5e07c4b83cd21936fe71f73d2   expects (:wat::stream::Stream :- [:?6984])
//! 30845554fdb827b0111a3ff42389336b   expects (:wat::stream::Stream :- [:?4675])
//! 10e496ce7380f1bf8d54475d127bed5e   …
//! 6ad4c615a7fa7623cae0e14b8d0472b2   …
//! 2d60f2f89976079e6914b44e3b33d361   …
//! ```
//!
//! Same binary, same file, five runs, five outputs. It was already being paid for: the D11 corpus
//! scan produced **9 false "regressions"** from it and had to normalise `:?[0-9]+` away to get a
//! usable answer. The three renderers now emit the wildcard `_`; this gate is what keeps them
//! honest, and what makes the NEXT one a red build instead of folklore.
//!
//! ## THE THREE FILES THIS GATE DOES NOT ASSERT OVER — A FINDING, NOT A FLAKE LIST
//!
//! ⛔ These are **not** "known flakes". Each is a SECOND, INDEPENDENT source of nondeterminism,
//! captured, characterised, and reproducible on demand — a defect this strike found and did not
//! have licence to fix (its blast radius was the renderers). They are listed here so the gate can
//! ship green over the other 278 instead of being deleted, and each carries the evidence needed to
//! re-check it in ten seconds. None of them is a type-variable problem; normalising `:?[0-9]+`
//! makes none of them stable.
//!
//! Reproduce any of them with:
//! ```text
//! for i in $(seq 1 30); do ./target/release/wat <path> 2>&1 | sed 's/:?[0-9]*/:?N/g' | md5sum; done | sort | uniq -c
//! ```
//! Each comes back with TWO hashes at roughly 50/50 (measured 10/20 and 16/14 over 30 runs).
//!
//! ⚠ AND NOTE HOW THE THIRD ONE WAS FOUND, because it is the methodological finding of this
//! strike. A 2-run scan of the corpus reported exactly TWO offenders. It was wrong: a defect whose
//! two outcomes are ~50/50 is invisible to a 2-run scan HALF THE TIME, so a clean 2-run sweep is
//! evidence about the sweep, not about the corpus. `probe_arc170_c2_mixed_macro_swap.wat.bad` was
//! caught only when this gate itself ran — and a 24-run-per-file sweep of all 280 was then needed
//! to close the set at three. Any future audit of this property must sample deeply, not twice.
//!
//! 1. **`tests/rete/probe_arc278_rete_defn_recurse_mutual.wat.bad` — the error's IDENTITY varies.**
//!    A mutual-recursion pair; which member is named as the offender flips run to run, and the
//!    reported LINE flips with it:
//!    ```text
//!    < :probe::b   … :line 5      >  :probe::a   … :line 8
//!    ```
//!    A reader is told a different function is at fault depending on the hash seed.
//!
//! 2. **`tests/services/probe_arc170_w2a_kwargs_check_mint_swap.wat.bad` — the error ORDER varies.**
//!    The same four errors every run; the `:wat::core::match` scrutinee mismatch at line 40 appears
//!    FIRST in some runs and LAST in others, with the other three unmoved.
//!
//! 3. **`tests/services/probe_arc170_c2_mixed_macro_swap.wat.bad` — the error ORDER varies.**
//!    Same shape as (2) at larger scale: the same NINE errors every run, but they arrive as two
//!    blocks that swap — seven `:wat::core::match` scrutinee mismatches (lines 91-127) and two
//!    `:probe::enrich::kwargs-check::Kwargs` parameter mismatches (line 156) — so a reader diffing
//!    two runs sees nine moved errors and zero real changes.
//!
//! All three sit in the SAME class: a `HashMap`-ordered traversal upstream deciding which errors
//! are emitted, and in what order, per process. That is the STOP-1 territory this strike was
//! explicitly barred from ("do not chase the traversal to determinism"), and it is a strictly
//! larger job than a renderer.
//!
//! ★ Note what this contradicts: the C19 DESIGN's bounding table asserts that "error kinds, their
//! ORDER, spans, message text" are **stable** and that normalising `:?N` makes runs byte-identical.
//! That held over the 120 files it sampled; over all 280 it is false for these two. The bound was
//! measured on a subset and read as a property.
//!
//! ## WHAT THIS GATE DELIBERATELY DOES NOT DO
//!
//! It does NOT assert that the two quarantined files are STILL nondeterministic. That assertion is
//! attractive — it would make the list self-expiring — and it was rejected on purpose: with each
//! file's two outcomes at ~50/50, "observe at least two distinct outputs in N runs" is wrong with
//! probability `2 * 0.5^(N-1)`, which at any N cheap enough to run is a genuine, if rare, false
//! RED. This repo does not ship a test that can fail for a reason other than the defect. What is
//! asserted instead is deterministic and still blocks silent rot: the list's LENGTH is pinned (a
//! fourth entry cannot be slipped in without editing a number and writing a reason) and every
//! quarantined path must still EXIST (a rename or deletion forces the list to be revisited).
//!
//! ★ AND STATE THE GATE'S OWN SENSITIVITY, so nobody reads a green here as a proof of determinism.
//! Two runs catch a variance that is ~50/50 only half the time. A green floor is therefore evidence
//! that no file went nondeterministic-AND-lost-the-coin-flip, not evidence that none did. Across 277
//! files and repeated floors a real regression surfaces quickly, but a SINGLE green run does not
//! close the question — which is exactly the mistake the 2-run corpus scan above made. Raising the
//! per-file run count is the lever if that ever matters; it costs linearly (see `.config/nextest.toml`).
//!
//! ## SHARDING
//!
//! [`N_SHARDS`] tests, each taking every `(index % N_SHARDS)`-th path of the sorted corpus, so
//! nextest parallelises them and a failure names which shard broke. Measured: `wat` costs ~0.30s
//! per run on this corpus (~0.16s of that is startup alone), so the full corpus twice is ~167s of
//! CPU — ~33s wall at 8-way parallelism. One test carrying all of it would blow the default
//! 15s/30s deadline outright; a shard is ~17 files (~35 runs, ~10.4s alone). See
//! `.config/nextest.toml` for the budget and its derivation.

use std::path::{Path, PathBuf};
use std::process::Command;

/// How many parallel shards the corpus is split across. See the SHARDING note above: chosen so a
/// shard's isolated cost (~10.4s) sits inside a budget with real margin under this repo's measured
/// 3.5x–4.4x floor-contention band, rather than needing a deadline raise per corpus growth.
const N_SHARDS: usize = 16;

/// Paths excluded from the byte-equality assertion because they carry a SECOND source of
/// nondeterminism, independent of the type-variable rendering this strike fixed. Each entry is
/// (path, the shape of the variance). See the module header for the captured evidence.
///
/// ⛔ NOT a flake list. Adding a row here means "I found another determinism defect and could not
/// fix it in this blast radius" — it must come with captured evidence in the header above and it
/// must move [`QUARANTINE_LEN`], which is what makes the addition deliberate.
const QUARANTINE: &[(&str, &str)] = &[
    (
        "tests/rete/probe_arc278_rete_defn_recurse_mutual.wat.bad",
        "which member of a mutual-recursion pair is named as the offender (and its line) flips run to run",
    ),
    (
        "tests/services/probe_arc170_w2a_kwargs_check_mint_swap.wat.bad",
        "the four errors are the same every run but their ORDER flips (the line-40 match mismatch moves first<->last)",
    ),
    (
        "tests/services/probe_arc170_c2_mixed_macro_swap.wat.bad",
        "the nine errors are the same every run but arrive as two blocks (7 match + 2 Kwargs) whose ORDER flips",
    ),
];

/// Pinned length of [`QUARANTINE`]. A third source of diagnostic nondeterminism cannot be absorbed
/// silently: it has to change this number, and changing it is the moment someone asks why.
const QUARANTINE_LEN: usize = 3;

/// The fixture that drives `check::format_type_inner`'s `Var` arm — the NESTED type renderer.
///
/// ⛔ IT EXISTS BECAUSE A MEASUREMENT SAID IT HAD TO. The C19 render fix touched two renderers in
/// `check.rs`, and the mutation proof for it is "break ONE of the two sites and the gate must still
/// go RED" — which proves both are covered rather than just the one the repro happens to exercise.
/// Marking the inner arm and sweeping all 280 `.wat.bad` returned **zero** hits: no file in the
/// corpus reached it, so that proof was unprovable for the inner site and a regression there would
/// have shipped green. This fixture closes the hole. See its own header for why a TUPLE is what it
/// takes.
const INNER_RENDERER_DRIVER: &str = "tests/lint/probe_c19_nested_type_var_render.wat.bad";

/// The `:got` field that fixture must produce. Two DISTINCT unresolved variables, both rendered as
/// the wildcard — one nested inside a parametric binder, one a direct tuple element.
///
/// This is the assertion that would catch a "stable but useless" fix: a renderer that dropped the
/// type information entirely, or emitted a constant for the whole type, would still be byte-stable
/// and would still pass every shard. It would not pass this.
const INNER_RENDERER_EXPECTED_GOT: &str = ":((wat::core::Vector :- [_]),_)";

fn collect_bad(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_bad(&p, out);
        } else if p.to_str().is_some_and(|s| s.ends_with(".wat.bad")) {
            out.push(p);
        }
    }
}

/// The sorted corpus, minus the quarantined paths. Sorted so a shard's membership is stable across
/// runs and machines — an unsorted `read_dir` would move files between shards and make a shard
/// failure unreproducible.
fn corpus() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_bad(Path::new("tests"), &mut paths);
    paths.sort();
    paths.retain(|p| {
        let s = p.to_str().expect("utf8 path");
        !QUARANTINE.iter().any(|(q, _)| *q == s)
    });
    paths
}

/// One fresh process. Returns (exit code, stdout, stderr) — all three, because a diagnostic can
/// vary on any of them and comparing only stdout would let a varying stderr through.
///
/// A FRESH PROCESS IS THE POINT: the variance this gate hunts comes from Rust's per-process random
/// `HashMap` seed, so two calls inside one process would share a seed and the gate would be green
/// by construction.
fn run_once(path: &Path) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let out = Command::new(bin)
        .arg(path)
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    (out.status.code(), out.stdout, out.stderr)
}

fn check_shard(shard: usize) {
    let paths = corpus();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS. The
    // floor sits well under the 278 non-quarantined `.wat.bad` this walk finds today (277 pre-existing + the C19 positive control), so it catches
    // a walk gone blind — a moved root, a renamed extension — without rotting as the corpus grows.
    assert!(
        paths.len() > 200,
        "the .wat.bad walk found only {} file(s) — it is not reaching the corpus it claims to \
         guard, so a green shard means nothing",
        paths.len()
    );

    let mut mine: Vec<&PathBuf> = paths.iter().skip(shard).step_by(N_SHARDS).collect();
    mine.sort();
    assert!(
        !mine.is_empty(),
        "shard {shard}/{N_SHARDS} covers no files — the sharding arithmetic is wrong and this \
         test's green is vacuous"
    );

    let mut failures = Vec::new();
    for path in mine {
        let (code_a, out_a, err_a) = run_once(path);
        let (code_b, out_b, err_b) = run_once(path);
        if code_a != code_b || out_a != out_b || err_a != err_b {
            failures.push(format!(
                "  {}\n    exit: {code_a:?} vs {code_b:?}\n    --- run 1 stdout ---\n{}\n    \
                 --- run 2 stdout ---\n{}\n    --- run 1 stderr ---\n{}\n    --- run 2 stderr ---\n{}",
                path.display(),
                String::from_utf8_lossy(&out_a),
                String::from_utf8_lossy(&out_b),
                String::from_utf8_lossy(&err_a),
                String::from_utf8_lossy(&err_b),
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} file(s) in shard {shard}/{N_SHARDS} produced DIFFERENT output on two runs of the same \
         binary over the same source. A diagnostic that changes between runs makes every output \
         diff — a rider's, a reviewer's, a script's — noise that looks like signal.\n\n{}\n\n\
         Both runs are printed in full above; do NOT summarise them when reporting.",
        failures.len(),
        failures.join("\n\n")
    );
}

/// The quarantine cannot rot silently: its length is pinned, and every path in it must still exist.
/// A renamed or deleted fixture therefore forces the list to be revisited rather than leaving a
/// stale exclusion that quietly widens what this gate does not look at.
#[test]
fn the_determinism_quarantine_is_pinned_and_its_paths_exist() {
    assert_eq!(
        QUARANTINE.len(),
        QUARANTINE_LEN,
        "the diagnostic-determinism quarantine changed size. Every entry is a SECOND determinism \
         defect that this gate stops asserting over — adding one is a finding to report, not a \
         line to slip in. Update QUARANTINE_LEN and write the captured evidence into this file's \
         header."
    );
    for (path, why) in QUARANTINE {
        assert!(
            Path::new(path).exists(),
            "quarantined path {path} no longer exists (it was excluded because: {why}). Either it \
             was renamed — update the entry — or it was deleted, in which case remove the entry so \
             the gate stops carrying a dead exclusion."
        );
    }
}

/// POSITIVE CONTROL — the nested renderer is DRIVEN, and what it renders still names the types.
///
/// Two things a shard cannot check on its own:
///
/// 1. **That `check::format_type_inner`'s `Var` arm is exercised at all.** No other `.wat.bad` in
///    the corpus reaches it (measured: 0 of 280), so without this the inner renderer could regress
///    to printing an allocator counter and every shard would still be green — the counter would be
///    the same in both runs of a shard only if it were stable, but the whole point is that it is
///    NOT, so in fact a regression there WOULD red a shard... only because this file is in the
///    corpus. Remove it and the coverage goes with it.
/// 2. **That the rendering still says something.** Byte-stability is satisfied by a renderer that
///    prints nothing useful. The `assert_eq!` below pins the whole `got` field, so a fix that
///    achieved stability by losing the type information fails here and only here.
#[test]
fn the_nested_type_renderer_is_driven_and_renders_no_counter() {
    let path = Path::new(INNER_RENDERER_DRIVER);
    assert!(
        path.exists(),
        "{INNER_RENDERER_DRIVER} is missing — it is the ONLY file in the corpus that reaches \
         `check::format_type_inner`'s Var arm, so without it that renderer is untested"
    );
    let (_code, _out, err) = run_once(path);
    let text = String::from_utf8_lossy(&err).into_owned();

    // Extract the `:got "…"` field rather than matching loosely inside the whole diagnostic: the
    // extraction PANICS with the full output if the shape ever changes, so it cannot silently
    // degrade into a check that passes over nothing.
    const KEY: &str = ":got \"";
    let start = text
        .find(KEY)
        .unwrap_or_else(|| panic!("no `{KEY}` field in the diagnostic for {INNER_RENDERER_DRIVER}; \
                                   the whole output was:\n{text}"))
        + KEY.len();
    let rest = &text[start..];
    let end = rest
        .find('"')
        .unwrap_or_else(|| panic!("unterminated `{KEY}` field for {INNER_RENDERER_DRIVER}; \
                                   the whole output was:\n{text}"));
    let got = &rest[..end];

    assert_eq!(
        got, INNER_RENDERER_EXPECTED_GOT,
        "the nested type renderer's output changed. A digit in place of a `_` means an allocator \
         counter is leaking into a diagnostic again (C19); anything else means the rendering \
         stopped naming the types. Whole diagnostic:\n{text}"
    );
}

/// Expand one `#[test]` per shard. Written out rather than looped so nextest can schedule them in
/// parallel and so a failure names WHICH shard — a single test carrying all 278 files would be one
/// long pole and one undifferentiated red.
macro_rules! shards {
    ($($name:ident = $idx:expr;)*) => {
        $(
            #[test]
            fn $name() { check_shard($idx); }
        )*
    };
}

shards! {
    every_wat_bad_diagnostic_is_byte_stable_shard_00 = 0;
    every_wat_bad_diagnostic_is_byte_stable_shard_01 = 1;
    every_wat_bad_diagnostic_is_byte_stable_shard_02 = 2;
    every_wat_bad_diagnostic_is_byte_stable_shard_03 = 3;
    every_wat_bad_diagnostic_is_byte_stable_shard_04 = 4;
    every_wat_bad_diagnostic_is_byte_stable_shard_05 = 5;
    every_wat_bad_diagnostic_is_byte_stable_shard_06 = 6;
    every_wat_bad_diagnostic_is_byte_stable_shard_07 = 7;
    every_wat_bad_diagnostic_is_byte_stable_shard_08 = 8;
    every_wat_bad_diagnostic_is_byte_stable_shard_09 = 9;
    every_wat_bad_diagnostic_is_byte_stable_shard_10 = 10;
    every_wat_bad_diagnostic_is_byte_stable_shard_11 = 11;
    every_wat_bad_diagnostic_is_byte_stable_shard_12 = 12;
    every_wat_bad_diagnostic_is_byte_stable_shard_13 = 13;
    every_wat_bad_diagnostic_is_byte_stable_shard_14 = 14;
    every_wat_bad_diagnostic_is_byte_stable_shard_15 = 15;
}
