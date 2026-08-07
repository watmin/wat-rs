//! Arc 278 #88 — the DISCONFIRMING PROBE for the rete `defn`, with its non-vacuity control.
//!
//! WHY: an ordinary `defn` called from a `where` is rete-admissible *by accident of its current
//! body*. Nobody declared it, so there is no contract, so nothing can be broken, so nothing warns —
//! edit one op inside such a helper and the failure names the RULE, with not one frame naming the
//! helper. #88 mints `(:wat::rete::core::defn …)`: same registration and symbol binding as
//! `:wat::core::defn`, body checked tighter AT THE DEFINITION SITE.
//!
//! WHAT THIS PROBE PINS, TODAY: the form does not exist, and the gap is isolated to exactly that
//! form. The fixture's body is already law-A clean (`:wat::rete::core::i64::>` is a minted
//! `RETE_OPS` row) and its control is byte-identical minus the one head — so the RED cannot be
//! blamed on anything else in the file.
//!
//! AND IT PINS THE DIAGNOSTIC'S SHAPE, which is the part worth keeping: an unrecognised head is
//! treated as a CALL, so the signature is evaluated as ARGUMENTS and the checker complains that
//! `:wat::core::i64` is a TYPE keyword in value position. Nothing names the helper, the contract,
//! or law A. `test_the_gap_diagnostic_does_not_name_the_helper` asserts that absence, so when the
//! strike lands it is the test that FORCES the diagnostic to improve rather than merely allowing
//! it to — the acceptance criterion is written down before the code exists.
//!
//! AFTER THE STRIKE: repoint the fixture's body at a NON-rete op; it must then be refused at the
//! definition with an error naming `:probe::declared`. EXPECTATIONS-rete-defn.md rows 2 and 3.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const GAP: &str = "tests/rete/probe_arc278_rete_defn_gap.wat.bad";
const CONTROL: &str = "tests/rete/probe_arc278_rete_defn_gap_control.wat";

/// THE GAP — the ruled form is unminted, so the file does not load.
#[test]
fn rete_defn_form_does_not_exist_yet() {
    let err = startup_from_file(GAP)
        .expect_err("(:wat::rete::core::defn …) is unminted — this fixture must fail to load");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // The unrecognised head is treated as a call, so its signature lands in value position.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::i64"
            && reason.contains("TYPE keyword"));
}

/// THE NON-VACUITY CONTROL — delete only that form and the very same file loads. Without this,
/// the RED above would prove "something in the fixture is bad", not "exactly this form is missing".
#[test]
fn control_without_the_rete_defn_form_loads() {
    startup_from_file(CONTROL).unwrap_or_else(|e| {
        panic!(
            "the control must load — if it does not, the gap probe's RED no longer isolates the \
             rete-defn form and BOTH tests are lying. Fix this first. Got: {e:?}"
        )
    });
}

/// The gap, stated as the acceptance criterion: today NOTHING in the diagnostic names the helper
/// whose declaration is at fault. When #88 lands this must fail, and the strike is not done until
/// it is rewritten to assert the helper IS named.
#[test]
fn the_gap_diagnostic_does_not_name_the_helper() {
    let err = startup_from_file(GAP).expect_err("fixture must fail to load");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    let rendered = format!("{errs:?}");
    assert!(
        // rune:lint(loose-assert) — a targeted ABSENCE over a large output (the lint's own named
        // exemption). The claim is "the helper's name appears NOWHERE in the diagnostic", which an
        // exact assert_eq!/.edn golden cannot express: pinning the whole CheckErrors would go red on
        // any unrelated wording change AND would still not state the absence. The output also embeds
        // machine-specific absolute paths from startup_from_file.
        !rendered.contains(":probe::declared"),
        "the diagnostic now names the helper — #88 has landed. REWRITE this test to assert the \
         helper IS named (that is row 3 of EXPECTATIONS-rete-defn.md, the load-bearing one), and \
         repoint the fixture's body at a non-rete op so it exercises the membrane rather than the \
         missing form. Diagnostic was: {rendered}"
    );
}
