//! PROBE — a PARAMETRIC vector field of a variant map-ctor is CHECKED. Ratchet fired.
//!
//! The fixture beside this file was `wat-scripts/scratch-pad/probe-parametric-vector-field-is-
//! unchecked.wat` (`02dc901a2`), and it existed to PIN A DEFECT: `--check` exited 0 on a bogus
//! field name inside a vector literal whose element type was parametric. Nothing inferred it.
//!
//! ★ It was cured by two fixes aimed elsewhere — `expand_form` learning to walk MAP and SET (an
//! unexpanded head carries no scheme, so `infer` returned a fresh var and `assignable` passed
//! trivially), and field instantiation ceasing to drop a `Parametric`'s type arguments. The
//! defect's own probe is the evidence, which is why it is kept and inverted rather than deleted.
//!
//! ⚠ Its relocation is load-bearing, not tidying: under `wat-scripts/` it sat beneath a gate that
//! requires every file to LOAD CLEANLY, so a fixture whose job is to be REFUSED reads there as
//! rot — and did, as the floor's `every_wat_scripts_file_loads` red. Arc 255 Stone 4 made exactly
//! this move for exactly this reason.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn check() -> (i32, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let rel = "tests/types/probe_arc251_parametric_vector_field_is_checked.wat";
    assert!(root.join(rel).exists(), "fixture missing: {rel}");
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg("--check")
        .arg(rel)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// The inverted assertion. `exit 0` here was the FINDING; `exit 1` is the cure.
#[test]
fn a_bogus_field_in_a_parametric_vector_element_is_refused() {
    let (code, out) = check();
    assert_eq!(code, 1, "the bogus field must now be REFUSED at --check; got:\n{out}");
    // Structural golden: the error must NAME the unknown field and the aggregate's declared set,
    // not merely be some error. A test that only asserted `exit 1` would pass if the file broke
    // for any unrelated reason — which is precisely how the arc255 probe misled a ruling today.
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_parametric_vector_field_is_checked.edn"
    );
}
