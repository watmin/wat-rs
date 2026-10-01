//! Arc 255 Stone 255.73 — repair the rotted probe, and make it RUN.
//!
//! `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` (259 S3b Blocker A's "disconfirming
//! probe first" — `docs/arc/2026/06/259-forced-hand/NOTE-S3b-blockers-and-resolution.md`) PROVES
//! the derive-and-splice mechanism: extract a work-fn's concrete arg/return type AST nodes off
//! its `fn-forms` reification, splice them into a shipped process-runner's `self-peer`/`Peer`
//! tuple types via quasiquote, spawn the runner as a child PROCESS, and drain two round trips.
//!
//! 255.72 found it rotted SILENTLY: `every_wat_scripts_file_loads` only type-checks `wat-scripts/`
//! files, never RUNS them, so the probe crashed at run time (line 59, an angle-bracket keyword
//! `Peer<...>` built by string concatenation — illegal since arc 109 "annihilate the angle
//! bracket") before AND after that stone, with nothing to notice. 255.73 repaired it in today's
//! language (the type FORM `(Head :- [args])`, spliced off the raw type-position AST nodes
//! directly — no `ast-name`/string round-trip) and fixed a second, previously-unreached defect
//! the repair exposed (`__runner`'s `recv` bare-bound the `RecvOutcome` wrapper instead of
//! matching it). THIS TEST is the wire-in: without it, a future rot of this exact probe is
//! loader-green and run-time-silent again, same as before.
//!
//! Run: `cargo test --release --test probe_arc170_s3b_astsplice`

use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn probe_s3b_astsplice_runs_and_prints_6_10() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel = "wat-scripts/probes/arc-170/probe-s3b-astsplice.wat";
    assert!(manifest.join(rel).exists(), "probe fixture missing: {rel}");

    let out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel}: {e}"));

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(
        out.status.code(),
        Some(0),
        "probe-s3b-astsplice.wat must run cleanly (the derive-and-splice mechanism it proves \
         must keep working); stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout, "\"6 10\"\n",
        "the probe's own documented claim (header: `EXPECT \"6 10\"`) — the parent sends \
         (0,3)/(1,5), the spliced-typed process runner doubles the payload via the derived \
         work-fn and sends back the index-tagged results; stderr:\n{stderr}"
    );
}
