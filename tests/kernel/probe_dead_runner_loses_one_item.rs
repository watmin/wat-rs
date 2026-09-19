//! a-dead-runner-loses-one-item-not-the-run — control by mutation.
//!
//! Runtime: 4 items, 2 thread runners; worker 0 panics on its first item.
//! Re-dispatch must still return `[1 2 3 4]` (survivor is x+1, not double).
//! Source pin: Closed/Lost in collect-loop call `collect-requeue`. Delete that
//! and this file goes red — the predecessor control failed that bar.

use std::path::Path;

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

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

fn run_vec(path: &str) -> Vec<i64> {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute eval")
    {
        Value::Vec(v) => v
            .iter()
            .map(|tv| match tv {
                Value::i64(n) => *n,
                other => panic!("non-i64 element: {other:?}"),
            })
            .collect(),
        other => panic!("expected Vector; got {other:?}"),
    }
}

/// The collect-loop arm for `variant`, not a constructor that produces it.
/// A later helper may construct Closed/Lost/Malformed; nth(1) would land there.
fn collect_loop_arm_body<'a>(code: &'a str, variant: &str) -> &'a str {
    let mut search = code;
    loop {
        let Some(idx) = search.find(variant) else {
            panic!("collect-loop has no {variant} arm");
        };
        let after = &search[idx + variant.len()..];
        let body = match after.find(":wat::spawn::ServiceEvent::") {
            Some(end) => &after[..end],
            None => after,
        };
        if body.contains("collect-requeue") {
            return body;
        }
        search = after;
    }
}

/// ⭐ Closed/Lost re-dispatch is actually in the collect-loop arms.
/// Delete `collect-requeue` from those arms and this reddens.
#[test]
fn closed_and_lost_requeue_the_held_item() {
    let code = bracket_code();
    let closed_body = collect_loop_arm_body(&code, ":wat::spawn::ServiceEvent::Closed");
    assert!(
        // rune:lint(loose-assert) — presence of the re-dispatch helper in the Closed
        // arm body. The claim is "this call is here", not a value equality.
        closed_body.contains("collect-requeue"),
        "Closed arm no longer re-dispatches via collect-requeue — the mutation this \
         control exists to catch"
    );
    let lost_body = collect_loop_arm_body(&code, ":wat::spawn::ServiceEvent::Lost");
    assert!(
        // rune:lint(loose-assert) — presence of the re-dispatch helper in the Lost
        // arm body. The claim is "this call is here", not a value equality.
        lost_body.contains("collect-requeue"),
        "Lost arm no longer re-dispatches via collect-requeue — the mutation this \
         control exists to catch"
    );
}

/// All 4 results returned after worker 0 dies holding item 0.
#[test]
fn killed_runner_item_is_redelivered() {
    let got = run_vec("tests/kernel/probe_dead_runner_loses_one_item.wat");
    assert_eq!(got, vec![1, 2, 3, 4]);
}

/// Non-vacuity: worker 0 cannot produce x+1. If the kill never fired, item 0
/// would be 0 (worker 0's double) or the run would hang. `[1 2 3 4]` means the
/// survivor did every item, including the one the dead runner held.
#[test]
fn dead_runner_fault_fired() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = std::process::Command::new(bin)
        .arg("tests/kernel/probe_dead_runner_loses_one_item.wat")
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "map should survive the dead runner; stdout={stdout} stderr={stderr}"
    );
    // println of a String writes the EDN form, quotes included.
    assert_eq!(stdout.trim(), "\"1,2,3,4\"");
}

/// ⭐ THE SEAM CONTROL — `Malformed` must RE-DISPATCH, not panic.
///
/// ⛔ WHY THIS EXISTS. `one-selectable-set-primitive` unified the classification door, so
/// `:wat::kernel::select` now builds `ServiceEvent::Malformed` on a decode failure where it
/// previously built `Lost`. That reclassification moved a garbled frame OUT of the Closed/Lost
/// re-dispatch arms and INTO a panic — for one commit, a fault that this very file had just made
/// survivable killed the whole run again. No test caught it; the two stones were graded in the
/// wrong order. This is the tripwire for that seam.
///
/// ⚠ `Malformed` is NOT the same act as Closed/Lost: the peer is still there (it finished and
/// sent; only the frame was unreadable), so it must be marked idle and KEPT in `alive`. Removing
/// it would discard a healthy worker.
#[test]
fn malformed_requeues_and_keeps_the_runner_alive() {
    let code = bracket_code();
    let body = collect_loop_arm_body(&code, ":wat::spawn::ServiceEvent::Malformed");
    assert!(
        body.len() > 80 && body.len() < 1200,
        "Malformed arm sliced to {} bytes — the terminator moved and the window is wrong; \
         re-derive it before trusting the assertions below",
        body.len()
    );

    assert!(
        // rune:lint(loose-assert) — presence of the re-dispatch helper in the Malformed arm
        // body. The claim is "this call is here", not a value equality.
        body.contains("collect-requeue"),
        "⛔ Malformed no longer re-dispatches. Since the classification unification, `select` \
         builds Malformed on a decode failure — a garbled frame that panics here kills the whole \
         bracket run, which is the regression this control exists to catch."
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE over one arm body: the arm must not raise.
        !body.contains("assertion-failed!"),
        "⛔ Malformed raises again. A decode failure means the item's result did not arrive — the \
         same fact as Closed/Lost — and the work is re-runnable. The wall-clock bound is the stop \
         for a deterministic encode fault, not a panic on the first one."
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE: the peer is alive, so it must not be
        // dropped from the select set the way Closed/Lost drop a dead one.
        !body.contains("alive-without"),
        "⛔ Malformed drops the runner from `alive`. The peer is ALIVE — it finished and sent, \
         only the frame was unreadable. Dropping it discards a healthy worker."
    );
    assert!(
        // rune:lint(loose-assert) — presence: the runner is marked idle so collect-feed-idle can
        // hand it the re-queued item, exactly as the Message arm does after a delivery.
        body.contains("holding-set"),
        "Malformed must mark the runner idle (`holding-set … -1`) or collect-feed-idle will never \
         hand it the item it just re-queued"
    );
}

/// ⭐ FrameTooLarge is Rejected, not Lost. The runner is wedged in write_all —
/// re-dispatching to it deadlocks. Drop it from `alive` (Closed/Lost shape),
/// re-queue the item for a survivor. Keep it in `alive` and the 2000 ms bound hangs.
#[test]
fn rejected_drops_the_wedged_runner() {
    let code = bracket_code();
    let body = collect_loop_arm_body(&code, ":wat::spawn::ServiceEvent::Rejected");
    assert!(
        body.len() > 80 && body.len() < 1600,
        "Rejected arm sliced to {} bytes — the terminator moved and the window is wrong; \
         re-derive it before trusting the assertions below",
        body.len()
    );
    assert!(
        // rune:lint(loose-assert) — item still known; re-queue for a survivor.
        body.contains("collect-requeue"),
        "Rejected must re-queue the held item via collect-requeue"
    );
    assert!(
        // rune:lint(loose-assert) — the runner is not readable; drop it.
        body.contains("alive-without"),
        "⛔ Rejected keeps the wedged runner in `alive`. Handing it work deadlocks \
         (send waits, runner blocked in write_all)."
    );
}
