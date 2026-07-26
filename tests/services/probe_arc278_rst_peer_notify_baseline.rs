//! DESIGN-STONE-rst-peer-notify.md — the RST stone (arc 278 tail), RED gate.
//!
//! A PROCESS-locus service whose handler GENUINELY panics (`assertion-failed!`, a real runtime
//! panic mid-handler — not a decode rejection). `c` is a SEPARATE `connect'`-ed peer (matching
//! `probe_arc272_rs2_crash_surfaces_to_client` / `probe_arc278_process_crash_reason_carried`'s
//! shape exactly) whose `boom` call's `recv'` observes whatever the dying service leaves behind.
//!
//! ORIGINALLY a RED-baseline probe (locked the pre-fix behavior: a bare clean-EOF
//! `RecvError::Disconnected` collapse, "recv failed: peer closed / channel disconnected",
//! carrying neither the crash sentinel nor any distinct signal — see git history for that
//! text). Flipped to the GREEN gate once the mechanism (Option A — `serve-dispatch-op'`,
//! `RecvError::PeerCrashed`) landed: the same scenario must now surface a DISTINCT signal —
//! `RecvError::PeerCrashed` — NOT the old bare-disconnect text, and carrying NO reason (the
//! reason is administrative, owner-channel-only per arc 294's ruling).
//!
//! Arc 278 recv'-wall: the generated client method `/boom` returns a matchable `RecvOutcome<Reply>`
//! VALUE, never a raise (a raise unwinds past the reader — the mask the wall kills). A genuine
//! far-side handler panic must surface a DISTINCT `RecvOutcome::Lost` — a reason-free 500 (the crash
//! reason is administrative, owner-channel-only per arc 294's ruling) — NOT a bare clean-EOF
//! `RecvOutcome::Closed` (the old mute disconnect) and NOT a fake `RecvOutcome::Message`. The fixture
//! MATCHES the outcome and RETURNS a marker; we assert `is_ok` (it matched a value, not a raise) +
//! that the client saw `LOST:` (peer crashed), distinct from a bare `CLOSED` disconnect, and that the
//! returned reason carries NO crash sentinel (it is reason-free).
//!
//! Run: cargo test --release -p wat --test services rst_peer_notify_baseline

use wat::freeze::call_beside_value;

#[test]
fn client_sees_peer_crashed_not_bare_disconnect() {
    let result = call_beside_value(file!(), ":user::compute");
    let text = format!("{result:?}");
    assert!(
        result.is_ok(),
        "a genuine handler panic must surface to the client as a matchable RecvOutcome::Lost VALUE \
         (never a raise, which would unwind past the reader); got Err: {text}"
    );
    assert!(
        // rune:lint(loose-assert) — distinguishing the value-based RecvOutcome marker ("LOST" vs
        // "CLOSED"/"MESSAGE"); the value is wrapped as Ok(String(..)) so an exact scalar eq does not apply.
        text.contains("LOST") && !text.contains("CLOSED") && !text.contains("MESSAGE"),
        "GREEN gate: the client must see a DISTINCT RecvOutcome::Lost (peer crashed), not a bare \
         clean-EOF ::Closed disconnect and not a fake ::Message; got: {text}"
    );
    // The client's ::Lost is REASON-FREE by construction — the generated client method scrubs the
    // cause to a reason-free 500 (the real crash reason is administrative, owner's-crash-channel-only
    // per arc 294), so the crash sentinel never reaches the client.
    assert!(
        // rune:lint(loose-assert) — absence check: the crash sentinel must NOT leak to the client (the
        // reason is administrative, owner's-crash-channel-only); the client's ::Lost is reason-free.
        !text.contains("RST-BASELINE-SENTINEL-7731"),
        "the client's ::Lost must be REASON-FREE — the crash reason is administrative (owner's crash \
         channel only), never leaked to the client; got: {text}"
    );
}
