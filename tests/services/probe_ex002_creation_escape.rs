//! Excursus 002 stone 1 — the creation-scope escape wall.
//!
//! The red probe carries both shapes side by side: `:red::conn` must keep compiling,
//! `:red::dial-and-drop` must be rejected by name.
//!
//! It lives under `docs/excursus/.../probes/`, not `wat-scripts/`, because
//! `every_wat_scripts_file_loads` type-checks that tree — a must-be-rejected file there turns the
//! floor red for as long as the wall works. Runing it instead would silence the wall's only proof
//! that it fires at all.
//! Assert the kind structurally (no `.contains(` — `no_loose_string_assert`).

use wat::check::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn dial_and_drop_is_rejected_and_conn_is_not() {
    let err = match startup_from_file(
        "docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-creation-escape.wat",
    ) {
        Err(e) => e,
        Ok(_) => panic!(
            "dial-and-drop must not freeze — a green check here means the wall fired on nothing"
        ),
    };
    let StartupError::Check(errs) = err else {
        panic!("expected a check error (creation-scope escape); got {err:?}");
    };
    let hits: Vec<&CheckErrorKind> = errs
        .0
        .iter()
        .map(|e| &e.kind)
        .filter(|k| {
            matches!(
                k,
                CheckErrorKind::HandleCreationEscape { function, .. }
                    if function == ":red::dial-and-drop"
            )
        })
        .collect();
    assert!(
        !hits.is_empty(),
        "wall must name :red::dial-and-drop; kinds were: {:?}",
        errs.0.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
    for k in &hits {
        match k {
            CheckErrorKind::HandleCreationEscape { function, service, .. } => {
                assert_eq!(*function, ":red::dial-and-drop");
                assert_eq!(*service, ":red::Alpha");
            }
            _ => unreachable!(),
        }
    }
    let conn_named = errs.0.iter().any(|e| {
        matches!(
            &e.kind,
            CheckErrorKind::HandleCreationEscape { function, .. }
                if function == ":red::conn"
        )
    });
    assert!(
        !conn_named,
        "conn-is-safe must keep compiling — a hit means the rule was keyed on the parameter"
    );
}
