//! Arc 170 closure #6 — the ps-visible spawned-process label, end-to-end.
//!
//! Every test here drives a REAL `wat` subprocess via `Command::new(CARGO_BIN_EXE_wat)`
//! (the `tests/cli/wat_cli.rs` pattern), never an in-process `spawn_process_peer` call —
//! that call forks, and forking inside cargo's own multi-threaded test binary is exactly
//! the hazard `tests/kernel/spawn_program_prime_process.rs` walls off behind `#[ignore]` +
//! `integration-run.sh`. Going through a freshly-exec'd, single-threaded `wat` process for
//! the OUTER driver sidesteps that hazard entirely: ITS OWN fork (to spawn the labeled
//! child) happens inside a process cargo never forked.
//!
//! ## THE WALL, gated
//!
//! 1. `label_present_child_argv_stays_empty_and_shows_in_real_proc_cmdline` — the
//!    positive case: a `#wat.process/Service {...}` label lands in the REAL OS argv
//!    (read back from `/proc/<pid>/cmdline`, not a Rust-internal buffer), AND the
//!    labeled child's own `(:wat::runtime::argv)` is independently proven empty (it
//!    self-asserts this before reporting anything — see the fixture).
//! 2. `unlabeled_process_argv_is_exe_only` — the negative control: `ProcessOpts/label`
//!    defaulting to `:None` leaves argv exactly `[exe]`, unchanged from before this
//!    field existed.
//! 3. `forged_label_at_shell_accomplishes_nothing` — THE WALL itself: a shell invocation
//!    handed a fake `#ns/Thing {}` argument, with no fd 3 (`was_spawned()` false), is
//!    read as an ordinary (bogus) entry-file path — identical behavior to any other
//!    garbage argv[1] — never specially routed.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

const LABELED_FIXTURE: &str = "tests/process/wat_arc170_closure6_label_wall_labeled.wat";
const UNLABELED_FIXTURE: &str = "tests/process/wat_arc170_closure6_label_wall_unlabeled.wat";

/// The label a labeled child must render into its real OS argv, and the forged
/// one the wall test feeds at a shell. Both live in co-located `.edn` goldens
/// (the `no_inlined_edn` rubric): the first is compared STRUCTURALLY via
/// `wat::assert_edn_eq!`, the second is the literal argv byte-string under test,
/// so the two never drift from one hand-copied spelling.
const SERVICE_LABEL_EDN: &str = include_str!("wat_arc170_closure6_label_wall__service_label.edn");
const FORGED_LABEL_EDN: &str = include_str!("wat_arc170_closure6_label_wall__forged_label.edn");

/// `argv[0]` of a spawned child is `ExecPlan::build`'s `runtime_binary()` — the
/// RESOLVED interpreter path, not the caller's spelling. Assert it byte-exact
/// against the canonicalized test binary; `ends_with("wat")` would pass on
/// `/usr/bin/not-really-wat` and on any appended garbage.
fn assert_argv0_is_the_wat_binary(argv0: &str) {
    let expected = std::fs::canonicalize(env!("CARGO_BIN_EXE_wat"))
        .expect("canonicalize the wat test binary");
    assert_eq!(
        std::path::Path::new(argv0),
        expected.as_path(),
        "argv[0] must be the resolved wat binary, byte-exact"
    );
}

/// Read `/proc/<pid>/cmdline` and split it into its NUL-separated argv fields.
/// A trailing NUL (the normal case for a live process) leaves no trailing empty
/// field — `cmdline` always ends in NUL when non-empty, so `split('\0')` would
/// otherwise report one phantom empty tail entry.
fn read_proc_cmdline(pid: i32) -> Vec<String> {
    let raw = std::fs::read(format!("/proc/{pid}/cmdline"))
        .unwrap_or_else(|e| panic!("read /proc/{pid}/cmdline: {e}"));
    raw.split(|&b| b == 0)
        .filter(|field| !field.is_empty())
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect()
}

/// Drive an outer fixture through the shared protocol: spawn it, read ONE line of
/// stdout (the labeled/unlabeled child's self-reported pid), read `/proc/<pid>/cmdline`
/// WHILE both the outer process and its held child peer are still blocked (guaranteed
/// alive — neither has been told to proceed yet), then release the outer and wait for a
/// clean exit. Returns the child's real OS argv fields.
fn drive_and_read_child_cmdline(fixture: &str) -> Vec<String> {
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(fixture)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn outer wat driver");

    let mut stdout = BufReader::new(child.stdout.take().expect("outer stdout"));
    let mut pid_line = String::new();
    stdout
        .read_line(&mut pid_line)
        .expect("read the labeled child's self-reported pid");
    let child_pid: i32 = pid_line
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("child pid line {pid_line:?} must parse as i32: {e}"));

    // The moment of truth: read the REAL OS argv while the child is provably still
    // alive (blocked on its own readln; the outer hasn't been released yet, so it
    // hasn't dropped the peer that would EOF the child's stdin).
    let argv_fields = read_proc_cmdline(child_pid);

    // Release the outer: it drops the child peer (EOF -> child's readln exits ->
    // child exits cleanly) then exits itself. `readln` is EDN-only (the kernel
    // stdio contract), so the release line must be a valid EDN form — a bare `1`.
    child
        .stdin
        .take()
        .expect("outer stdin")
        .write_all(b"1\n")
        .expect("release the outer driver");

    let status = child.wait().expect("wait for outer driver");
    assert!(
        status.success(),
        "outer driver must exit cleanly; status = {status:?}"
    );

    argv_fields
}

#[test]
fn label_present_child_argv_stays_empty_and_shows_in_real_proc_cmdline() {
    let argv_fields = drive_and_read_child_cmdline(LABELED_FIXTURE);

    // The fixture's assert-eq (0 == length of the CHILD's own ambient argv) already
    // ran inside the child before it reported anything; a failure there would have
    // panicked the child, surfaced through the outer's assertion-failed! raise, and
    // failed drive_and_read_child_cmdline's `status.success()` assertion above. So
    // reaching this line already proves invariant #2 (label present, ambient argv
    // stays empty) — this is the "assert it rather than trust it" the brief asked for,
    // just proven by the child's OWN check rather than re-derived here.

    assert_eq!(
        argv_fields.len(),
        2,
        "a labeled process must show exactly [exe, label] in its real OS argv; got {argv_fields:?}"
    );
    assert_argv0_is_the_wat_binary(&argv_fields[0]);

    // The golden is a TEMPLATE because `:line` is edit-specific — wherever
    // `(:probe::labeled-locus)` currently sits. It is DERIVED from the fixture below rather
    // than hardcoded, so moving that call cannot silently rot this into a lie; and it is not
    // weakened to a `.contains`, which would stop proving the origin is the CALLER's position
    // (the whole point) and start passing on any file and any line.
    //
    // `:file` is no longer machine-specific: spans now carry the REPO-RELATIVE path
    // (`load::span_display_path`), so the fixture constant IS the expected label verbatim.
    // This assertion USED to canonicalize at test time to dodge an absolute path — that
    // workaround is what the normalization retired.
    let fixture_path = LABELED_FIXTURE;
    let fixture_src = std::fs::read_to_string(LABELED_FIXTURE).expect("read the labeled fixture");
    // Located by MARKER, not by matching the call's source text: the latter also matches
    // the fixture's own prose about it (it did — this assertion caught it) and would put an
    // inlined wat form in this .rs (no_inlined_wat_in_tests, which also fired).
    let marker = "<<origin-call>>";
    let marked: Vec<usize> = fixture_src
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains(marker))
        .map(|(i, _)| i + 1)
        .collect();
    assert_eq!(
        marked.len(),
        1,
        "the fixture must carry exactly ONE {marker} marker (found at lines {marked:?}); \
         more than one and this test would silently assert against the wrong site"
    );
    // The marker is a TRAILING comment on the call line itself — no offset to drift.
    let call_line = marked[0];

    let expected = SERVICE_LABEL_EDN
        .replace("{FILE}", fixture_path)
        .replace("{LINE}", &call_line.to_string());

    // STRUCTURE-exact, not string-exact: `assert_edn_eq!` parses both sides, so this asserts
    // the rendered label IS the declared value and stays honest across whitespace/field-order
    // changes in the writer.
    wat::assert_edn_eq!(argv_fields[1].clone(), &expected);
}

#[test]
fn unlabeled_process_argv_is_exe_only() {
    let argv_fields = drive_and_read_child_cmdline(UNLABELED_FIXTURE);
    assert_eq!(
        argv_fields.len(),
        1,
        "ProcessOpts/label defaulting to :None must leave argv as [exe] only, unchanged \
         from before the label field existed; got {argv_fields:?}"
    );
    assert_argv0_is_the_wat_binary(&argv_fields[0]);
}

/// ⛔ THE WALL. A forged label at a shell — no fd 3, so `was_spawned()` is false — must
/// accomplish NOTHING: it is read by the ORDINARY CLI parser as an entry-file path, the
/// exact same as any other garbage `argv[1]`, and fails to load. Proven by running the
/// SAME binary with (a) a string that LOOKS like a label and (b) an unrelated bogus
/// string, and showing they produce byte-identical outcomes — the label carries no
/// special meaning whatsoever outside `ExecPlan::build`'s own rendering.
#[test]
fn forged_label_at_shell_accomplishes_nothing() {
    let bin = env!("CARGO_BIN_EXE_wat");

    let forged_arg = FORGED_LABEL_EDN.trim_end();
    let forged = Command::new(bin)
        .arg(forged_arg)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat with a forged label argument");

    let ordinary_bogus = Command::new(bin)
        .arg("this-is-not-a-label-either")
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat with an ordinary bogus argument");

    // Neither fd 3 nor a --forms-server flag exists on either invocation, so BOTH must
    // fall through to the ordinary "load this as an entry-file path" branch and fail
    // identically — same exit code (EX_NOINPUT, 66), same error shape (a file-read
    // error naming the literal argv string), proving the label is parsed as nothing
    // more than a file path that doesn't exist.
    assert_eq!(
        forged.status.code(),
        Some(66),
        "a forged label with no fd 3 must fail as an ordinary missing-file usage error \
         (EX_NOINPUT); got status {:?}, stderr: {}",
        forged.status,
        String::from_utf8_lossy(&forged.stderr)
    );
    assert_eq!(
        forged.status.code(),
        ordinary_bogus.status.code(),
        "a forged label and an unrelated bogus argument must produce the IDENTICAL exit \
         code — any divergence would mean the label is being recognized as something \
         other than an ordinary (bogus) entry-file path"
    );

    // Byte-exact, not `.contains`: the WHOLE diagnostic must be the ordinary
    // read-the-entry-file error naming the forged string verbatim. A loose check here
    // would still pass if the label were ALSO recognized somewhere and emitted an extra
    // line — which is precisely the routing this wall exists to forbid.
    let forged_stderr = String::from_utf8_lossy(&forged.stderr);
    assert_eq!(
        forged_stderr,
        format!("wat: read {forged_arg}: No such file or directory (os error 2)\n"),
        "the forged label must be reported as a literal (nonexistent) file path, and \
         NOTHING else, proving nothing parsed or routed on its EDN shape"
    );

    // was_spawned() is fd-3-gated and nothing else — a forged label never sets fd 3, so
    // this is implied by the identical-exit-code assertion above, but state it directly:
    // no stdout was produced (a served/spawned runtime would have gone through the boot
    // handshake and behaved completely differently, not printed a usage/file error to stderr).
    assert!(
        forged.stdout.is_empty(),
        "a forged label must never reach a code path that writes anything to stdout; got: {:?}",
        String::from_utf8_lossy(&forged.stdout)
    );
}
