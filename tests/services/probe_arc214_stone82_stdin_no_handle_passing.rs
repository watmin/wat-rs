//! Arc 214 Stone 8.2 — StdInService reborn (FM-2-bis disconfirming gate).
//!
//! The trio's third member (DESIGN-SLICE-8 § stones: "8.2 — StdInService
//! reborn: the reply-routing proof"). Today the StdInService `Add` event
//! carries `data-rx <- Receiver<...>` and `reply-tx <- Sender<...>` as
//! FIELDS — the same handle-passing registration protocol Stones 8.1/8.1b
//! killed for the write pair.
//!
//! This gate reads the service's wat source (sibling of the stone81/81b
//! probes): RED while any Event variant carries a channel-typed field;
//! GREEN when the service's message surface is scalars only. The
//! reply-routing proof (concurrent readers each get their own line) and the
//! EOF-doctrine preservation (lock-step violation panics the service —
//! stdin.wat's deliberate assertion-failed!) ride the BRIEF; this probe
//! pins the STRUCTURAL kill.
//!
//! Run: `cargo test --release --test services probe_arc214_stone82_stdin_no_handle_passing`

use std::fs;

/// The StdInService's message types must carry NO channel handles —
/// no `Receiver<` / `Sender<` typed fields anywhere in the service's wat
/// source (channels belong to the universe, not the message surface).
#[test]
fn probe_1_stdin_service_messages_carry_no_handles() {
    let src = fs::read_to_string("wat/kernel/services/stdin.wat")
        .expect("wat/kernel/services/stdin.wat must exist");
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
        "StdInService must be a pure tagged-event loop — no channel-typed \
         fields/params in its wat source (handles are the universe's, not the \
         message surface's). Offending lines:\n{}",
        offenders
            .iter()
            .map(|(n, l)| format!("  stdin.wat:{} → {}", n, l))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The Add/Remove registration protocol must be GONE — registration is the
/// universe's job (Rust-side reply registry), not a wat message exchange.
#[test]
fn probe_2_stdin_service_has_no_add_remove_protocol() {
    let src = fs::read_to_string("wat/kernel/services/stdin.wat")
        .expect("wat/kernel/services/stdin.wat must exist");
    let has_add = src.lines().any(|l| {
        let code = l.split(";;").next().unwrap_or("");
        code.contains(":Add") || code.contains("handle-add") || code.contains("handle-remove")
    });
    assert!(
        !has_add,
        "StdInService must not implement a registration protocol — the \
         Add/Remove dance (and its routing table) is the handle-passing \
         architecture this stone kills"
    );
}
