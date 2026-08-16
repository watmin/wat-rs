//! RED probe — arc 293.W (the deep wire wall): THE CONTAINMENT RULE.
//!
//! A portable aggregate (record / holon) may declare ONLY portable field types. A `Struct` field is
//! ILLEGAL at declaration: a struct cannot be reconstructed from EDN bytes on the far side (no default
//! for a bound resource), so a record holding one could never cross — it must not exist. This makes
//! §7's "a struct crosses NO comms" a TYPE guarantee instead of a runtime hope: a record cannot HOLD a
//! struct, so it can never CARRY one across (the wire-wall breach becomes unrepresentable).
//!
//! GROUNDED BREACH this guards: at HEAD a record-with-struct-field loads, serializes, and a struct
//! crosses a process peer (`#w/S {:a 99}` reconstructed far-side; §7 / R3 SUB SUPERFICIE QUOD ES violated).
//!
//! RED at HEAD: the illegal declaration loads cleanly. GREEN after 293.W: the load is REJECTED, the error
//! naming the offending non-portable field.
//!
//! STRIKE-READY: committed `#[ignore]`'d (RED) so the floor stays 0; un-ignore when 293.W lands.

use wat::freeze::startup_beside;

#[test]
fn a_record_cannot_declare_a_struct_field() {
    match startup_beside(file!()) {
        Ok(_) => panic!(
            "a record declaring a struct field (:w::Bad's `c <- :w::Conn`) must be REJECTED at declaration \
             (the containment rule); it loaded cleanly — the wire-wall breach (a struct can then cross comms)."
        ),
        Err(e) => {
            let msg = format!("{e:?}");
            // CLASS-C RULING (296 Wave B1, builder overrule 2026-08-15): this golden pins an
            // INTERNAL `src/check.rs` `rust_caller_span!()` — the Rust source line:col of the
            // `TypeError::new` call site that raised `ImpureFieldInPureAggregate`, not a user
            // `.wat` span. The orchestrator proposed normalizing/dropping it because any edit
            // above that line in check.rs re-churns the pinned line. The builder overruled:
            // (1) the churn cost is trivial — exactly one other `.edn` golden in the tree pins a
            // `src/*.rs` span; (2) a pinned line that gets updated when it moves is in a constant
            // state of correctness, while a DROPPED field is permanently blind; (3) the span
            // DISCRIMINATES THE EMITTER — `ImpureFieldInPureAggregate` can be raised from more
            // than one call site in check.rs, and `rust_caller_span!()` says which. Drop it and
            // this test goes green the moment a *different* code path starts raising the same
            // error kind — that silent pass is exactly the coverage this pin buys. KEEP PINNING
            // THE SPAN. Do not re-propose dropping it.
            wat::assert_edn_matches_file!(msg, "probe_arc293_W_containment__a_record_cannot_declare_a_struct_field.edn", "record declaring a struct field: ImpureFieldInPureAggregate (internal check.rs span pinned — see comment above)");
        }
    }
}
