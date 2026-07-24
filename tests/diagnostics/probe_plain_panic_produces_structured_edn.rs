//! Probe — plain Rust panic path crosses the primed wire as a structured cause
//! (arc 170 slice 1i; arc 278 IPC de-prime).
//!
//! Path exercised: a forked child hits a bare Rust `panic!()` (NOT an
//! AssertionPayload). The only way to trigger that from a wat body is the
//! `:wat::holon::Bundle` capacity-exceeded path with `capacity_mode = :panic`:
//! with `dim_count = 1` the budget is `floor(sqrt(1)) = 1`, so a Bundle with 2
//! atoms exceeds capacity and calls `panic!("...: capacity exceeded ...")` — a
//! bare String payload.
//!
//! IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
//! (fork + OS-pipe scrape → `:wat::kernel::RunResult { stdout, stderr, failure }`)
//! onto the PRIMED peer wire — the fixture now `spawn-program' :process` + `recv'`
//! and returns the crash cause's message as a plain `:wat::core::String`.
//!
//! Mapping: a bare Rust panic in the child surfaces over the wire as
//! `recv'` → `Lost[cause]` with `cause = LociDiedError::Panic`; the panic's String
//! rides `Panic.message`. The retired capture model's contract ("Failure.message
//! carries the actual panic text, NOT the exit-code-only fallback 'forked program
//! exited N'") is preserved: the returned String is that panic text.
//!
//! Row G (path-honesty): the child exercises ONLY the non-AssertionPayload panic
//! exit path. No assert-eq, no raise!, no RuntimeError.

use wat::freeze::call_beside;
use wat::runtime::Value;

/// Call a zero-arg compute fn in the co-located fixture and return its
/// `:wat::core::String` result (the crash cause's message).
fn run_fn(fn_name: &str) -> String {
    match call_beside(file!(), fn_name).expect("compute should run") {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

#[test]
fn probe_plain_panic_produces_structured_edn() {
    // The child sets dim_count=1 / capacity-mode :panic (private to its own
    // process runtime) and builds a 2-atom Bundle that exceeds the budget →
    // panic!("capacity exceeded under :panic"). The parent's recv' sees
    // Lost[Panic]; the fixture returns Panic.message.
    let msg = run_fn(":probe::plain-panic");

    eprintln!("===== probe_plain_panic_produces_structured_edn =====");
    eprintln!("Panic.message: {:?}", msg);
    eprintln!("=====================================================");

    // The capacity-exceeded panic message is a FULLY DETERMINISTIC scalar — it
    // embeds NO host-specific path/pid/span (unlike a RuntimeError diagnostic).
    // So the whole `Panic.message` is byte-identical assertable, which subsumes
    // every weaker check at once: it proves the crash crossed the wire as a
    // structured `Lost[LociDiedError::Panic]` carrying the ACTUAL panic text —
    // not a `Message`/`Closed`/`WRONG:<variant>` sentinel, and not the retired
    // exit-code-only fallback "forked program exited N".
    assert_eq!(
        msg, ":wat::holon::Bundle: capacity exceeded under :panic — cost 2 > budget 1 (d=1)",
        "a bare Rust panic must surface over the primed wire as LociDiedError::Panic \
         carrying the exact capacity-exceeded panic text"
    );
}
