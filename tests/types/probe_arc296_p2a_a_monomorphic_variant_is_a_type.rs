//! PROBE — a variant is not a type, and the annotation position now says so LOUDLY.
//!
//! Measured on the pushed green tree `5283b376b`:
//!
//! ```text
//!   (defn :user::f [c <- :usr::Colour::Red] …)   check=1
//!     UnknownNamedType "annotation names unknown type :usr::Colour::Red — not a declared type,
//!                       not a type variable, and not a builtin"
//!   (:wat::runtime::is-type? :usr::Colour::Red)  ->  false
//!   (:usr::Colour::Red {:shade 7})               types as  :usr::Colour        ← ERASED
//! ```
//!
//! ★ **P-1 is what makes this stone drawable.** Before it, a variant-typed annotation checked
//! CLEAN — accepted as a phantom opaque nominal type and erased. The rows below can now rest on
//! a refusal BECOMING an acceptance, which is falsifiable, instead of on "silent erasure becomes
//! correct typing," which is not.
//!
//! ## ⛔ Why registering the NAME without fixing the ERASURE would be a regression
//!
//! `Colour::Red <: Colour`, so a `Colour` cannot flow INTO a `Colour::Red` position. Register the
//! name alone and `[c <- :usr::Colour::Red]` becomes an **uninhabitable parameter**: accepted at
//! the annotation, unsatisfiable at every call site, because no expression produces that type. The
//! clear refusal we have today is strictly better than that. **Both halves or neither.**
//!
//! ## ⛔ THE SCOPE FENCE — this stone is MONOMORPHIC ONLY
//!
//! `generic_variant_stays_refused` must STAY refused. `Thread'<I,O> → :Spawned` (arc 209/267)
//! proves parametric subsumption works for a parametric child under a BARE parent. A variant edge
//! is parametric → parametric **with argument correspondence** — `Option::Some<T> → Option<T>` —
//! and edge identity is spelled as STRINGS. That is the class CLAUDE.md names as recurring, with
//! three instances in arc 278 alone. The fence is a measured fact here, not a promise elsewhere.
//!
//! Corpus: 104 monomorphic enum declarations, 20 generic.
//!
//! The three subject tests are `#[ignore]`d and are UN-IGNORED BY THE STONE.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(case: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_p2a_a_monomorphic_variant_is_a_type__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

/// Returns (exit code, stdout+stderr) — the message matters wherever the code alone is ambiguous.
fn check(case: &str) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(fixture(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}

fn run(case: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    assert!(out.status.success(), "fixture {case} did not run cleanly: {}",
            String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// ⛔ THE SUBSUMPTION CONTROL. Green today and the row that breaks if the ctor stops erasing
/// without the variant→enum edge carrying the value. This is the widest thing the stone can break.
#[test]
fn a_variant_value_still_flows_to_an_enum_parameter() {
    let (code, _) = check("variant_flows_to_enum_param");
    assert_eq!(code, 0, "`Colour::Red` must still be accepted where `Colour` is expected");
}

/// ⛔ THE SCOPE FENCE. A GENERIC enum's variant stays refused, and stays refused with the SAME
/// diagnostic — so "the stone stopped where it said" is measured, not asserted.
#[test]
fn a_generic_enums_variant_stays_refused() {
    let (code, out) = check("generic_variant_stays_refused");
    assert_eq!(code, 1);
    assert!(
        out.contains("UnknownNamedType"),
        "the generic case must remain an UNKNOWN TYPE refusal, not silently become something \
         else; got: {out}"
    );
}

/// SUBJECT — the annotation position accepts a monomorphic variant.
#[test]
#[ignore = "arc 296 P-2a — a variant is not a registered type"]
fn a_monomorphic_variant_is_accepted_in_annotation_position() {
    let (code, out) = check("variant_annotation");
    assert_eq!(code, 0, "`:usr::Colour::Red` names a real variant of a declared enum; got: {out}");
}

/// SUBJECT — the verb agrees with the wall, per P-3's rule.
#[test]
#[ignore = "arc 296 P-2a — a variant is not a registered type"]
fn is_type_answers_true_for_a_variant() {
    assert_eq!(run("is_type_on_a_variant"), "true");
}

/// ⛔ SUBJECT **and** the row a defect satisfies. This fixture exits 1 TODAY — because the
/// annotation names an unknown type. After the stone it must exit 1 for a COMPLETELY DIFFERENT
/// reason: a `Colour` is not known to be `Red`. Identical exit code, different mechanism, so the
/// bar is the MESSAGE. A stone that registered variants as ALIASES of their enum would flip this
/// to 0 and pass every other row in this file.
#[test]
#[ignore = "arc 296 P-2a — refused today as UnknownNamedType, not as a direction violation"]
fn an_enum_value_does_not_flow_into_a_variant_parameter() {
    let (code, out) = check("enum_does_not_flow_to_variant_param");
    assert_eq!(code, 1);
    assert!(
        !out.contains("UnknownNamedType"),
        "after the stone this must be a TYPE MISMATCH (a Colour is not known to be a Colour::Red), \
         NOT an unknown-type refusal — the exit code is the same either way; got: {out}"
    );
    assert!(
        out.contains("Colour::Red"),
        "the diagnostic must name the variant the caller failed to supply; got: {out}"
    );
}
