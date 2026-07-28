//! Arc 214 Stone 8.2 — the stdio services carry no channel handles on their message surface.
//!
//! THE PROPERTY (arc 214's, unchanged): a service's message surface is SCALARS. Channels belong
//! to the universe, not to what crosses between peers — so no `Receiver<`/`Sender<`-typed field
//! or param may appear in a stdio service's wat source, and the `Add`/`Remove` handle-passing
//! registration protocol must not exist (registration is the universe's job).
//!
//! ── RE-SPECIMENED 2026-07-28 (arc 170 #24), AND WHY ─────────────────────────────────────────
//! This gate used to read `wat/kernel/services/stdin.wat` and assert those properties of the
//! hand-rolled `StdInService`. That service was DELETED in arc 170 Phase 3 — stdin became the
//! `:wat::kernel::stdin-svc` defservice — and the file it left behind held only the `readln`
//! macro and a cap constant. So this gate was reading a file with no service in it: zero
//! `Receiver<`, zero `Sender<`, zero `:Add`, and therefore GREEN ON NOTHING, indefinitely.
//!
//! It was found because arc 170 #24 renamed that file to `wat/kernel/readln.wat` (naming it for
//! what it actually holds), which broke the path and forced someone to look. Re-pointing it at
//! `readln.wat` would have preserved the vacuum in a new location — the macro has no message
//! surface either. So it is re-pointed at the LIVE subject instead: `stdio.wat`, which holds all
//! three primed stdio defservices (`stdin-svc`, `stdout-svc`, `stderr-svc`). The property now
//! covers the whole trio, which is strictly stronger than the stone asked for and true to it.
//!
//! This is the same disposition `wat_cli__check_bad.wat` got when it stopped being a bad program
//! (re-specimen, don't delete) — and the same class as the 11 gates `91bbb8cd` found asserting
//! nothing, R55's swallowing verifier, and R59's gate structurally incapable of noticing. A gate
//! whose subject is deleted out from under it does not fail. It passes, forever, silently.
//!
//! Each test now also pins the declared service SET exactly before asserting anything about it,
//! so the next time one moves the gate goes red and NAMES it, instead of quietly measuring an
//! empty room.
//!
//! Run: `cargo test --release --test services probe_arc214_stone82_stdio_services_no_handle_passing`

use std::fs;

const STDIO_SERVICES: &str = "wat/kernel/services/stdio.wat";

/// Every `:wat::kernel::*` service this file declares, in declaration order.
///
/// Pinned as a SET rather than probed with `contains`, so the gate names exactly which service
/// moved instead of only noticing that one did — and so it cannot drift into passing on a file
/// that still mentions a service in a comment while no longer declaring it.
fn declared_services(src: &str) -> Vec<&str> {
    src.lines()
        .filter_map(|l| {
            l.split(";;")
                .next()
                .unwrap_or("")
                .trim()
                .strip_prefix('(')
                .and_then(|r| r.strip_prefix(":wat::service::defservice "))
                .map(str::trim)
        })
        .collect()
}

const EXPECTED_SERVICES: [&str; 3] = [
    ":wat::kernel::stdout-svc",
    ":wat::kernel::stderr-svc",
    ":wat::kernel::stdin-svc",
];

/// True when `code` names `variant` as a WHOLE token, not as a substring.
///
/// The original matcher was a bare `.contains(":Add")`, which `:wat::kernel::Address<…>`
/// satisfies. That false positive could never fire while this gate was reading a file with no
/// service in it — two defects, the second hidden by the first. An empty room never trips a
/// loose pattern, which is the quiet cost of a vacuous gate: it also hides its own bugs.
fn names_variant(code: &str, variant: &str) -> bool {
    code.match_indices(variant).any(|(i, _)| {
        code[i + variant.len()..]
            .chars()
            .next()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '-' || c == '_'))
    })
}

/// The matcher must DISCRIMINATE, proven here rather than assumed — because this gate's whole
/// history is a check that passed without being able to fail. `NISI FRANGAS, NIHIL PROBAS`.
///
/// Done as a unit test on the predicate rather than by corrupting `stdio.wat`, because that file
/// is baked into the binary at build time: breaking it deliberately would break the freeze, not
/// the gate.
#[test]
fn the_matcher_discriminates_a_variant_from_a_prefix() {
    // The exact token that produced the false positive.
    assert!(
        !names_variant(":wat::kernel::Address<wat::kernel::StdOut::Op>", ":Add"),
        "`:Address` must NOT read as the `:Add` variant — this is the false positive that made \
         the re-specimened gate fail on a property that actually holds"
    );
    // …and the shapes a real Add variant would take must still be caught.
    assert!(names_variant(":Add [tid <- :wat::core::i64]", ":Add"));
    assert!(names_variant(":svc::Event::Add tid) nil)", ":Add"));
    assert!(names_variant(":Remove)", ":Remove"));
}

/// The stdio services' message types must carry NO channel handles — no `Receiver<` / `Sender<`
/// typed fields anywhere in their wat source (channels belong to the universe, not the message
/// surface).
#[test]
fn probe_1_stdio_service_messages_carry_no_handles() {
    let src = fs::read_to_string(STDIO_SERVICES)
        .unwrap_or_else(|e| panic!("{STDIO_SERVICES} must exist and be readable: {e}"));

    assert_eq!(
        declared_services(&src),
        EXPECTED_SERVICES,
        "{STDIO_SERVICES} no longer declares the expected stdio services — this gate's subject \
         has moved again. Re-point it at the live services rather than letting it pass on a file \
         that no longer holds them."
    );

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
        "A stdio service must be a pure tagged-message loop — no channel-typed fields/params in \
         its wat source (handles are the universe's, not the message surface's). Offending \
         lines:\n{}",
        offenders
            .iter()
            .map(|(n, l)| format!("  stdio.wat:{n} → {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The Add/Remove registration protocol must be GONE — registration is the universe's job (the
/// Rust-side wiring), not a wat message exchange.
#[test]
fn probe_2_stdio_services_have_no_add_remove_protocol() {
    let src = fs::read_to_string(STDIO_SERVICES)
        .unwrap_or_else(|e| panic!("{STDIO_SERVICES} must exist and be readable: {e}"));

    assert_eq!(
        declared_services(&src),
        EXPECTED_SERVICES,
        "{STDIO_SERVICES} no longer declares the expected stdio services — re-point this gate."
    );

    let has_add = src.lines().any(|l| {
        let code = l.split(";;").next().unwrap_or("");
        names_variant(code, ":Add")
            || names_variant(code, ":Remove")
            || code.contains("handle-add")
            || code.contains("handle-remove")
    });
    assert!(
        !has_add,
        "A stdio service must not implement a registration protocol — the Add/Remove dance (and \
         its routing table) is the handle-passing architecture this stone kills."
    );
}
