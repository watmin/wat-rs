//! Arc 278 #88 — THE ACCEPTANCE GATE for the rete `defn`, with its non-vacuity control.
//!
//! WHY: an ordinary `defn` called from a `where` used to be rete-admissible *by accident of its
//! current body*. Nobody declared it, so there was no contract, so nothing could be broken, so
//! nothing warned — editing one op inside such a helper failed naming the RULE, with not one
//! frame naming the helper. #88 minted `(:wat::rete::core::defn …)`: same registration and symbol
//! binding as `:wat::core::defn`, body checked tighter AT THE DEFINITION SITE.
//!
//! WAS a disconfirming probe pinning that the form did not exist. NOW it is the acceptance gate
//! for THE MEMBRANE: the fixture declares `:probe::declared` with a body that is deliberately a
//! NON-rete op (`:wat::core::i64::>`, not the `:wat::rete::i64::>` RETE_OPS row) — the
//! exact "reproduced live" shape DESIGN-STONE-the-rete-defn.md opens with. That declaration must
//! be refused AT LOAD, with a located error naming `:probe::declared` directly — the inversion
//! the whole stone exists for (today's `where`-clause failure used to name the calling RULE).
//!
//! Its non-vacuity control is byte-identical minus form #2, and MUST load — without it the RED
//! below would prove "something in the fixture is bad", not "exactly this declaration is
//! refused" (R59 `NISI FRANGAS, NIHIL PROBAS`).
//!
//! MUTATION, both directions (see the rider's report for the hand-run): repoint the fixture's
//! form #2 body back to `:wat::rete::i64::>` (rete-clean) and the file loads GREEN: the
//! membrane's refusal flips off exactly when the body becomes law-A clean, proving this gate can
//! actually go both ways rather than being permanently red or permanently green by construction.

use wat::freeze::{startup_from_file, StartupError};

const GAP: &str = "tests/rete/probe_arc278_rete_defn_gap.wat.bad";
const CONTROL: &str = "tests/rete/probe_arc278_rete_defn_gap_control.wat";

/// THE MEMBRANE BITES — a `(:wat::rete::core::defn …)` declaration whose body is a non-rete op
/// (law A violated) is refused AT LOAD. EXPECTATIONS-rete-defn.md row 2.
#[test]
fn rete_defn_with_non_rete_body_refused_at_definition() {
    let err = startup_from_file(GAP).expect_err(
        "(:wat::rete::core::defn :probe::declared …) has a non-rete body (:wat::core::i64::>) — \
         the definition-site check must refuse to load this file",
    );
    // The definition-site check runs post-registration (`freeze::env::build_env`'s step
    // 6.975), inside the same pipeline stage `register_defines`'s own collisions surface
    // through — a `RuntimeError` wrapping `RuntimeErrorKind::ReteDefnAxisViolation`.
    let StartupError::Runtime(re) = &err else {
        panic!("expected StartupError::Runtime(ReteDefnAxisViolation), got {err:?}");
    };
    let rendered = format!("{re:?}");
    assert!(
        // rune:lint(loose-assert) — the rendering is machine-specific EDN (an absolute source
        // path, a live span) that a golden can't pin; a targeted PRESENCE check for the error
        // kind's own EDN tag is the precise claim available, mirroring this file's ABSENCE
        // check below for the same reason.
        rendered.contains("#wat.runtime/ReteDefnAxisViolation"),
        "expected the rete-defn axis-violation error kind, got: {rendered}"
    );
}

/// THE NON-VACUITY CONTROL — delete only that form and the very same file loads. Without this,
/// the RED above would prove "something in the fixture is bad", not "exactly this declaration is
/// what's refused".
#[test]
fn control_without_the_rete_defn_form_loads() {
    startup_from_file(CONTROL).unwrap_or_else(|e| {
        panic!(
            "the control must load — if it does not, the gate's RED no longer isolates the \
             rete-defn declaration and BOTH tests are lying. Fix this first. Got: {e:?}"
        )
    });
}

/// THE ERROR NAMES THE HELPER — not the rule, not the caller: `:probe::declared` itself, the
/// declared helper whose body failed. This is the inversion the whole stone exists for
/// (EXPECTATIONS-rete-defn.md row 3, the one load-bearing row easiest to ship broken: a membrane
/// that refuses correctly but still doesn't say WHO fixes nothing a reader would notice).
#[test]
fn the_diagnostic_names_the_helper() {
    let err = startup_from_file(GAP).expect_err("fixture must fail to load");
    let StartupError::Runtime(re) = &err else {
        panic!("expected StartupError::Runtime(ReteDefnAxisViolation), got {err:?}");
    };
    let rendered = format!("{re:?}");
    assert!(
        // rune:lint(loose-assert) — same reason as above: the rendering embeds a machine-specific
        // absolute path and live span, so a full golden can't pin it; this is a targeted PRESENCE
        // check for the one fact this test exists to prove — the helper's own FQDN appears.
        rendered.contains(":probe::declared"),
        "the diagnostic must name the declared helper `:probe::declared` directly — an ordinary \
         `defn` used to fail this same shape naming only the calling RULE, with not one frame \
         naming the helper; that inversion IS the stone. Diagnostic was: {rendered}"
    );
}
