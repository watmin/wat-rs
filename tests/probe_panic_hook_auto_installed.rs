//! Proves `#[ctor]` auto-installs `panic_hook` at library load.
//! No explicit `panic_hook::install()` call in this test — if
//! `is_installed()` returns true, the ctor fired before `main()`.
//!
//! Arc 211a.

#[test]
fn panic_hook_auto_installed_via_ctor() {
    assert!(
        wat::panic_hook::is_installed(),
        "panic_hook should be auto-installed via #[ctor] at library load \
         (no explicit install() call in this test)"
    );
}
