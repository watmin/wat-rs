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
//! `RuntimeError`'s `Display`/`Debug` both render as EDN (`to_wire_edn` — arc 296 stone B: no
//! `{:?}`-impostor Rust-struct face survives). Asserted structurally against a co-located golden
//! (`assert_edn_eq!`, mirroring `probe_arc215_collection_literal_inference.rs` probe 5 /
//! `probe_arc278_journal_service_logs.rs`) rather than a Debug-string `contains`/`==`: the golden
//! SUBSUMES all three properties in one structure-exact comparison — the `:reason` field IS the
//! distinct PeerCrashed text (not the old clean-EOF collapse), and it carries no crash sentinel.
//!
//! Run: cargo test --release -p wat --test services rst_peer_notify_baseline

use wat::freeze::call_beside;

#[test]
fn client_sees_peer_crashed_not_bare_disconnect() {
    let result = call_beside(file!(), ":user::compute");
    let err = result.expect_err("a genuine handler panic must still raise to the client");
    wat::assert_edn_eq!(
        format!("{err}"),
        include_str!("probe_arc278_rst_peer_notify_baseline__peer_crashed.edn"),
        "GREEN gate: the client's recv' must surface the distinct RecvError::PeerCrashed \
         MalformedForm — no crash reason, distinct from the old clean-EOF collapse text"
    );
}
