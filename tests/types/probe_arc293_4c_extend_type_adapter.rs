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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

/// A foreign `:wat::core::String`, taught `:t::Tagged` via `extend-type`, satisfies the surface and
/// dispatches `:t::Tagged/tag` to the adapter's impl (constant 42).
#[test]
fn extend_type_teaches_a_foreign_type_to_satisfy_a_surface() {
    let world = startup_beside(file!())
        .expect("293.4c: extend-type must teach :wat::core::String to satisfy :t::Tagged");

    let got = eval_in_frozen(
        &wat::parse_one!("(:t::probe)").expect("parse"),
        &world,
        &Environment::new(),
    )
    .expect("(:t::probe) must dispatch :t::Tagged/tag on a String to the extend-type impl")
    .value_owned();

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
        "tests/types/probe_arc293_4c_extend_type_adapter_dup_bad.wat",
    );
    assert!(
        result.is_err(),
        "two extend-type impls for the same :<T>/<method> must be rejected as DuplicateDefine \
         at startup; but startup succeeded — collision detection is broken"
    );
}

/// NEGATIVE arm (EXPECTATIONS row #4): a foreign type NOT extended (no `:<T>/tag` registered)
/// passed where `:t::Tagged` is required must be rejected at check time.
/// Proves that surface satisfaction is a real check, not always-true.
#[test]
fn non_extended_foreign_type_is_rejected_at_check_time() {
    let result = startup_from_file(
        "tests/types/probe_arc293_4c_extend_type_adapter_notextended_bad.wat",
    );
    assert!(
        result.is_err(),
        "a foreign type without `extend-type` for the surface must NOT satisfy it; \
         the check must reject the call, but startup succeeded — satisfaction is broken (STOP-3)"
    );
}
