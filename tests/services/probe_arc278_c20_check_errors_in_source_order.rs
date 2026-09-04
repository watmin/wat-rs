//! Arc 278, C20 — ★ CHECK ERRORS MAY NOT BE ORDERED BY A HASH SEED.
//!
//! ## What was broken
//!
//! `check_program` (`src/check.rs`) collects errors in two passes that walk
//! `SymbolTable::functions_iter()` — a `HashMap<String, Arc<Function>>`. Rust reseeds
//! `RandomState` per process, so the per-function error BLOCKS emerged in a different sequence
//! on every run. The set of findings never changed; their order did. Two `wat` runs over one
//! broken file printed the same errors in a different sequence, so every diff of two runs was
//! noise that looked like signal — the D11 corpus scan had already paid for this class once
//! (see `tests/lint/diagnostic_output_is_deterministic.rs`).
//!
//! Measured at `75e82f882`, 24 runs of the release binary per file, `:?N`-normalised:
//!
//! ```text
//!   probe_arc170_c2_mixed_macro_swap.wat.bad        14 / 10   (two outputs)
//!   probe_arc170_w2a_kwargs_check_mint_swap.wat.bad 14 / 10   (two outputs)
//! ```
//!
//! The cure is `check::error::sort_into_source_order` at `check_program`'s exit. The `HashMap`
//! is NOT changed: it is a hot symbol-lookup path, and C10's ruling forbids paying `O(log n)`
//! there to serve a diagnostic.
//!
//! ## Why SOURCE order and not merely a stable one
//!
//! De-randomising into hash-stable order would have satisfied the determinism gate and served
//! no reader. Note what the fix actually produced for `w2a`: BOTH pre-fix outputs were wrong
//! for a reader (`40, 51, 51, 48` and `51, 51, 48, 40`), and the post-fix order `40, 48, 51, 51`
//! is NEITHER of them. The cure is not one of the two coin faces — it is a third, better answer.
//!
//! ## Why 24 runs, and why 2 would be worse than nothing
//!
//! Each file's variance was a per-process Bernoulli draw at roughly even odds (14/10 measured
//! above, twice). An N-run test that pins one output is a FALSE GREEN on a surviving bug exactly
//! when all N draws land on the pinned side: `p^N`, which at p≈0.58 is 0.34 for N=2 and 3.3e-5
//! for N=24. A "run it twice" regression test would go green over a live defect about a third of
//! the time, and the next hand would read that green as proof. 24 is also the count C19's own
//! corpus sweep needed to close its set at three after a 2-run scan reported two.
//!
//! A FRESH PROCESS PER RUN IS THE POINT: the variance is a per-process hash seed, so 24 checks
//! inside one process would share one seed and this file would be green by construction.
//!
//! ## Cost, measured — the budget in `.config/nextest.toml` is derived from these
//!
//! 24 sequential runs, 6 samples each, this box, release:
//!
//! ```text
//!   c2  ... 9.64 9.73 9.69 9.74 9.66 9.68  s   (median 9.69)
//!   w2a ... 8.00 7.94 7.91 7.87 7.92 7.96  s   (median 7.93)
//! ```
//!
//! At this repo's own recorded floor-contention band (3.5x-4.4x) the worse of the two projects
//! to 42.9s, which does not fit the default 30s kill — hence the named override.

use wat::check::error::{sort_into_source_order, CheckError, CheckErrorKind};
use wat::span::Span;

/// See the header: derived from `p^N`, not picked.
const RUNS: usize = 24;

const C2: &str = "tests/services/probe_arc170_c2_mixed_macro_swap.wat.bad";
const W2A: &str = "tests/services/probe_arc170_w2a_kwargs_check_mint_swap.wat.bad";

/// `C2`'s findings in source order: seven `:wat::core::match` scrutinee mismatches inside seven
/// separate service `defn`s, then the two `kwargs-check` parameter mismatches in `:user::main`.
///
/// ⚠ THE LAST TWO ROWS ARE THE POINT. They are a genuine SAME-SPAN PAIR — identical file, line,
/// col, end, and variant — so a sort keyed on `(line, col)` orders them by INPUT order, i.e. by
/// the hash seed. They are what makes this fixture, not `W2A`, the one that proves the tie-break
/// is reachable in the real corpus rather than only in a constructed pair.
const C2_SOURCE_ORDER: &[(i64, i64, i64, i64)] = &[
    (91, 29, 91, 79),
    (97, 29, 97, 79),
    (103, 29, 103, 79),
    (109, 29, 109, 79),
    (115, 29, 115, 79),
    (121, 29, 121, 79),
    (127, 29, 127, 79),
    (156, 5, 158, 53),
    (156, 5, 158, 53),
];

/// `W2A`'s findings in source order. ⚠ This sequence appeared in NEITHER pre-fix output: the
/// `:user::main` return-type mismatch at 48:3 sat first or third before, never second. Pinning it
/// is what separates "source order" from "one of the two orders the hash used to produce".
const W2A_SOURCE_ORDER: &[(i64, i64, i64, i64)] = &[
    (40, 22, 40, 84),
    (48, 3, 51, 53),
    (51, 41, 51, 44),
    (51, 49, 51, 51),
];

/// The `:param` values of `C2`'s same-span pair, in the order the TIE-BREAK must produce.
///
/// `CheckError::source_order_key`'s final component is the derived `Debug` of `CheckErrorKind`,
/// and `TypeMismatch`'s fields are declared `callee, param, expected, got` — the callees are
/// identical here, so `param` is what actually decides, and `"#1"` sorts before `"#2"`. Pinning
/// this is what proves the last key component RAN on this file rather than merely existing.
const C2_TIED_PAIR_PARAMS: (&str, &str) = ("#1", "#2");

/// One fresh process. Returns the whole thing — exit code, stdout and stderr — because a
/// diagnostic can vary on any of the three and comparing only stdout would let a varying stderr
/// through.
fn run_once(path: &str) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let out = std::process::Command::new(bin)
        .arg(path)
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {path}: {e}"));
    (out.status.code(), out.stdout, out.stderr)
}

/// Pull every `:location #wat.core/Span {…}` out of a diagnostic, IN ORDER, as
/// `(line, col, end_line, end_col)`.
///
/// Extraction, not `contains`: it PANICS with the whole output if the shape moves, so it cannot
/// decay into a check that passes over nothing. Same idiom as
/// `tests/lint/diagnostic_output_is_deterministic.rs`'s `:got` reader.
fn span_sequence(text: &str) -> Vec<(i64, i64, i64, i64)> {
    const LOC: &str = ":location #wat.core/Span {:file ";
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(LOC) {
        rest = &rest[at + LOC.len()..];
        let num = |hay: &str, key: &str| -> (i64, usize) {
            let start = hay.find(key).unwrap_or_else(|| {
                panic!("no `{key}` after a `{LOC}`; the whole output was:\n{text}")
            }) + key.len();
            let tail = &hay[start..];
            let end = tail
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or_else(|| panic!("unterminated `{key}`; the whole output was:\n{text}"));
            (
                tail[..end].parse().unwrap_or_else(|e| {
                    panic!("`{key}` is not a number ({e}); the whole output was:\n{text}")
                }),
                start + end,
            )
        };
        let (line, a) = num(rest, ":line ");
        let (col, b) = num(&rest[a..], ":col ");
        // The `:end` block repeats `:line`/`:col` inside a `#wat.core/Pos`.
        let (end_line, c) = num(&rest[a + b..], ":line ");
        let (end_col, _) = num(&rest[a + b + c..], ":col ");
        out.push((line, col, end_line, end_col));
    }
    out
}

/// The shared body: N fresh processes, one output, and that output in SOURCE order.
///
/// Two assertions, because neither can see the other's mutation:
///
/// 1. **All `RUNS` outputs byte-identical.** Deleting the sort fails HERE.
/// 2. **The span sequence is the pinned source order.** Sorting into any other fixed order —
///    reversed, or the hash-stable order a `BTreeMap` would have given — is perfectly
///    deterministic and passes assertion 1 while serving a reader nothing. Only pinning the
///    sequence proves the order is MEANINGFUL and not merely repeatable.
fn assert_stable_and_in_source_order(path: &str, expected: &[(i64, i64, i64, i64)]) -> String {
    let mut runs: Vec<(Option<i32>, Vec<u8>, Vec<u8>)> = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        runs.push(run_once(path));
    }

    // NON-VACUITY: the refusal must actually have happened. A `wat` that started exiting 0 on
    // this file would be byte-stable and span-free, and both assertions below need this one to
    // notice rather than asserting determinism over a program that stopped diagnosing anything.
    assert_ne!(
        runs[0].0,
        Some(0),
        "{path} must be REFUSED at check; it exited 0, so this test would be asserting \
         determinism over a program that no longer produces the diagnostic at all"
    );

    let distinct: std::collections::BTreeSet<_> = runs.iter().collect();
    assert_eq!(
        distinct.len(),
        1,
        "{path} produced {} DISTINCT outputs over {RUNS} runs of the same binary. Check errors \
         are being ordered by a per-process hash seed again (arc 278 C20): `check_program` \
         collects per-function errors from `SymbolTable::functions_iter()`, and \
         `check::error::sort_into_source_order` at its exit is what makes that order a property \
         of the SOURCE rather than of the run. Every distinct output follows IN FULL; do NOT \
         summarise them when reporting:\n\n{}",
        distinct.len(),
        distinct
            .iter()
            .map(|(code, out, err)| format!(
                "    exit: {code:?}\n    --- stdout ---\n{}\n    --- stderr ---\n{}",
                String::from_utf8_lossy(out),
                String::from_utf8_lossy(err),
            ))
            .collect::<Vec<_>>()
            .join("\n\n")
    );

    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&runs[0].1),
        String::from_utf8_lossy(&runs[0].2)
    );
    assert_eq!(
        span_sequence(&text),
        expected.to_vec(),
        "{path}'s findings are stable but NOT in source order. Stability alone does not make \
         this right — a fixed hash order, or a reversed sort, is equally repeatable and equally \
         useless to someone reading their own file top to bottom. Whole diagnostic:\n{text}"
    );
    text
}

#[test]
fn c2_mixed_macro_swap_check_errors_are_in_source_order_every_run() {
    let text = assert_stable_and_in_source_order(C2, C2_SOURCE_ORDER);

    // ★ THE TIE-BREAK, PROVEN ON THE REAL CORPUS. The last two findings share a span exactly
    // (156:5..158:53), so the `(line, col, end)` part of the key cannot separate them and the
    // stable sort would fall back to input — hash — order. Their `:param` fields are what the
    // final key component orders. If this pair ever stops being same-span the pin above fails
    // first and says so; this assertion is about WHICH of the two comes out in front.
    let params: Vec<&str> = text
        .match_indices(":param \"")
        .map(|(i, k)| {
            let tail = &text[i + k.len()..];
            let end = tail
                .find('"')
                .unwrap_or_else(|| panic!("unterminated `:param` field; whole output:\n{text}"));
            &tail[..end]
        })
        .collect();
    let tied = &params[params.len() - 2..];
    assert_eq!(
        (tied[0], tied[1]),
        C2_TIED_PAIR_PARAMS,
        "the same-span pair at 156:5..158:53 came out in the wrong order. `source_order_key`'s \
         last component (the derived `Debug` of `CheckErrorKind`) is the only thing that can \
         order these two — drop it and this pair returns to hash order while every span-level \
         assertion above stays green. Whole diagnostic:\n{text}"
    );
}

#[test]
fn w2a_kwargs_check_mint_swap_check_errors_are_in_source_order_every_run() {
    assert_stable_and_in_source_order(W2A, W2A_SOURCE_ORDER);
}

/// ★ THE TIE-BREAK IS LOAD-BEARING — the direct, corpus-independent proof.
///
/// The 24-run tests above drive `sort_into_source_order` through the real checker, and the c2 one
/// pins a real same-span pair — but the two members of that pair are pushed by ONE deterministic
/// intra-function walk, so their input order does not actually vary in that fixture. That makes
/// c2 proof that the tie-break is REACHED, not proof that it CHANGES anything.
///
/// This test supplies the missing half by feeding the same production function the same same-span
/// pair in BOTH input orders. `sort_by_cached_key` is STABLE, so a key that stops at
/// `(file, line, col, end)` returns the input untouched and the two orders come out different —
/// which is exactly the defect surviving inside a sort. One output over both inputs is the
/// property, and it is the assertion.
#[test]
fn tie_break_decides_a_same_span_pair_in_either_input_order() {
    let span = || {
        Span::with_end(
            std::sync::Arc::new("probe.wat".to_string()),
            156,
            5,
            158,
            53,
        )
    };
    let mismatch = |param: &str| CheckError {
        span: span(),
        kind: CheckErrorKind::TypeMismatch {
            callee: ":probe::enrich::kwargs-check::Kwargs".to_string(),
            param: param.to_string(),
            expected: ":probe::A".to_string(),
            got: ":probe::B".to_string(),
        },
    };

    // NON-VACUITY: these must really be same-span, or this test proves nothing about ties.
    let (a, b) = (mismatch("#1"), mismatch("#2"));
    assert_eq!(
        (a.span.line, a.span.col, a.span.end.as_ref().map(|p| (p.line, p.col))),
        (b.span.line, b.span.col, b.span.end.as_ref().map(|p| (p.line, p.col))),
        "the pair this test is built on is not same-span, so it exercises no tie at all"
    );

    let render = |errs: &[CheckError]| {
        errs.iter().map(|e| format!("{e:?}")).collect::<Vec<_>>().join("\n")
    };

    let mut forward = vec![mismatch("#1"), mismatch("#2")];
    let mut backward = vec![mismatch("#2"), mismatch("#1")];

    // NON-VACUITY: the two inputs must differ, or "both orders agree" is trivially true.
    assert_ne!(
        render(&forward),
        render(&backward),
        "the two inputs are identical, so this test cannot tell a total key from a partial one"
    );

    sort_into_source_order(&mut forward);
    sort_into_source_order(&mut backward);

    assert_eq!(
        render(&forward),
        render(&backward),
        "two errors at the SAME span came out in different orders depending on the order they \
         were collected in. `sort_by_cached_key` is stable, so a key that stops at \
         (file, line, col, end) leaves such pairs in input order — which at the call site in \
         `check_program` is `HashMap` order. That is the defect wearing a sort. The final key \
         component (the derived `Debug` of `CheckErrorKind`) is what breaks this tie, and it is \
         not optional."
    );
}
