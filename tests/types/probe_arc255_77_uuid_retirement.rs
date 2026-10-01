//! Arc 255.77 — cutover stone 3, "the UUID type goes home". The retired type-key spelling
//! `:wat::core::Uuid` must be HARD-CUT rejected with a structured retirement remedy naming
//! `wat.uuid/UUID` (U1, builder, 2026-10-01) — mirrors the `:wat::core::Char` precedent
//! (`tests/value/probe_arc242_stone1_lexeme_role.rs`'s `contract_03`).
//!
//! The end-to-end reachability gate (`tests/cli/retirement_table_reachable.rs`) separately
//! proves this row is reachable by walking `RETIREMENT_TABLE` itself and driving the real
//! binary; this test is the narrower, direct assertion on the message shape.
use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

#[test]
fn legacy_uuid_type_key_hard_cut_with_retirement_remedy() {
    let result = startup_from_file("tests/types/probe_arc255_77_uuid_retirement.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::Uuid"
            && reason == "':wat::core::Uuid' is retired (arc 255.77); use 'wat.uuid/UUID' \
                instead (wat.type/ holds only the 24 hard primitives; other typed things, like \
                Uuid, live in their own homes)"
    );
}
