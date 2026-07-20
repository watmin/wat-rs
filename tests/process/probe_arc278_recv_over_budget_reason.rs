//! Arc 278 #15 (mute-kill floor) — the SPEAK facet at the wat `recv'` client surface, PROCESS locus.
//!
//! The parent spawns a child via `spawn-program'` with a small per-message frame budget
//! (`process/max-message-bytes 256`); the child `println`s ONE over-budget string; the parent `recv'`s
//! it. The parent's output receiver rejects the over-budget frame with `RecvError::FrameTooLarge`.
//!
//! THE LAW (wat never hides a failure): the `recv'` MUST raise carrying the frame-cap reason
//! ("frame exceeded cap … budget"), NOT the reasonless mute
//! "recv failed: peer closed / channel disconnected".
//!
//! STATUS (grounded): this path is ALREADY reasoned at HEAD — a spawn-program' PROCESS peer's
//! `recv'` routes through `ProcessPeerBundle::recv()` → `classify_peer_error` (spawn.rs:338:
//! `FrameTooLarge => Lost(reason)`) → `PeerRecvError::Crashed(reason)` → the runtime's
//! PROCESS_PEER_TYPE_PATH arm maps `Crashed(reason) => reason` (runtime.rs:~26078). So this test
//! PASSES at HEAD; it is a REGRESSION GUARD locking in the already-dead over-budget mute on the
//! process-peer path, NOT a RED gate. (The genuinely-reachable remaining `FrameTooLarge` mute is the
//! SOCKET-tier `recv_wire` arm at runtime.rs:~26177, reached via socket peers — a distinct path this
//! probe does NOT exercise.)
//!
//! Modeled on tests/services/probe_arc278_sift_rules_arena.rs's harness (call the wat entry fn and
//! assert on the raised `RuntimeError`).
//!
//! Run SERIALLY (spawns a process):
//!   `cargo test --release -p wat --test process recv_over_budget_reason -- --test-threads=1`

use wat::freeze::call_beside;

/// A `recv'` of an over-budget frame from a spawned child MUST raise with the frame-cap reason.
#[test]
fn process_recv_over_budget_frame_surfaces_cap_reason_not_a_mute() {
    match call_beside(file!(), ":user::over-budget-recv") {
        Ok(v) => panic!(
            "expected :user::over-budget-recv to RAISE — the child's over-budget frame must be \
             rejected by the parent's budgeted receiver; got Ok({v:?})"
        ),
        Err(e) => {
            let text = format!("{e:?}").to_lowercase();
            // SPEAK: the raise must NAME the frame-cap reason — one of these cap words.
            let carries_reason = text.contains("frame")
                || text.contains("exceed")
                || text.contains("too large")
                || text.contains("cap")
                || text.contains("budget");
            // MUTE-KILL: it must NOT read as the reasonless peer-closed collapse.
            let is_bare_mute = text.contains("peer closed / channel disconnected")
                || text.contains("channel disconnected");
            assert!(
                carries_reason && !is_bare_mute,
                "THE LAW (wat never hides a failure) — the SPEAK floor: an over-budget frame at the \
                 wat `recv'` client surface must RAISE carrying the frame-cap reason (frame / exceed / \
                 too large / cap / budget), not collapse to the reasonless mute \
                 \"recv failed: peer closed / channel disconnected\". got: {e:?}"
            );
        }
    }
}
