//! FLOOR GATE for `wat-scripts/lib/gen.wat` — the finite-generator library proves its own laws.
//!
//! Sibling of `hunt_tooling_selftests.rs`: tooling that other gates depend on must itself be
//! gated, or a silent break in it turns every downstream gate green-and-meaningless.
//!
//! `wat-scripts/fuzz/gen-selftest.wat` drives five laws THROUGH `gen-check`, over spaces built by
//! `gen-coords` — the checker checks the checker. The load-bearing one is the BIJECTION law:
//! `at` must map 0..card onto the coordinate space injectively and totally. If it does not,
//! enumeration silently visits some tuples twice and misses others, and a fuzzer reporting
//! "288 cases, 0 mismatches" would be lying in a way no other gate here could detect.
//!
//! This exists because of a real hole: when `gen.wat` was first committed, FOUR of its six verbs
//! (`gen-ints`, `gen-fmap`, `gen-digit`, `gen-shift`) had zero call sites anywhere in the repo,
//! and the library had no test of its own. A library that tests things, untested.
//!
//! MUTATION-PROVEN twice, both against `gen.wat` itself:
//!   * `gen-shift` made a no-op (the mixed-radix carry never advances) → 118 violations
//!   * `gen-digit` using `base + 1` (digits leak past their base)      → 186 violations

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
fn gen_library_satisfies_its_own_laws() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = Path::new("wat-scripts/fuzz/gen-selftest.wat");
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
        "gen-selftest did not exit 0.\n  stdout: {stdout}\n  stderr: {stderr}"
    );

    let checked = field(&stdout, "checked=")
        .unwrap_or_else(|| panic!("no `checked=` in output — shape changed?\n{stdout}"));
    assert!(
        checked > 0,
        "the law suite checked ZERO points and would report `violations=0` regardless — the \
         vacuity gate. stdout: {stdout}"
    );

    let bad = field(&stdout, "violations=")
        .unwrap_or_else(|| panic!("no `violations=` in output — shape changed?\n{stdout}"));
    assert_eq!(
        bad, 0,
        "the generator library VIOLATES its own laws at {bad} of {checked} checked points. If the \
         bijection law (L4) is among them, every enumerating fuzzer built on this library is \
         reporting a case count it does not actually cover.\n{stdout}"
    );
}
