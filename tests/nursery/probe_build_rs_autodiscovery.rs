//! FM-2-bis (build-system property): proof that build.rs auto-discovers test
//! files with ZERO manual step.
//!
//! This file was dropped into tests/nursery/ and NEVER added to any committed
//! `mod` list — there is no committed mod list anymore. If this test runs at
//! all, build.rs scanned the dir, generated the module declaration into OUT_DIR,
//! and the `include!` stub in mod.rs picked it up. The act of running IS the
//! proof; a forgotten test is now structurally impossible (the failure class the
//! old gen-test-mods.sh + --check gate guarded by hand is annihilated).
//!
//! The disconfirming companion is procedural: at the pre-build.rs HEAD, dropping
//! a file without running `gen-test-mods.sh` left it uncompiled (the dev-loop
//! window). After build.rs that window cannot exist.

// No-panic-IS-the-proof (named honestly — not a sentinel assertion): if this
// test appears in the run report at all, build.rs compiled a file no mod list
// ever named. The empty body is deliberate; the discovery is the assertion.
#[test]
fn this_test_was_auto_discovered_by_build_rs() {}
