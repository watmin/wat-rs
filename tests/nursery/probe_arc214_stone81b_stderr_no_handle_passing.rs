//! Arc 214 Stone 8.1b — StdErrService reborn (FM-2-bis disconfirming gate).
//!
//! The 15-line haircut's second application (DESIGN-SLICE-8 § stones: "stderr
//! follows as 8.1b — same shape, fd 2"). Today the StdErrService `Add` event
//! carries `data-rx <- Receiver<...>` and `ack-tx <- Sender<...>` as FIELDS —
//! the same handle-passing registration protocol Stone 8.1 killed for stdout.
//!
//! This gate reads the service's wat source (sibling of
//! `probe_arc214_stone81_stdout_no_handle_passing`): RED while any Event
//! variant carries a channel-typed field; GREEN when the service's message
//! surface is scalars only (the rebirth). The behavioral regression gates
//! (eprintln round-trip, the ProcessPanics envelope path — which writes fd 2
//! DIRECTLY and never traverses the service — the corpus) ride the BRIEF;
//! this probe pins the STRUCTURAL kill.
//!
//! Run: `cargo test --release --test nursery probe_arc214_stone81b_stderr_no_handle_passing`

use std::fs;

/// The StdErrService's message types must carry NO channel handles —
/// no `Receiver<` / `Sender<` typed fields anywhere in the service's wat
/// source (channels belong to the universe, not the message surface).
#[test]
fn probe_1_stderr_service_messages_carry_no_handles() {
    let src = fs::read_to_string("wat/kernel/services/stderr.wat")
        .expect("wat/kernel/services/stderr.wat must exist");
    let offenders: Vec<(usize, &str)> = src
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            let code = l.split(";;").next().unwrap_or("");
            code.contains("Receiver<") || code.contains("Sender<")
        })
        .map(|(i, l)| (i + 1, l.trim()))
        .collect();
    assert!(
        offenders.is_empty(),
        "StdErrService must be a pure tagged-event loop — no channel-typed \
         fields/params in its wat source (handles are the universe's, not the \
         message surface's). Offending lines:\n{}",
        offenders
            .iter()
            .map(|(n, l)| format!("  stderr.wat:{} → {}", n, l))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The Add/Remove registration protocol must be GONE — registration is the
/// universe's job (Rust-side reply registry), not a wat message exchange.
#[test]
fn probe_2_stderr_service_has_no_add_remove_protocol() {
    let src = fs::read_to_string("wat/kernel/services/stderr.wat")
        .expect("wat/kernel/services/stderr.wat must exist");
    let has_add = src.lines().any(|l| {
        let code = l.split(";;").next().unwrap_or("");
        code.contains(":Add") || code.contains("handle-add") || code.contains("handle-remove")
    });
    assert!(
        !has_add,
        "StdErrService must not implement a registration protocol — the \
         Add/Remove dance (and its routing table) is the handle-passing \
         architecture this stone kills"
    );
}
