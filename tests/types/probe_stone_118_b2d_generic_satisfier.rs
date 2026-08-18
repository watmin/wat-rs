//! Stone 118.B2d — DISCONFIRMING PROBE for **door 2**: a GENERIC satisfier cannot bind the
//! surface's type param, so a surface METHOD'S RETURN loses the receiver's instantiation.
//!
//! `Seqable/seq` on a `Vector<i64>` must yield `Stream<i64>`. It yields `Stream<T>`, `T` free.
//!
//! Found while migrating the six walkers (118.B2b, `d4c6f3a5`); PRE-EXISTING since B1 (`488eacd0`).
//! Design: `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.B2d-a-generic-satisfier-cannot-bind-the-surface-param.md`
//!
//! ## The mechanism this pins
//!
//! `src/check.rs:4926-4948`, path (1), resolves a parametric-surface member's type by looking up the
//! satisfier's registered `<ConcreteType>/<method>` scheme — on the assumption, stated in its own
//! comment, that `extend-type` already substituted the surface's `<T>` to a CONCRETE binding
//! (*"e.g. `T=i64` for `(extend-type :IntBox :Holds<i64> …)`"*). `Seqable<T>` is satisfied by
//! GENERIC containers, so `(extend-type :wat::core::Vector :wat::core::Seqable<T>)` binds `T -> T`,
//! a VARIABLE. The stored scheme's return stays `Stream<T>`; nothing instantiates it from the
//! receiver. Path (2) holds exactly the needed machinery but is guarded to fire only when the
//! receiver IS the surface.
//!
//! ## ✅ THIS FILE WAS A WITNESS AND IT INVERTED, EXACTLY AS ITS HEADER PROMISED
//!
//! It was committed asserting the BROKEN behaviour — `_neg.wat.bad`, green on the broken substrate —
//! with the header stating *"when B2d lands it must go RED, and the fix's job is to move that
//! fixture from `.wat.bad` to a passing `.wat`."* That is precisely what happened.
//!
//! ## THE FIX
//!
//! Path (1) now binds the surface's params from the RECEIVER's args when the arities line up (the
//! same guard path (2) already applies). **No new state, and `rename` is the signal**: a satisfier
//! that bound CONCRETELY leaves no surface param in its scheme, so the rename is the identity and
//! those schemes are byte-identical. The safety is structural rather than a guard to maintain.
//!
//! ## ★ The positive half is load-bearing
//!
//! `_neg` failing proves nothing alone — *"`Seqable<T>` is broken"* explains it just as well, and
//! would aim the fix at the wrong door. The two `control_*` rows bound it: a concrete container
//! still satisfies a concrete surface instantiation (B1a intact), AND the `Stream<T>` result is
//! perfectly usable by an equally polymorphic consumer. That second row is *why nothing caught this*
//! — `core-seqable.wat` only ever fed `Seqable/seq` into `into`, whose Stream clause is itself
//! `Stream<T>`. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

use wat::freeze::startup_from_file;

const WAS_NEG: &str = "tests/types/probe_stone_118_b2d_generic_satisfier.wat";
const POS: &str = "tests/types/probe_stone_118_b2d_generic_satisfier_pos.wat";

/// ★ THE STONE. `Seqable/seq` on a `Vector<i64>` now yields `Stream<i64>`, so its result can be
/// handed to a consumer wanting a concrete element type.
///
/// This fixture was `_neg.wat.bad` and asserted the DEFECT. It inverted when B2d landed; that
/// inversion is the acceptance signal, and the file moved to a plain `.wat` that must check clean.
#[test]
fn surface_method_return_carries_the_receivers_instantiation() {
    startup_from_file(WAS_NEG).unwrap_or_else(|e| {
        panic!(
            "Seqable/seq on a Vector<i64> must yield Stream<i64>. A failure here means the surface \
             method's return has lost the receiver's instantiation again — the ONE method \
             Seqable<T> has cannot then have its result typed. Got: {e:?}"
        )
    });
}

#[test]
fn control_direct_container_and_polymorphic_consumer_both_check_clean() {
    startup_from_file(POS).unwrap_or_else(|e| {
        panic!(
            "CONTROL BROKEN — if this fails, door 2 is NOT 'the method's return loses the \
             instantiation' and the stone is mis-aimed. Row 1 is B1a (a concrete container \
             satisfies a concrete surface instantiation); row 2 is the polymorphic consumer that \
             hid this defect for a month. Got: {e:?}"
        )
    });
}
