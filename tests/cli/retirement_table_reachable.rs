//! Arc 255 STONE-retirement-table-becomes-mechanism — end-to-end reachability gate.
//!
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-retirement-table-becomes-the-mechanism.md`
//! (§ "DO THIS FIRST"), `docs/arc/2026/06/255-builtin-registry/NOTE-the-retirement-table-is-inert-for-half-its-rows.md`.
//!
//! `RETIREMENT_TABLE` (`src/remedy/retirement.rs`) looked like a lookup the substrate
//! performs. It was not: only thirteen hand-written check-time arms consulted it; the
//! other twenty-two rows fell through to a silent-accept and died at RUNTIME as a bare
//! `unknown function: <name>` with no help. This gate is the wall that makes the next
//! inert row impossible rather than merely unlikely.
//!
//! Three load-bearing properties (brief's own words):
//!
//! 1. **Walks `RETIREMENT_TABLE` itself.** `wat::retirement_table_names_for_gate()` is a
//!    thin `#[doc(hidden)]` bridge (through the `pub(crate)` `remedy` module, which an
//!    external integration-test crate cannot otherwise reach) straight onto the table —
//!    not a hand-list of names copied into this file. A hand-list here would be exactly
//!    the defect this stone fixes, one level up
//!    (`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).
//! 2. **Drives the real binary, end to end.** `Command::new(env!("CARGO_BIN_EXE_wat"))`,
//!    the `wat_grep.rs` / `wat_cli.rs` pattern. An in-process `check_program` call would
//!    report the inert rows GREEN — they pass the checker silently and fail only at
//!    runtime, so only a real process boundary can see the bug.
//! 3. **The assertion is the NEGATIVE — no exemption list.** A row fails this gate iff
//!    the combined stdout+stderr contains a bare `unknown function: <name>` with no
//!    accompanying "is retired" remedy. A `MalformedForm` or `TypeMismatch` that DOES
//!    name a replacement passes without being special-cased — which is exactly what
//!    admits the seven `vec`/`list`/`tuple`/`Some`/`Ok`/`Err`/`:None` rows (diagnosed by
//!    a third path per the DESIGN doc's "out of scope" section) with no per-name code
//!    here for them either.

use std::io::Write;
use std::process::Command;

/// Write `contents` to a uniquely-named temp `.wat` file and return its path.
fn write_temp(contents: &str, tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "wat-retirement-gate-{}-{}-{}.wat",
        std::process::id(),
        tag,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let mut f = std::fs::File::create(&path).expect("create temp");
    f.write_all(contents.as_bytes()).expect("write");
    path
}

// rune:lint(no-inlined-wat) — the program under test is BUILT FROM `RETIREMENT_TABLE` at runtime,
// one per row, and therefore cannot be a co-located fixture: a fixture set would be a hand-written
// copy of the table's names, which is precisely the defect this stone exists to remove
// (`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`). The literal below is a two-line
// scaffold whose only variable is the row's own name; it is never a golden and never compared.
/// Drive the real `wat` binary on a program that calls `retired_name` in head position
/// with zero arguments, wrapped in `println` — the exact shape the brief's own
/// acceptance script establishes for `:wat::core::Uuid/v4`. Zero args is safe
/// uniformly: every dispatch arm that intercepts a retired name (the working
/// thirteen's hand-written arms, and this stone's new Door 1) matches on the callee
/// NAME alone and returns before arity is ever inspected.
fn probe(retired_name: &str) -> String {
    let program = format!(
        "(:wat::core::defn :user::main [] -> :wat::core::nil\n  (:wat::kernel::println ({retired_name})))\n"
    );
    let tag: String = retired_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let path = write_temp(&program, &tag);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin).arg(&path).output().expect("spawn wat");
    let _ = std::fs::remove_file(&path);
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    combined
}

/// A row is INERT iff the output names the retired form as an `unknown function` with
/// no accompanying "is retired" remedy — the exact bare shape doors 1 and 2 both
/// close. Substring, not equality: check-time output carries file:line:col prefixes
/// that vary per (uniquely-named) temp path.
fn is_inert(output: &str) -> bool {
    output.contains("unknown function: ") && !output.contains("is retired")
}

#[test]
fn retirement_table_is_fully_reachable() {
    let names = wat::retirement_table_names_for_gate();
    // Non-vacuity: the table has 35 rows at the time this stone was written: if this
    // ever reads near-zero, the bridge itself broke and every row below would pass
    // vacuously.
    assert!(
        names.len() >= 30,
        "sanity: RETIREMENT_TABLE should have ~35 rows; bridge returned {} — \
         the bridge is broken, not the table",
        names.len()
    );

    let mut red: Vec<(&str, String)> = Vec::new();
    let mut late: Vec<(&str, String)> = Vec::new();
    for name in &names {
        let output = probe(name);
        if is_inert(&output) {
            red.push((name, output));
        } else if !output.contains("wat.check/") {
            // ⛔ DIAGNOSED, BUT TOO LATE. The name is retired and something says so — but the
            // something is the RUNTIME (door 2), not the checker (door 1). A statically-written
            // head must be caught before the program runs, which is where the hand-written arms
            // have always caught the bare names.
            //
            // This arm exists because the gate WITHOUT it was measured non-discriminating:
            // door 1 was neutered (`if false && …`), the gate stayed GREEN on all 33 rows,
            // because door 2's message alone satisfies "names a replacement". A gate that
            // survives the removal of the door it exists to guard is a claim, not a wall.
            // NISI FRANGAS, NIHIL PROBAS.
            late.push((name, output));
        }
    }

    assert!(
        late.is_empty(),
        "{} row(s) are diagnosed only at RUNTIME — door 1 (check.rs's retirement consult) is not \
         firing for them, so the error arrives after the program starts instead of before:\n{}",
        late.len(),
        late.iter()
            .map(|(n, o)| format!("\n  {}\n    -> {}\n", n, o.trim().replace('\n', "\n       ")))
            .collect::<String>()
    );

    if !red.is_empty() {
        let mut msg = format!(
            "{} of {} retirement-table rows are INERT (bare `unknown function:` with no replacement named):\n",
            red.len(),
            names.len()
        );
        for (name, output) in &red {
            msg.push_str(&format!("\n  {}\n    -> {}\n", name, output.trim().replace('\n', "\n       ")));
        }
        panic!("{}", msg);
    }
}
