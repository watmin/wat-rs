//! Excursus 003 step 4 (D4) — G4, no user frame.
//!
//! `RuntimeErrorKind::UserMainMissing` (`src/freeze.rs`, `invoke_user_main_orchestrated`)
//! is the one live producer of the "no frame is in user source" arm: it fires before any
//! `apply_function` call has run on this thread, so the captured `wat_frames` are empty,
//! and the raise span itself is a Rust call site (`src/freeze.rs`), never user source
//! either. Derivation must keep the raise span — there is nothing more local to point at.

use wat::freeze::{invoke_user_main, startup_from_file};
use wat::runtime::RuntimeErrorKind;

#[test]
fn g4_no_user_main_keeps_the_raise_span() {
    let world = startup_from_file("tests/diagnostics/probe_excursus003_step4_g4_no_user_frame.wat")
        .expect("a world with no :user::main still starts up");

    let err = invoke_user_main(&world, Vec::new())
        .expect_err(":user::main was never declared");

    assert!(
        matches!(err.kind(), RuntimeErrorKind::UserMainMissing),
        "expected UserMainMissing; got {:?}",
        err.kind()
    );
    assert_eq!(
        err.span().file.as_str(),
        "src/freeze.rs",
        "no frame is in user source (wat_frames is empty) — the raise span must be KEPT \
         (src/freeze.rs's own rust_caller_span!()); got {:?}",
        err.span()
    );
}
