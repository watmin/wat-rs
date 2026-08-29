//! `edn::write` on a value it cannot tag must RAISE, not abort the process.
//!
//! Before 2026-08-29 `value_to_edn_with` returned a bare `OwnedValue` and `panic!`ed in its holon
//! arm. The failure channel already existed one frame up — `eval_edn_write` has always returned
//! `Result<Value, RuntimeError>` — so the failure had nowhere to go only because the callee could
//! not express it. Worse, the discarded error was ALREADY a located `TypeMismatch` from
//! `from_holon_item`: the panic stringified a good diagnostic and then aborted.
//!
//! ⛔ THE SECOND ROW IS THE LOAD-BEARING ONE. Asserting "it errors" alone would pass just as
//! happily if the encoder had been made to return `Err` for EVERYTHING. The good door must still
//! produce its exact bytes.

use wat::freeze::startup_from_file;
use wat::runtime::apply_function;
use wat::value::Value;

const FIXTURE: &str = "tests/diagnostics/probe_edn_write_unencodable_is_a_diagnostic.wat";

fn call(entry: &str) -> Result<Value, String> {
    let world = startup_from_file(FIXTURE).map_err(|e| format!("startup: {e:?}"))?;
    let func = world
        .symbols()
        .get(entry)
        .unwrap_or_else(|| panic!("fixture must define {entry}"));
    apply_function(func.clone(), vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn an_unencodable_holon_raises_instead_of_panicking() {
    // THE GOOD DOOR — exact bytes, so a blanket-Err regression cannot pass this test.
    let good = call(":user::good").expect("a classifier-wrapped holon must still encode");
    let Value::String(rendered) = &good else {
        panic!("edn::write must return a String; got {good:?}")
    };
    wat::assert_edn_eq!(
        rendered.as_str().to_string(),
        include_str!("probe_edn_write_unencodable_is_a_diagnostic__good_door.edn"),
        "a classifier-wrapped holon must still encode, and encode to exactly this — without this \
         row, making the encoder return Err for EVERYTHING would pass the test below"
    );

    // THE BAD DOOR — a diagnostic, and one that still teaches.
    let err = call(":user::compute").expect_err("an unencodable holon must not encode");
    for needle in ["TypeMismatch", "wat::edn::write", "unclassified"] {
        // rune:lint(loose-assert) — targeted presence of three independent facts in one long EDN
        // error face; pinning the whole rendered diagnostic would freeze a span and prose that
        // are meant to be improvable.
        assert!(
            err.contains(needle),
            "the failure must be a LOCATED diagnostic naming the op and the shape — the panic it \
             replaced stringified exactly this information and then aborted. missing {needle:?} \
             in: {err}"
        );
    }
    // rune:lint(loose-assert) — targeted absence: the whole point of the change.
    assert!(
        !err.contains("Panic"),
        "an encode failure is data-dependent, not an invariant violation — it may not surface as \
         a Panic: {err}"
    );
}
