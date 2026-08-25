//! FLOOR GATE for `wat-scripts/fuzz/rete-differential.wat` — the rete differential fuzzer.
//!
//! The fuzzer enumerates a bounded shape-space of rules and queries, fires each through BOTH
//! `fire-rules` (native) and `fire-rules$oracle` (the wat reference), and compares row counts read
//! from the rule's own LHS. Nothing hardcodes an expected value: the oracle supplies them, which
//! is what lets the space grow without hand-authoring cases.
//!
//! ## Why this is cheap enough to sit on every floor
//!
//! The oracle is superlinear — measured ~O(n²): 31 facts 11.5 ms, 136 facts 81 ms, 556 facts
//! 0.96 s, 2236 facts 15.3 s. The whole grid at high sizes takes hours for exactly this reason.
//! So the fuzzer trades fact VOLUME for shape DIVERSITY: a handful of facts per case, 288 cases,
//! ~2 s total. A join or negation defect shows at 3 facts exactly as it does at 3000.
//!
//! ## Three assertions, and the second one is the one that matters
//!
//! 1. the process exits 0,
//! 2. `cases=` is NON-ZERO — a fuzzer that silently enumerates nothing reports `mismatches=0`
//!    and looks identical to a passing one. This is the vacuity gate,
//! 3. `mismatches=0`.
//!
//! MUTATION-PROVEN: clearing `leading_emitted` per round in `fire/delta.rs` (reintroducing the
//! 2026-08-24 leading-filter defect) turns this red with 36 of 288 mismatching, each naming its
//! coordinate. The failure SET localizes the defect on its own — every one has `prefix=0`, and
//! `wpos=first` never fails, because a leading `where` makes the filter no longer parentless.

use std::path::Path;
use std::process::Command;

fn field(stdout: &str, key: &str) -> Option<i64> {
    let at = stdout.find(key)?;
    let rest = &stdout[at + key.len()..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse::<i64>().ok()
}

#[test]
fn rete_fuzzer_finds_no_native_oracle_divergence() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = Path::new("wat-scripts/fuzz/rete-differential.wat");
    assert!(
        path.exists(),
        "{} is missing — the gate would pass vacuously",
        path.display()
    );

    let out = Command::new(bin)
        .arg(path)
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    assert!(
        out.status.success(),
        "the fuzzer did not exit 0.\n  stdout: {stdout}\n  stderr: {stderr}"
    );

    let cases = field(&stdout, "cases=").unwrap_or_else(|| {
        panic!("no `cases=` in fuzzer output — shape changed?\n  stdout: {stdout}")
    });
    assert!(
        cases > 0,
        "the fuzzer ran ZERO cases and would report `mismatches=0` regardless — this gate is \
         measuring nothing. stdout: {stdout}"
    );

    let bad = field(&stdout, "mismatches=").unwrap_or_else(|| {
        panic!("no `mismatches=` in fuzzer output — shape changed?\n  stdout: {stdout}")
    });

    // ── A RATCHET, NOT A ZERO ─────────────────────────────────────────────────────────────
    // The fuzzer currently finds 22 real divergences of 504 shapes, all in the accumulate
    // family, decomposing into exactly the two defects documented and reproduced in
    // `probe_arc278_accumulate_divergences.{rs,wat}`:
    //
    //   family A  prefix=0, wpos=0, dups>=2   native = depth+1, oracle = 1   (rows == rounds)
    //   family B  prefix>=1, wpos=2, dups>=2  native = 0,       oracle = 1   (second `where`)
    //
    // Asserting 0 here would make the floor red and block every unrelated commit; deleting the
    // accumulate shape from the fuzzer would hide a live bug to keep a gate green, which is the
    // exact trade this codebase refuses. So the gate pins the number instead.
    //
    // ANY movement is a red test that demands an explanation, in both directions: fewer means a
    // fix landed (lower this, and un-`#[ignore]` the matching probe), more means a NEW divergence
    // has appeared and the MISMATCH lines name its coordinate. A pinned count is only honest
    // while every case behind it is documented — if you cannot map a mismatch to a family above,
    // it is new, and it does not belong under this number.
    const KNOWN_OPEN: i64 = 22;
    assert_eq!(
        bad, KNOWN_OPEN,
        "the fuzzer's divergence count MOVED: {bad} of {cases} shapes, expected {KNOWN_OPEN}.\n\
         If FEWER: a fix landed — lower KNOWN_OPEN and un-ignore the probe it fixed.\n\
         If MORE: a new divergence exists; each MISMATCH line names its coordinate, which is a \
         permanent case name rather than a seed — dial it back in to reproduce.\n{stdout}"
    );
}
