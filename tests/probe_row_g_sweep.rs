//! Arc 170 Phase 1D Row G sweep — 50 trials at delay=0 via subprocess.
//!
//! Runs the probe_pdeathsig_diagnostic binary 50 times with
//! WAT_PROBE_SUPERVISOR_DELAY_MS=0 and counts pass/fail.
//! Pass criterion: 50/50 pass, 0 fail.
//!
//! This file is a one-shot tool for Row G verification; not a permanent
//! regression fixture. After Phase 1D SCORE is written, this file can
//! be deleted (it is not the historical artifact — probe_pdeathsig_diagnostic
//! is).

#[test]
fn row_g_50_trials_delay_zero() {
    // Find the test binary. We look for probe_pdeathsig_diagnostic-*
    // in the same deps directory as this binary.
    let our_path = std::env::current_exe().expect("current_exe failed");
    let deps_dir = our_path.parent().expect("no parent dir");

    // Find the diagnostic binary.
    let mut diagnostic_bin = None;
    for entry in std::fs::read_dir(deps_dir).expect("read deps dir") {
        let entry = entry.expect("dir entry");
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("probe_pdeathsig_diagnostic-")
            && !name_str.ends_with(".d")
            && !name_str.contains('.')
        {
            diagnostic_bin = Some(entry.path());
            break;
        }
    }
    let bin = diagnostic_bin.expect("could not find probe_pdeathsig_diagnostic binary in deps");
    eprintln!("[row_g] using binary: {}", bin.display());

    let trials = 50;
    let mut pass = 0usize;
    let mut fail = 0usize;

    for i in 0..trials {
        let output = std::process::Command::new(&bin)
            .arg("--quiet")
            .arg("--test")
            .arg("probe_pdeathsig_diagnostic")
            .env("WAT_PROBE_SUPERVISOR_DELAY_MS", "0")
            .output()
            .expect("failed to run diagnostic binary");
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("test result: ok") {
            pass += 1;
        } else {
            fail += 1;
            eprintln!("[row_g] trial {} FAIL — stdout: {}", i, stdout.trim());
        }
    }

    eprintln!("[row_g] delay=0: pass={} fail={} (out of {})", pass, fail, trials);
    assert_eq!(
        fail, 0,
        "Row G: {}/{} trials failed at delay=0 — lifeline mechanism not race-free",
        fail, trials
    );
}
