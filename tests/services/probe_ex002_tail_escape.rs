//! Excursus 002 stone 2 — the tail-escape wall.
//!
//! The red probe carries four shapes side by side:
//! - `:red::tail-escape` must be rejected by name
//! - `:red::held` must keep compiling (drive in a binding — condition 2)
//! - `:red::builtin-head` must keep compiling (`:wat::i64::+` emits no TailCall)
//! - `:red::conn` must keep compiling (handle arrives as a param)
//!
//! It lives under `docs/excursus/.../probes/`, not `wat-scripts/`. A rune on it
//! would silence the wall's only proof that it fires.
//! Assert the kind structurally (no `.contains(` — `no_loose_string_assert`).

use wat::check::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn tail_escape_is_rejected_and_held_and_builtin_are_not() {
    let err = match startup_from_file(
        "docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-tail-escape.wat",
    ) {
        Err(e) => e,
        Ok(_) => panic!(
            "tail-escape must not freeze — a green check here means the wall fired on nothing"
        ),
    };
    let StartupError::Check(errs) = err else {
        panic!("expected a check error (tail escape); got {err:?}");
    };
    let hits: Vec<&CheckErrorKind> = errs
        .0
        .iter()
        .map(|e| &e.kind)
        .filter(|k| {
            matches!(
                k,
                CheckErrorKind::HandleTailEscape { function, .. }
                    if function == ":red::tail-escape"
            )
        })
        .collect();
    assert!(
        !hits.is_empty(),
        "wall must name :red::tail-escape; kinds were: {:?}",
        errs.0.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
    for k in &hits {
        match k {
            CheckErrorKind::HandleTailEscape {
                function,
                service,
                ..
            } => {
                assert_eq!(*function, ":red::tail-escape");
                assert_eq!(*service, ":red::Alpha");
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
        !named(":red::held"),
        "held must keep compiling — a hit means condition 2 (let itself in tail with a user-fn tail expr) was ignored"
    );
    assert!(
        !named(":red::builtin-head"),
        "builtin-head must keep compiling — a hit means the wall treated :wat::i64::+ as a user-function TailCall"
    );
    assert!(
        !named(":red::conn"),
        "conn-is-safe must keep compiling — a hit means the rule was keyed on the parameter"
    );
    assert!(
        !named(":red::consume-peer"),
        "consume-peer takes a Peer as a param; it must not be named"
    );
}
