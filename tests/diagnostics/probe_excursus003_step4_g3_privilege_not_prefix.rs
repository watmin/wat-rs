//! Excursus 003 step 4 (D4) — G3, privilege, not prefix.
//!
//! A user program whose path string BEGINS WITH `wat/` (e.g. run from a temp dir
//! holding `wat/foo.wat`) must still classify as user source — the wall is provenance
//! (was this label genuinely read as the entry program / a load target?), never a
//! `starts_with("wat/")` shortcut. See the co-located `.wat` fixture for the overflow
//! (reused from G1's own repro shape).

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::loader::{InMemoryLoader, SourceLoader};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/diagnostics/probe_excursus003_step4_g3_privilege_not_prefix.wat"
);
const FAKE_LABEL: &str = "wat/foo.wat";

#[test]
fn g3_a_path_starting_with_wat_is_still_user_source() {
    let src = std::fs::read_to_string(FIXTURE).expect("fixture must be readable");
    let loader: Arc<dyn SourceLoader> = Arc::new(InMemoryLoader::new());
    let world = startup_from_source(&src, Some(FAKE_LABEL), loader)
        .expect("wat/foo.wat is not a real stdlib label — must start up");

    let func = world.symbols().get(":user::grow").expect(":user::grow defined");
    let err = wat::runtime::apply_function(
        func.clone(),
        vec![wat::runtime::Value::i64(41)],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect_err("41 + i64::MAX must overflow");

    assert_eq!(
        err.span().file.as_str(),
        FAKE_LABEL,
        "a user file whose path merely STARTS WITH wat/ must still be classified as \
         user source — :location must be its own line, not wat/core.wat; got {:?}",
        err.span()
    );
}
