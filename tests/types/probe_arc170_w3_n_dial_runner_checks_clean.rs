//! Arc 255 Stone 255.75 — `probe_arc170_w3_n_dial_runner.wat` is a `--check`-only
//! type-composition fixture (no `:user::main`, by design — its own header line 1:
//! "freeze-only (--check), no fork"). It moved out of `wat-scripts/probes/` (the
//! new "every probe runs" gate requires `:user::main` and exit 0, which this file
//! structurally cannot satisfy) to live beside this, its consumer, per the
//! brief's "fragment" disposition.
//!
//! The probe's claim: a hand-written N=2 heterogeneous dial-runner (a `PoolMsg`
//! carrying `(Tuple [Address<Echo> Address<Kv>])`, connecting each component into
//! its own typed Peer, running a 2-peer work-fn) TYPE-COMPOSES. If `--check`
//! freezes clean, the claim holds (W3's codegen just needs to generalize what
//! already type-checks by hand).

use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn w3_n_dial_runner_checks_clean() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel = "tests/types/probe_arc170_w3_n_dial_runner.wat";
    assert!(manifest.join(rel).exists(), "fixture missing: {rel}");

    let out = Command::new(bin)
        .arg("--check")
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} --check {rel}: {e}"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "the hand-written N=2 heterogeneous dial-runner must type-compose; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
