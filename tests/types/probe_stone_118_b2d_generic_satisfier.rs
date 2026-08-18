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
//! ## ⚠ THIS FILE IS A WITNESS, AND IT INVERTS WHEN THE FIX LANDS
//!
//! `defect_*` asserts the BROKEN behaviour, so it is GREEN on the broken substrate. **When B2d
//! lands, `defect_surface_method_return_loses_the_instantiation` must go RED** — that RED is the
//! stone's acceptance, and the fix's job is to move that fixture from `.wat.bad` to a passing
//! `.wat`. Kept rather than described in prose
//! (`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`), and deliberately NOT
//! `#[ignore]`d (`[[feedback_a_house_convention_can_be_the_mechanism_that_built_the_pile]]`).
//!
//! ## ★ The positive half is load-bearing
//!
//! `_neg` failing proves nothing alone — *"`Seqable<T>` is broken"* explains it just as well, and
//! would aim the fix at the wrong door. The two `control_*` rows bound it: a concrete container
//! still satisfies a concrete surface instantiation (B1a intact), AND the `Stream<T>` result is
//! perfectly usable by an equally polymorphic consumer. That second row is *why nothing caught this*
//! — `core-seqable.wat` only ever fed `Seqable/seq` into `into`, whose Stream clause is itself
//! `Stream<T>`. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const NEG: &str = "tests/types/probe_stone_118_b2d_generic_satisfier_neg.wat.bad";
const POS: &str = "tests/types/probe_stone_118_b2d_generic_satisfier_pos.wat";

#[test]
fn defect_surface_method_return_loses_the_instantiation() {
    let err = startup_from_file(NEG).expect_err(
        "Seqable/seq on a Vector<i64> yields Stream<T> (T free), which cannot satisfy a concrete \
         Seqable<i64> parameter — if this now PASSES, door 2 is fixed and this witness must be \
         retired into the positive fixture",
    );
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };

    // The arm, named exactly: a CONCRETE surface instantiation was expected, and what arrived was
    // the method's return with its type param still free. Asserting `got` is what distinguishes
    // "the instantiation was lost" from any other satisfaction failure.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == ":wat::core::Seqable<wat::core::i64>"
            && got == ":wat::stream::Stream<T>");
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
