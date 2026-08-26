//! Probe — runtime-error path crosses the primed wire as a structured cause
//! (arc 170 slice 1i; arc 278 IPC de-prime).
//!
//! Path exercised: a forked child whose body errors at RUNTIME (not a Rust
//! panic). Integer division by zero — `(:wat::i64::/ 1 0)` — passes the
//! type-checker but fails at child runtime, flowing through `apply_function` as
//! `Err(RuntimeError)` (the Ok(Err(runtime)) arm of the forked child).
//!
//! IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
//! (fork + OS-pipe scrape → `:wat::kernel::RunResult`) onto the PRIMED peer wire —
//! the fixture now `spawn-program' :process` + `recv'` and returns the crash
//! cause's message as a plain `:wat::core::String`.
//!
//! Mapping: a runtime error in the child surfaces over the wire as `recv'` →
//! `Lost[cause]` with `cause = LociDiedError::RuntimeError` (NOT Panic — a runtime
//! error is not a Rust panic; same mapping wat_run_sandboxed's missing-main case
//! grounds). The retired capture model's contract ("Failure.message carries the
//! actual runtime error text, NOT 'forked program exited N'") is preserved: the
//! returned String is that runtime-error text.
//!
//! Row G (path-honesty): the child exercises ONLY the runtime-error exit path.
//! No AssertionPayload, no plain panic.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Call a zero-arg compute fn in the co-located fixture and return its
/// `:wat::core::String` result (the crash cause's message).
fn run_fn(fn_name: &str) -> String {
    match call_beside_value(file!(), fn_name).expect("compute should run") {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

/// The `RuntimeError.message` crosses the wire as `to_wire_edn(re)` — a wat EDN
/// form `#wat.runtime/DivisionByZero {:message "…" :location #wat.core/Span{…} …}`
/// whose `:location` embeds the ABSOLUTE host path to the fixture (host-varying).
/// Per the substrate doctrine (every wat stdio value is EDN — assert the
/// STRUCTURE, never a loose `contains`), we parse it and field-extract the
/// host-independent `:message` datum for an EXACT assertion.
fn diagnostic_message(edn: &str) -> String {
    let parsed = wat_edn::parse_owned(edn.trim())
        .unwrap_or_else(|e| panic!("RuntimeError.message must be a wat EDN form; got {edn:?}: {e}"));
    let map = match &parsed {
        // The RuntimeError diagnostic is a #wat.runtime/<Variant> {…} tagged map.
        wat_edn::OwnedValue::Tagged(_, body) => match body.as_ref() {
            wat_edn::OwnedValue::Map(entries) => entries,
            other => panic!("expected a tagged Map diagnostic; got {other:?}"),
        },
        other => panic!("expected a tagged RuntimeError diagnostic; got {other:?}"),
    };
    for (k, v) in map {
        if let wat_edn::OwnedValue::Keyword(kw) = k {
            if kw.namespace().is_none() && kw.name() == "message" {
                if let wat_edn::OwnedValue::String(s) = v {
                    return s.to_string();
                }
            }
        }
    }
    panic!("RuntimeError diagnostic carried no :message String field; got {edn:?}");
}

#[test]
fn probe_runtime_error_produces_structured_edn() {
    // The child divides by zero → RuntimeError::DivisionByZero; the parent's
    // recv' sees Lost[RuntimeError]; the fixture returns RuntimeError.message.
    let msg = run_fn(":probe::runtime-err");

    eprintln!("===== probe_runtime_error_produces_structured_edn =====");
    eprintln!("RuntimeError.message: {:?}", msg);
    eprintln!("=======================================================");

    // A Message/Closed/Panic/WRONG:<variant> sentinel is NOT a wat EDN diagnostic
    // form, so field-extraction would panic on it — the structural parse itself is
    // the guard that the crash crossed the wire as a genuine Lost[RuntimeError].
    // The extracted :message is the host-independent datum (the full diagnostic
    // embeds the absolute host path in :location); it must be EXACTLY the
    // division-by-zero text — which is neither the WRONG:<variant> sentinels nor
    // the retired plain-text fallback "forked program exited N".
    let diag = diagnostic_message(&msg);
    assert_eq!(
        diag, "division by zero",
        "a runtime error must surface over the primed wire as LociDiedError::RuntimeError \
         carrying the structured division-by-zero diagnostic; got: {msg:?}"
    );
}
