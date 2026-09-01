//! The `:impls` completeness guard — `features ⊆ impls`, never equality.
//!
//! The red probe carries three shapes side by side: `:probe::partial` must be
//! rejected (naming BOTH missing ops), `:probe::complete` must keep compiling,
//! `:probe::ticking` (extra internal `-tick`) must keep compiling.
//!
//! It lives under `docs/arc/.../probes/`, not `wat-scripts/`, because
//! `every_wat_scripts_file_loads` type-checks that tree — a must-be-rejected
//! file there turns the floor red for as long as the wall works. No rune on it:
//! runing the acceptance criterion would produce a green floor from a guard
//! that fires on nothing.

use wat::check::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn partial_satisfier_is_rejected_complete_and_internal_arm_are_not() {
    let err = match startup_from_file(
        "docs/arc/2026/06/278-rules-engine/probes/red-partial-satisfier.wat",
    ) {
        Err(e) => e,
        Ok(_) => panic!(
            "partial satisfier must not freeze — a green check here means the wall fired on nothing"
        ),
    };
    let StartupError::Check(errs) = err else {
        panic!("expected a check error (impls incomplete); got {err:?}");
    };
    let hits: Vec<&CheckErrorKind> = errs
        .0
        .iter()
        .map(|e| &e.kind)
        .filter(|k| {
            matches!(
                k,
                CheckErrorKind::ImplsIncomplete { service, .. } if service == ":probe::partial"
            )
        })
        .collect();
    assert!(
        !hits.is_empty(),
        "wall must name :probe::partial; kinds were: {:?}",
        errs.0.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
    for k in &hits {
        match k {
            CheckErrorKind::ImplsIncomplete {
                service,
                surface,
                missing,
            } => {
                assert_eq!(*service, ":probe::partial");
                assert_eq!(*surface, ":probe::Trio");
                assert!(
                    missing.iter().any(|m| m == "pong"),
                    "must name every missing op; pong absent from {missing:?}"
                );
                assert!(
                    missing.iter().any(|m| m == "pang"),
                    "must name every missing op; pang absent from {missing:?}"
                );
                assert_eq!(
                    missing.len(),
                    2,
                    "exactly the two missing features; got {missing:?}"
                );
            }
            _ => unreachable!(),
        }
    }
    let complete_named = errs.0.iter().any(|e| {
        matches!(
            &e.kind,
            CheckErrorKind::ImplsIncomplete { service, .. } if service == ":probe::complete"
        )
    });
    assert!(
        !complete_named,
        "complete satisfier must keep compiling — a hit means the rule is not features ⊆ impls"
    );
    let ticking_named = errs.0.iter().any(|e| {
        matches!(
            &e.kind,
            CheckErrorKind::ImplsIncomplete { service, .. } if service == ":probe::ticking"
        )
    });
    assert!(
        !ticking_named,
        "extra internal -tick must keep compiling — a hit means the rule is symmetric"
    );
}
