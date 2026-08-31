//! Excursus 002 stone 3 — a Handle param is an owning binding, downward only.
//!
//! The red probe carries three shapes side by side:
//! - `:red::drive-param` must be rejected by name (param + tail-escape)
//! - `:red::conn` must keep compiling (upward: a param is a borrow)
//! - `:red::held-param` must keep compiling (drive in a binding)
//!
//! It lives under `docs/excursus/.../probes/`, not `wat-scripts/`. A rune on it
//! would silence the wall's only proof that it fires.
//! Assert the kind structurally (no `.contains(` — `no_loose_string_assert`).

use wat::check::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn drive_param_is_rejected_and_conn_and_held_are_not() {
    let err = match startup_from_file(
        "docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-param-tail-escape.wat",
    ) {
        Err(e) => e,
        Ok(_) => panic!(
            "drive-param must not freeze — a green check here means the wall fired on nothing"
        ),
    };
    let StartupError::Check(errs) = err else {
        panic!("expected a check error (param tail escape); got {err:?}");
    };
    let hits: Vec<&CheckErrorKind> = errs
        .0
        .iter()
        .map(|e| &e.kind)
        .filter(|k| {
            matches!(
                k,
                CheckErrorKind::HandleTailEscape { function, .. }
                    if function == ":red::drive-param"
            )
        })
        .collect();
    assert!(
        !hits.is_empty(),
        "wall must name :red::drive-param; kinds were: {:?}",
        errs.0.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
    for k in &hits {
        match k {
            CheckErrorKind::HandleTailEscape {
                function,
                service,
                param,
                ..
            } => {
                assert_eq!(*function, ":red::drive-param");
                assert_eq!(*service, ":red::Alpha");
                assert_eq!(param.as_deref(), Some("h"));
            }
            _ => unreachable!(),
        }
    }
    let named = |want: &str| {
        errs.0.iter().any(|e| {
            matches!(
                &e.kind,
                CheckErrorKind::HandleTailEscape { function, .. }
                    if function == want
            ) || matches!(
                &e.kind,
                CheckErrorKind::HandleCreationEscape { function, .. }
                    if function == want
            )
        })
    };
    assert!(
        !named(":red::conn"),
        "conn must keep compiling — a hit means the widening leaked into the upward direction"
    );
    assert!(
        !named(":red::held-param"),
        "held-param must keep compiling — drive sits in a binding, so this frame outlives the call"
    );
    assert!(
        !named(":red::consume-peer"),
        "consume-peer takes a Peer as a param; it must not be named"
    );
}
