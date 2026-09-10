//! PROBE — the RESERVED-PREFIX WALL is not the `:wat::*` BLANKET. They share one predicate and
//! they are opposite things, and the blanket's death must not touch the wall.
//!
//! `is_reserved_prefix` (`src/resolve/reserved.rs:42`) has TWO consumers doing TWO jobs:
//!
//! ```text
//!   THE WALL      src/resolve/registration.rs:129
//!                 `privilege == Privilege::User && is_reserved_prefix(name)` → Reserved
//!                 USERLAND MAY NOT DEFINE UNDER `:wat::*` OR `:rust::*`. wat owns its own
//!                 root; rust modules express their own names. This is arc 255's OPENING
//!                 CONDITION and it STAYS.
//!
//!   THE BLANKET   src/resolve/walk.rs, `is_resolvable_call_head`
//!                 `if is_reserved_prefix(head) { return true; }`
//!                 any `:wat::*` head accepted as a CALL TARGET, unvalidated. The checker was
//!                 made overly permissive for `:wat::*` things. THIS is what arc 255 kills.
//! ```
//!
//! ⛔ ONE PREDICATE, TWO JOBS — which is exactly how a stone aimed at the second could take the
//! first with it. Deleting the blanket removes an *acceptance*; the wall is a *refusal*, reached
//! through a different call path (registration, not resolution). These rows hold that line so no
//! future stone can quietly widen `:wat::*` to userland while "finishing arc 255".
//!
//! ★ MEASURED BOTH WAYS. Every refusal row below was run against the tree WITH the blanket and
//! against the tree with the blanket DELETED (`d79597b3a` applied): identical verdicts, the same
//! `#wat.runtime/ReservedPrefix` naming the same prefixes. The wall does not depend on the
//! blanket, and now it is pinned rather than merely believed.
//!
//! The control row is the other half: the same six forms under the user's OWN prefix must stay
//! legal. A wall that refuses userland its own namespace has stopped being a wall.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn rel(case: &str) -> String {
    format!("tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__{case}.wat")
}

fn run_check(case: &str) -> (i32, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(rel(case)).exists(), "fixture missing: {}", rel(case));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg("--check")
        .arg(rel(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// The wall's own diagnostic, STRUCTURE-EXACT against a per-case golden — not the exit code,
/// which any error satisfies, including the resolve refusal the blanket's death introduces
/// elsewhere. Only `#wat.runtime/ReservedPrefix`, carrying this name and this span, proves THIS
/// wall fired. Each golden is machine-independent: `run_check` invokes the binary from
/// CARGO_MANIFEST_DIR with a RELATIVE fixture path.
macro_rules! refused_as_reserved_prefix {
    ($case:literal) => {{
        let (code, out) = run_check($case);
        assert_eq!(code, 1, "{}: userland must not claim this name.\n{out}", $case);
        wat::assert_edn_eq!(
            out,
            include_str!(concat!(
                "probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__",
                $case,
                ".edn"
            ))
        );
    }};
}

#[test]
fn a_user_defn_may_not_claim_a_wat_name() {
    refused_as_reserved_prefix!("defn_wat");
}

/// Rust modules express their own rust names; userland wat may not mint them either.
#[test]
fn a_user_defn_may_not_claim_a_rust_name() {
    refused_as_reserved_prefix!("defn_rust");
}

#[test]
fn a_user_defmacro_may_not_claim_a_wat_name() {
    refused_as_reserved_prefix!("defmacro_wat");
}

#[test]
fn a_user_defstruct_may_not_claim_a_wat_name() {
    refused_as_reserved_prefix!("defstruct_wat");
}

#[test]
fn a_user_typealias_may_not_claim_a_wat_name() {
    refused_as_reserved_prefix!("typealias_wat");
}

#[test]
fn a_user_defenum_may_not_claim_a_wat_name() {
    refused_as_reserved_prefix!("defenum_wat");
}

/// ★ CONTROL — the same six forms under the user's OWN prefix. Green now, green after. A wall
/// that also refuses userland its own namespace is over-wide, and nothing else here would say so.
#[test]
fn the_same_forms_under_a_user_prefix_are_all_legal() {
    let (code, out) = run_check("control_user_prefix");
    assert_eq!(code, 0, "userland owns its own prefix:\n{out}");
}
