//! every-worker-races-its-own-timer — route (c): bounded select, N deadlines,
//! one wakeup. Control by mutation: N stalling runners GaveUp; remove
//! `select-by-deadline` and the source pin reddens (a hang is a fail).

use std::path::Path;
use std::process::Command;

fn bracket_code() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("wat/bracket.wat");
    let src = std::fs::read_to_string(p).expect("wat/bracket.wat must be readable");
    src.lines()
        .map(|l| match l.find(";;") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn wat_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wat"))
}

/// ⭐ Mutation: collect-loop waits with `select-by-deadline`, not unbounded
/// `select`. Delete that verb and this reddens without hanging the floor.
#[test]
fn collect_loop_waits_with_select_by_deadline() {
    let code = bracket_code();
    assert!(
        // rune:lint(loose-assert) — presence of the bounded wait.
        code.contains(":wat::kernel::select-by-deadline"),
        "collect-loop no longer calls select-by-deadline"
    );
    assert!(
        // rune:lint(loose-assert) — already/RST guard is the late-reply answer.
        code.contains("collect-already?"),
        "Message arm lost the already/RST duplicate guard"
    );
    assert!(
        // rune:lint(loose-assert) — expiry uses collect-requeue, guards intact.
        code.contains("collect-expire") && code.contains("SelectDeadline::TimedOut"),
        "TimedOut path no longer expire-scans via collect-expire"
    );
}

/// Primitive: two silent threads, 200ms deadline → TimedOut, at least 200ms.
#[test]
fn select_by_deadline_times_out_on_silent_peers() {
    let output = wat_cmd()
        .arg("tests/kernel/probe_select_by_deadline_silent.wat")
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let blob = format!("{stdout}{stderr}");
    eprintln!("silent-select status={:?} blob={blob}", output.status.code());
    assert_eq!(
        output.status.code(),
        Some(0),
        "select-by-deadline on silent peers must return; stdout={stdout} stderr={stderr}"
    );
    assert!(
        // rune:lint(loose-assert) — TimedOut is the claim.
        stdout.contains("arm=TimedOut"),
        "select-by-deadline must return TimedOut; got {blob}"
    );
    let elapsed = stdout
        .split("elapsed-ms=")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(-1);
    assert!(
        elapsed >= 200,
        "silent-peer timeout must be at least 200ms; got {blob}"
    );
}

/// N runners all stalling: GaveUp ~200ms, last=TimedOut. Remove the bound
/// (swap select-by-deadline for select) and this hangs — a timeout is a fail.
#[test]
fn n_stall_gives_up_not_hang() {
    let output = wat_cmd()
        .env("WAT_COLLECT_DEADLINE_MS", "200")
        .arg("tests/kernel/probe_every_worker_races_n_stall.wat")
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let blob = format!("{stdout}{stderr}");
    eprintln!("n-stall status={:?} blob={blob}", output.status.code());
    assert_eq!(
        output.status.code(),
        Some(2),
        "N stalling runners must GaveUp; stdout={stdout} stderr={stderr}"
    );
    assert!(
        // rune:lint(loose-assert) — GaveUp is the claim.
        blob.contains("GaveUp"),
        "collect-gave-up! must have fired; got {blob}"
    );

    // ⭐ THE ASSERTION THAT MAKES THIS A CONTROL. Exit-2 + "GaveUp" hold WITH AND WITHOUT the
    // bounded wait, so without this the test passes either way — measured: swapping
    // `select-by-deadline` back to unbounded `select` still exits 2 and still prints GaveUp,
    // because the workers are SLOW (nap 2000 ms), not stalled, so their replies eventually arrive
    // and the LOOP-HEAD `elapsed >= budget` check fires GaveUp instead of the wait arm.
    //
    //     bounded   → waited-ms=200
    //     unbounded → waited-ms=2000        (10×, and previously unasserted)
    //
    // So the number is the control, not the word. Lower bound is non-vacuity (the bound really
    // was reached); upper bound is what discriminates. Generous margin: this host runs 5337 tests
    // in parallel, and the discriminating gap is an order of magnitude wide.
    let waited: i64 = blob
        .split("waited-ms=")
        .nth(1)
        .and_then(|t| t.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|d| d.parse().ok())
        .unwrap_or_else(|| panic!("no waited-ms in the GaveUp report; got {blob}"));
    assert!(
        (200..1000).contains(&waited),
        "GaveUp must fire from the BOUNDED WAIT (~200 ms), not from the loop head after the slow \
         replies land (~2000 ms). waited-ms={waited}. If this reads ~2000, the wait is unbounded \
         again and `collect-loop` is relying on the loop-head check — which cannot save a runner \
         that never replies at all."
    );
    assert!(
        // rune:lint(loose-assert) — the injected bound is a substring of the interpolate.
        blob.contains("wall-clock bound 200 ms"),
        "the injected bound must be the one named; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — last=TimedOut names the select-by-deadline arm.
        blob.contains("last=TimedOut"),
        "N-stall fires via select-by-deadline TimedOut; got {blob}"
    );
    let waited = blob
        .split("waited-ms=")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit() && c != '-').next())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(-1);
    assert!(
        waited >= 200,
        "N-stall bound must be at least 200ms; got {blob}"
    );
}
