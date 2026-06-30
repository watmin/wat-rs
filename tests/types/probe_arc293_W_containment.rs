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
#[ignore = "arc 293.W — RED until the containment rule rejects a record declaring a struct field"]
fn a_record_cannot_declare_a_struct_field() {
    match startup_beside(file!()) {
        Ok(_) => panic!(
            "a record declaring a struct field (:w::Bad's `c <- :w::Conn`) must be REJECTED at declaration \
             (the containment rule); it loaded cleanly — the wire-wall breach (a struct can then cross comms)."
        ),
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("Conn") || msg.contains("portable") || msg.contains("struct"),
                "expected a containment-rule rejection naming the offending non-portable field; got: {msg}"
            );
        }
    }
}
