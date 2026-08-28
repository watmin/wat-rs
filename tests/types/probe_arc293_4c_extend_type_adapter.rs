//! Arc 293.4c — `extend-type` as the foreign-accessor adapter (the disconfirming probe).
//!
//! THE GAP this isolates: `extend-type` must teach a FOREIGN built-in (one you don't own — here
//! `:wat::core::String`) to satisfy a surface, by registering the surface's accessor as a `:<T>/<method>`
//! callable. Then a String value, passed where the surface is required, both SATISFIES it (check) and
//! DISPATCHES through it (runtime, via 293.4b).
//!
//! RED at HEAD (post-293.4b) — three gaps: (1) `extend-type` on a surface target registers no
//! `:<T>/<method>`; (2) satisfaction is Aggregate-only; (3) the dispatcher reads only Record/Struct/RustOpaque
//! receivers. So `(:t::tag-of "hello")` fails to type-check.
//!
//! GREEN at 293.4c — `extend-type :T :Surface` registers `:<T>/<method>` impls (collision = DuplicateDefine);
//! satisfaction resolves method members for any type whose `:<T>/<method>` exists; the dispatcher derives the
//! concrete FQDN from `receiver.type_name()`.

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::{RuntimeErrorKind, Value};

/// A foreign `:wat::core::String`, taught `:t::Tagged` via `extend-type`, satisfies the surface and
/// dispatches `:t::Tagged/tag` to the adapter's impl (constant 42).
#[test]
fn extend_type_teaches_a_foreign_type_to_satisfy_a_surface() {
    let got = call_beside_value(file!(), ":t::probe")
        .expect("(:t::probe) must dispatch :t::Tagged/tag on a String to the extend-type impl");

    match got {
        Value::i64(n) => assert_eq!(
            n, 42,
            "the monkeypatched :t::Tagged/tag on a String should return the adapter's constant 42; got {n}"
        ),
        other => panic!("expected i64 42 from the adapter dispatch; got {other:?}"),
    }
}

/// COLLISION arm (EXPECTATIONS row #3): two `extend-type` for the same `:<T>/<method>` must
/// fail at startup with DuplicateDefine. Proves that collisions are structurally enforced.
#[test]
fn extend_type_surface_collision_is_duplicate_define() {
    let result = startup_from_file(
        "tests/types/probe_arc293_4c_extend_type_adapter_dup.wat.bad",
    );
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(e.kind(), RuntimeErrorKind::DuplicateDefine(name)
            if name == ":wat::core::String/tag")
    );
}

/// NEGATIVE arm (EXPECTATIONS row #4): a foreign type NOT extended (no `:<T>/tag` registered)
/// passed where `:t::Tagged` is required must be rejected at check time.
/// Proves that surface satisfaction is a real check, not always-true.
#[test]
fn non_extended_foreign_type_is_rejected_at_check_time() {
    let result = startup_from_file(
        "tests/types/probe_arc293_4c_extend_type_adapter_notextended.wat.bad",
    );
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":t::tag-neg"
            && param == "#1"
            && expected == ":t::TaggedNeg"
            && got == ":wat::core::i64"
    );
}
