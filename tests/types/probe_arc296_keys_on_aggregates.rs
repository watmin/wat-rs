//! PROBE — `{:keys [...]}` destructures EVERY aggregate kind, not just `defstruct`.
//!
//! Arc 257.2 minted `{:keys [x y]}`. Its probe only ever exercised `defstruct`, so the
//! `defrecord` path was never wired and no test ever asked. Measured on the green tree:
//!
//! ```text
//!                          {:keys [x y]}      {x :x  y :y}
//!   defstruct                 check=0            check=0
//!   defrecord                 check=1  ⛔        check=0
//!   :wat::holon::defrecord    check=1  ⛔        check=0
//!   defholon                  check=1  ⛔        check=0
//! ```
//!
//! ⛔ THE GUARD IS A FOSSIL OF THE CHANGE THAT MADE IT UNNECESSARY. `src/check.rs:12725`:
//!
//! ```text
//!   // Arc 293.2b — struct-destructure requires Aggregate with kind==Struct.
//!   Some(TypeDef::Aggregate(a)) if a.nature == Nature::Struct => a.clone(),
//! ```
//!
//! 293.2b is the arc that UNIFIED struct and record into one `AggregateDef` — `StructDef` and
//! `RecordDef` do not exist. The predicate cites that unification as its authority while
//! encoding the world from before it.
//!
//! ★ AND IT IS BACKWARDS. `Nature::is_pure` (`types.rs:224`): *"Struct permits impurity (holds
//! resources); Record/HolonRecord guarantee purity."* The natures differ in PURITY, not SHAPE —
//! all four are aggregates with named fields, and destructuring READS fields. So the guard
//! permits destructuring the one nature that may hold a live socket, and refuses the three that
//! are guaranteed pure. `AGGREGATE-MODEL.md` records no design reason for the restriction.
//!
//! ⚠ EVERY BAR IS THE CONTROL, RUN IN THE SAME TEST. The control is the BINDER-FIRST form on a
//! `defrecord` — the same aggregate, the same fields, a form that already works. It isolates the
//! `{:keys}` reader as the only variable. A hand-written `== 0` would also pass on a mis-aimed
//! harness; comparing to a control that must stay green cannot.
//!
//! Un-ignored by the stone.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn check(case: &str) -> i32 {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_keys_on_aggregates__{case}.wat"));
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    out.status.code().unwrap_or(-1)
}

/// The bar. GREEN now and must stay green: the binder-first form on the SAME aggregate.
#[test]
fn the_binder_first_control_still_destructures_a_record() {
    assert_eq!(
        check("binder_first_control"),
        0,
        "`{{x :x}}` on a defrecord already works; a failure here means the harness is broken, \
         not that the stone's subject is"
    );
}

/// GREEN now — the one kind arc 257.2's probe happened to exercise. Must not regress.
#[test]
fn keys_destructures_a_defstruct() {
    assert_eq!(check("defstruct"), check("binder_first_control"));
}

#[test]
fn keys_destructures_a_defrecord() {
    assert_eq!(
        check("defrecord"),
        check("binder_first_control"),
        "a defrecord has named fields exactly as a defstruct does; the natures differ in \
         PURITY, not SHAPE, and destructuring reads fields"
    );
}

#[test]
fn keys_destructures_a_holon_defrecord() {
    assert_eq!(check("holon_defrecord"), check("binder_first_control"));
}

#[test]
fn keys_destructures_a_defholon() {
    assert_eq!(check("defholon"), check("binder_first_control"));
}
