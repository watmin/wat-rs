//! Arc 278 / 293.W — THE ACCEPTANCE GATE for opaque purity self-enrolment, with its
//! non-vacuity control.
//!
//! WHY: 293.W's containment rule makes "a pure aggregate holds only pure fields" a TYPE
//! guarantee — a record claiming to be EDN cannot contain a live resource. The rule itself was
//! always sound. Its **enrolment** was not: `is_pure_type` knew Rust opaques only through two
//! hand-written lists, so every `#[wat_dispatch]` opaque minted after those lists were written
//! read as PURE. A parametric one never even reached the `TypeExpr::Path` arm — it fell through
//! `TypeExpr::Parametric`'s `_ => args.iter().all(is_pure_type)`, which presumes the CONTAINER
//! pure and checks only its type arguments.
//!
//! The consequence, proven by run 2026-08-08 and green for a month before that: a defservice
//! could declare `:durable [cache <- :wat::cache::Lru<String,i64>]` — a live, thread-owned LRU
//! handle in the slot whose whole contract is "plain EDN that survives a wire and a hibernation"
//! — and it COMPILED. `wat/cache.wat` asserted the opposite in a comment, about that exact type.
//!
//! THE FIX (`is_registered_rust_opaque`, `src/check.rs`): every `#[wat_dispatch]` opaque already
//! self-registers its path into `RustDepsRegistry.types`. `is_pure_type` now consults that
//! registry on BOTH arms, before either fallthrough. Self-enrolling — no new hand list, and it
//! cannot drift out of step with the macro, because the macro is what populates it.
//!
//! ⚠ THIS GATE REPLACES A SCRATCH PROBE THAT PROVED THE HOLE BY LOADING GREEN
//! (`wat-scripts/scratch-pad/probe-293w-durable-admits-unenrolled-opaque.wat`, deleted in the
//! same commit). That file could not survive the fix — its whole point was to compile when it
//! should not — so the demonstration is preserved here, inverted into a standing wall.
//!
//! WHAT IS **NOT** CLOSED, and must not be read into a green here: `is_pure_type`'s
//! `TypeExpr::Path` arm still ends `None => true`, which is load-bearing for formal type
//! parameters AND for six of our own core types that are genuinely pure but unregistered
//! (`PersistentMap`, `PersistentVector`, `WatAST`, `HolonAST`, `time::Instant`, `time::Duration`
//! — measured: flipping that arm turns 2713 of 4376 tests red). Enrolling those is arc 255's
//! registry work. This gate covers REGISTERED RUST OPAQUES ONLY.

use wat::freeze::{startup_from_file, StartupError};

const BAD: &str = "tests/types/probe_arc278_opaque_purity_wall.wat.bad";
const CONTROL: &str = "tests/types/probe_arc278_opaque_purity_wall_control.wat";

/// THE WALL BITES — a registered Rust opaque in a defservice's `:durable` is refused AT LOAD.
///
/// This exercises two things at once, and the second is the non-obvious one: (a) the containment
/// rule refuses an impure field in a pure aggregate, and (b) a defservice's `:durable` slot IS
/// such an aggregate — it synthesizes `<svc>::Record` — so the wall reaches it. (b) is what the
/// connection-scoped-world stone's STOP-3 turned on.
#[test]
fn registered_rust_opaque_in_durable_is_refused_at_load() {
    let err = startup_from_file(BAD).expect_err(
        "a `#[wat_dispatch]` opaque (:wat::cache::Lru) declared in a defservice's `:durable` \
         must be refused by the 293.W containment rule — a live thread-owned handle cannot be \
         reconstructed from EDN on the far side of a wire",
    );
    let StartupError::Type(te) = &err else {
        panic!("expected StartupError::Type(ImpureFieldInPureAggregate), got {err:?}");
    };
    let rendered = format!("{te:?}");
    assert!(
        // rune:lint(loose-assert) — the rendering embeds a machine-specific span (absolute
        // source path + live line number), so a golden cannot pin it; a targeted PRESENCE check
        // for the error kind's own EDN tag is the precise claim available here.
        rendered.contains("ImpureFieldInPureAggregate"),
        "expected the containment-rule violation, got: {rendered}"
    );
    // Name the SUBJECT, not just the kind: a wall that fires on the wrong field would pass the
    // check above while proving nothing about the opaque.
    assert!(
        // rune:lint(loose-assert) — same reason as above.
        rendered.contains("Lru"),
        "the diagnostic must name the offending opaque field type, got: {rendered}"
    );
}

/// THE NON-VACUITY CONTROL — swap that one field for a plain `i64` and the very same file loads.
///
/// Without this, the RED above would prove "something in that fixture is bad", not "exactly the
/// opaque field is what's refused" (R59 `NISI FRANGAS, NIHIL PROBAS`). It also guards the
/// opposite failure: a change that made `is_pure_type` deny too much would turn this red, and
/// that is precisely the 2713-test cascade the fix was scoped to avoid.
#[test]
fn control_with_a_pure_durable_field_loads() {
    startup_from_file(CONTROL).unwrap_or_else(|e| {
        panic!(
            "the control MUST load — if it does not, the gate's RED no longer isolates the \
             opaque field and BOTH tests are lying. Fix this first. Got: {e:?}"
        )
    });
}
