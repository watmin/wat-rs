//! Arc 255 Stone 255.74 — `probe-compound-upcast.wat` keeps running after its Set case
//! retired.
//!
//! `wat-scripts/probes/arc-170/probe-compound-upcast.wat` (255.7x's positive gate for the
//! expected-type-directed up-cast rule — Tuple via ann-form, Map via call-arg, and formerly
//! Set via call-arg) had NO driven Rust test before this stone — `every_wat_scripts_file_loads`
//! only type-checks `wat-scripts/`, never runs it. Stone 255.74 made its Set case illegal
//! (`#{eh}` up-cast into `(HashSet :- [Capability])` — a service handle is never key-eligible)
//! and removed it, keeping only the Tuple and Map claims. This is the wire-in 255.73's own
//! header asked every repaired/trimmed probe to get: without it, a future rot of this probe
//! (or a future stone re-introducing a Set-of-resource claim) is loader-green and
//! run-time-silent again.
//!
//! Run: `cargo test --release --test probe_arc255_74_compound_upcast_runs`

use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn probe_compound_upcast_checks_and_runs_clean() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel = "wat-scripts/probes/arc-170/probe-compound-upcast.wat";
    assert!(manifest.join(rel).exists(), "probe fixture missing: {rel}");

    let check_out = Command::new(bin)
        .arg("--check")
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} --check {rel}: {e}"));
    assert_eq!(
        check_out.status.code(),
        Some(0),
        "probe-compound-upcast.wat must --check clean (Tuple-via-ann-form and Map-via-call-arg \
         up-casts, Set case retired by Stone 255.74); stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check_out.stdout),
        String::from_utf8_lossy(&check_out.stderr)
    );

    let run_out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel}: {e}"));
    let stdout = String::from_utf8_lossy(&run_out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&run_out.stderr).into_owned();
    assert_eq!(
        run_out.status.code(),
        Some(0),
        "probe-compound-upcast.wat must run cleanly; stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout, "\"compound-upcast: ok\"\n",
        "the probe's own documented claim; stderr:\n{stderr}"
    );
}
