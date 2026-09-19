//! Controls that eliding bake-time 8b / 8d(ALL-fns) / 8f does not elide the USER half.
//!
//! DESIGN: `docs/excursus/2026/08/001-sns-sqs/the-check-skips-what-the-build-already-proved/`
//!
//! Each test arms the boot cache, freezes a green program (stamping `infer_fresh_consumed`
//! so the next freeze elides), then freezes a USER-body offence. If the elision skipped
//! user bodies, the second freeze would be green.

use std::sync::Arc;

use wat::freeze::{startup_from_source, StartupError};
use wat::load::loader::InMemoryLoader;

const DIR: &str = "tests/function/";
const STEM: &str = "probe_tier_b_c_user_half_still_swept";

fn fixture(suffix: &str) -> String {
    let path = format!("{DIR}{STEM}{suffix}");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {path:?} must exist (run from crate root): {e}"))
}

fn cache_dir() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "wat-b-c-elision-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&p);
    p
}

fn arm_cache() {
    let dir = cache_dir();
    std::env::set_var("WAT_BOOT_CACHE_DIR", &dir);
    std::env::set_var("WAT_BOOT_CACHE", "on");
}

fn freeze(suffix: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(
        &fixture(suffix),
        Some("consumer.wat"),
        Arc::new(InMemoryLoader::new()),
    )
}

/// Stamp the cache so the next freeze in this process elides bake-time 8b/8d/8f.
fn stamp_elision() {
    freeze("_green.wat").expect("green program must freeze to stamp infer_fresh_consumed");
}

#[test]
fn user_body_type_error_still_reddens_when_stdlib_8f_is_elided() {
    arm_cache();
    stamp_elision();
    freeze("_typeerr.wat.bad")
        .expect_err("a user body `i64::+ 1 \"x\"` must still redden after bake-time 8f is elided");
}

#[test]
fn user_body_legacy_let_star_still_reddens_when_stdlib_8b_is_elided() {
    arm_cache();
    stamp_elision();
    freeze("_letstar.wat.bad").expect_err(
        "a user body naming `:wat::core::let*` must still redden after bake-time 8b is elided",
    );
}

#[test]
fn the_elided_sweeps_still_walk_non_bake_time_bodies() {
    let src = include_str!("../../src/check.rs");
    // 8b / 8d / 8f skip bake-time paths only — a `continue` that is not gated
    // on `is_bake_time_path` would elide the user half.
    for needle in [
        "if elide_bake_time.is_some() && is_bake_time_path(name)",
        "if elide_bake_time.is_some() && bake",
        "validate_def_positions_in_forms(forms, &mut errors)",
    ] {
        assert!(
            src.contains(needle),
            "elision skip is gone or now unguarded (would drop the user half): missing {needle}"
        );
    }
}
