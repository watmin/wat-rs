//! Stone 255.43 — journal' :init faces a failed ensure-schema.
//!
//! The store replies `EnsureSchemaResponse.Fatal`. `/start` of journal'
//! must raise that failure. Pre-stone the outcome was bound as `_es` and
//! dropped, and `/start` returned a handle.

use wat::assertion::AssertionPayload;
use wat::freeze::startup_beside;
use wat::runtime::apply_function;

const SENTINEL: &str = "journal ensure-schema fatal: SCHEMA-SETUP-FAILED";

#[test]
fn a_failed_ensure_schema_fails_journal_start() {
    let world = startup_beside(file!()).expect("the probe loads");
    let func = world
        .symbols()
        .get(":user::compute")
        .expect(":user::compute")
        .clone();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
    }));
    match caught {
        Ok(Ok(value)) => panic!("journal start swallowed the schema failure and returned {value:?}"),
        Ok(Err(err)) => panic!("journal start returned an error instead of the schema failure: {err:?}"),
        Err(payload) => {
            let message = payload
                .downcast_ref::<AssertionPayload>()
                .map(|p| p.message.as_str())
                .unwrap_or_else(|| panic!("journal start raised a payload that is not an assertion: {payload:?}"));
            assert_eq!(message, SENTINEL);
        }
    }
}
