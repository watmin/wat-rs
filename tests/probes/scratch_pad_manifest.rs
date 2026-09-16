//! ⭐ THE MANIFEST RUNNER — named scratch-pad probes, executed on every floor.
//!
//! Excursus 001 `the-probes-run-in-the-floor`, builder-directed (*"manifest runner
//! then"*). Measured when the stone was drawn: **416 `.wat` under
//! `wat-scripts/scratch-pad/`, 382 of them PARSED-ONLY** — type-checked by
//! `every_wat_scripts_file_loads_on_the_current_runtime` and executed by nothing. Every
//! lifecycle invariant driven during this excursus lived in that 382: the handshake
//! `TimedOut` arm, the two-faults `GaveUp`, the named orphan, the survived handler
//! raise. Each was proven ONCE, by hand, on ONE box, and then went unguarded — R59
//! `NISI FRANGAS, NIHIL PROBAS` turned on our own work, and the reason a CI red once sat
//! for three days.
//!
//! # ⛔ AN EXPLICIT MANIFEST, NEVER A GLOB
//!
//! [`MANIFEST`] names its rows. It must NEVER become a directory walk: of the 382,
//! many park 60–120 s deliberately, some were REFUTED by later measurement, and some
//! are obsolete. A runner that executes what it finds would be slow, flaky, and would
//! resurrect dead probes as if they were live — the graveyard the scratch-pad
//! convention exists to prevent, with a test attached.
//!
//! **A row may only be added for a probe whose expected output was DRIVEN and recorded
//! in a SCORE.** A row written from reading the probe is a guess with a test around it.
//! Each row below carries its SCORE citation.
//!
//! # ⭑ A MARKER MUST BE UNIQUE TO THE PASSING WORLD
//!
//! `expect_markers` are substrings of the combined stdout+stderr, and each row must
//! name something no OTHER world prints. `expect_exit` alone is never enough: the
//! pre-stone world for `probe-two-faults-before-stop` exited non-zero with
//! `defservice stop: expected Status::Stopped`, and a row asserting only `exit=0` would
//! pass on a probe that had silently stopped testing anything. Every row's
//! `failing_world` field records, in code, what the world this row fences against
//! printed instead — so the fence is readable without opening a SCORE.
//!
//! # ⛔⛔ `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` IS PER-INVOCATION, NEVER SUITE-WIDE
//!
//! Measured by `the-handshake-deadline-is-injectable`: `wat/test.wat:335` and `:447`
//! name that deadline but wait for a spawned test child's **COMPLETION**, not its
//! readiness, so there the number is the child's whole runtime budget. **300 ms starves
//! that harness; 500 ms does not.** A runner that exported the var once for the suite
//! would redden every spawned test program in the corpus. It is therefore set on the
//! [`Command`] of the one row that needs it — and [`run_row`] also `env_remove`s it for
//! every row that does NOT set it, so an ambient value in the developer's shell cannot
//! silently change what a row means.
//!
//! # Mechanics, and why each is the way it is
//!
//! - **Subprocess, not in-process.** These are process-tier probes that spawn children.
//!   In-process (`startup_from_file`, as `probe_queue_*` do) can carry neither per-row
//!   env nor a kill-on-timeout.
//! - **The binary is `env!("CARGO_BIN_EXE_wat")`**, the `tests/cli/wat_cli.rs`
//!   precedent. ⚠ `WAT_RUNTIME_BIN` (which the DESIGN names as the precedent) is NOT
//!   needed and is deliberately not set: it exists because a cargo TEST binary's own
//!   image cannot serve as a spawned runtime, and here the thing being spawned is the
//!   real `wat` CLI, which holds its own image fd from CLI entry
//!   (`src/process/exec_plan.rs`, `ImageSource::HeldSelf`). Driven: the process-tier row
//!   forks a child successfully with the variable unset.
//! - **A timeout SIGKILLs.** A blocked `wat` ignores SIGTERM — 125 s measured — so
//!   [`Child::kill`] (SIGKILL on Unix) is the only bound that holds. A row that times
//!   out is a **FAIL with the whole captured output**, never a skip.
//! - **Capture whole, print whole on failure.** No `head`/`tail` window
//!   (`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`). Both pipes are
//!   drained by threads, so a probe that prints kilobytes of EDN (the dead-runner one
//!   prints ~4 KB) cannot deadlock on a full pipe buffer.
//! - **One nextest test per row**, never one test looping rows: a loop stops at the
//!   first failure and hides the rest
//!   (`[[feedback_the_first_failure_hides_the_rest]]`), and a per-row test names itself
//!   in the Summary. [`every_manifest_row_has_its_own_test`] is the fence that keeps it
//!   that way — add a row without a test and it goes red.
//! - **A missing path is a FAIL, not a skip** — otherwise a renamed probe silently
//!   stops being tested (trap-door 4).
//! - **No row asserts a count, a rate, or an ELAPSED TIME.** `probe_chaos_gate_has_teeth`'s
//!   own header is the precedent (*"do not assert `fires > 0` at 200 bp"*, *"do not assert
//!   a draw count"*). ⛔ Two candidate markers were REMOVED under this rule and both went
//!   red first, which is why they are written down rather than merely forbidden:
//!   `runner 0` (dead-runner row) flipped to `runner 1` under nextest's own parallelism —
//!   6 of 12 concurrent runs — and `waited-ms=0` (two-faults row) became `waited-ms=1`
//!   under a full floor. Uncontended, both were stable across every hand-run. **A value
//!   the substrate computes is a fine marker; a threshold on a clock or on a scheduler is
//!   not** — and the uncontended stability is what makes such a marker a trap rather than
//!   an obvious mistake.
//!
//! Run just this group: `cargo nextest run --release -E 'binary_id(wat::probes)'`

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// One manifest row: a record, not a positional tuple (the house form — `#fanout/Input`
/// is the precedent). `name` is also the nextest test name, which is what
/// [`every_manifest_row_has_its_own_test`] keys on.
struct Row {
    /// The row key AND the name of the `#[test]` fn that runs it.
    name: &'static str,
    /// Repo-relative path to the probe. Must exist, or the row FAILS (trap-door 4).
    path: &'static str,
    /// Extra argv words after the program path (`argv[2..]`).
    args: &'static [&'static str],
    /// Expected process exit code. A probe whose invariant is a RAISE exits 2.
    expect_exit: i32,
    /// Substrings that must all appear in stdout+stderr. Each must be unique to the
    /// passing world — see `failing_world`.
    expect_markers: &'static [&'static str],
    /// What the world this row fences against printed INSTEAD. Documentation that lives
    /// next to the assertion rather than in a SCORE nobody opens. Not asserted: a
    /// negative-world string is not reproducible from this tree.
    failing_world: &'static str,
    /// SIGKILL deadline for the whole invocation.
    timeout_ms: u64,
    /// Per-invocation env. ⛔ Never exported suite-wide — see the module header.
    env: &'static [(&'static str, &'static str)],
}

/// ⛔ EXPLICIT. Never a glob. Every row cites the SCORE that DROVE its expectation.
const MANIFEST: &[Row] = &[
    // ── excursus 001 `every-status-arm-is-named`, SCORE "⭐ Row 9 — DRIVEN, on all
    // four sites, with a negative control". Driven line, verbatim, one of four:
    //   "stop     =GaveUp waited-ms=0 last=a second Status::Faulted arrived — the
    //    fault stream outran the one-level drain"
    // A second `Status::Faulted` queued ahead of the owner's ack is FACED, not fatal,
    // at all four owner surfaces.
    //
    // ⛔ `waited-ms=` IS NOT IN THE MARKERS, AND THE FLOOR IS WHY. This row was first
    // written with the SCORE's line up to `…=GaveUp waited-ms=0 last=…` and it went RED
    // on its first FULL floor (`.floor/2026-09-16T05-43-33Z/`, `Summary [ 557.580s] …
    // 1 failed`) with:
    //     MISSING  "\"stop     =GaveUp waited-ms=0 last=a second Status::Faulted arrived"
    //   and the probe's own stdout in the same block:
    //     "stop     =GaveUp waited-ms=1 last=a second Status::Faulted arrived — …"
    // ⭑ ONE MILLISECOND. `waited-ms` is an integer ms read from the method's own `t0`, so
    // `waited-ms=0` asserts that two recv round-trips finish inside 1 ms on a box running
    // 5257 tests — a threshold on a clock, which is trap-door 1's "assert the invariant,
    // never the statistic" (and trap-door 2 was right that `waited-ms=0` is a real value;
    // it is just not an invariant). Uncontended it is 0 every time, which is exactly what
    // makes it a trap. The invariant is the four `GaveUp`s and the `last=` MECHANISM.
    Row {
        name: "two_faults_before_stop_are_faced_at_all_four_surfaces",
        path: "wat-scripts/scratch-pad/probe-two-faults-before-stop.wat",
        args: &[],
        expect_exit: 0,
        expect_markers: &[
            // The non-vacuity control: the service answered a normal request first.
            "\"control=Message\"",
            // All four owner surfaces face it. No elapsed time in these, on purpose.
            "\"stop     =GaveUp",
            "\"hibernate=GaveUp",
            "\"grant    =GaveUp",
            "\"revoke   =GaveUp",
            // …and the MECHANISM, which is what makes those four GaveUps THIS stone's and
            // not some other world's: a GaveUp for any other reason fails this row.
            "last=a second Status::Faulted arrived — the fault stream outran the one-level drain",
        ],
        failing_world: "pre-stone (HEAD's wat/service.wat, rebuilt and driven as that \
                        SCORE's negative control) the owner DIED on a recoverable error: \
                        `AssertionFailure … \"defservice stop: expected Status::Stopped\"`, \
                        with the word `Faulted` nowhere in it. It printed no GaveUp line \
                        at all, and exited non-zero.",
        timeout_ms: 20_000,
        env: &[],
    },
    // ── excursus 001 `the-handshake-deadline-is-injectable`, SCORE "⭐ Rows 2 and 3 —
    // DRIVEN, four wall-clocks". Driven, thread tier, knob at 200 ms:
    //   +000.657 AssertionFailure "recv: timed out — the peer is alive and silent"
    //            frame :wat::spawn::ThreadOpts/launch            Δ = 0.201 s
    // The launcher's wait is BOUNDED: a child that neither crashes nor exits nor
    // announces readiness is reported, not waited on forever.
    //
    // ⛔ The 200 ms is the whole reason this row is affordable (30 s per tier before
    // stone 1). It is set HERE, on this row's Command — never for the suite.
    Row {
        name: "a_silent_child_cannot_hang_launch_on_the_thread_tier",
        path: "wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat",
        args: &[],
        expect_exit: 2,
        expect_markers: &[
            // The probe echoes the tier it drove, so the log says which arm this is…
            "\"sht: tier=thread\"",
            // …the arm itself…
            "recv: timed out — the peer is alive and silent",
            // …and the FRAME, which is what separates this row from its process twin:
            // ThreadOpts and ProcessOpts are separate extend-type impls, each with its
            // own bounded wait.
            ":wat::spawn::ThreadOpts/launch",
        ],
        failing_world: "the pre-stone launcher awaited readiness on a BARE recv, so this \
                        probe HUNG — no arm, no exit, and unkillable by SIGTERM (125 s \
                        ignored, measured). The row's timeout below is what turns that \
                        world into a FAIL instead of a wedged floor.",
        timeout_ms: 20_000,
        env: &[("WAT_STARTUP_HANDSHAKE_DEADLINE_MS", "200")],
    },
    // ── same SCORE, process tier, knob at 200 ms:
    //   +000.669 AssertionFailure "recv: timed out — the peer is alive and silent"
    //            frame :wat::spawn::ProcessOpts/launch           Δ = 0.207 s
    // ⭑ The tier is an ARGV WORD, not a source edit. The probe's `:locus` used to be a
    // literal flipped by hand between runs; it now reads `argv[2]`, so one file drives
    // both tiers and neither row hand-edits the thing under test.
    Row {
        name: "a_silent_child_cannot_hang_launch_on_the_process_tier",
        path: "wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat",
        args: &["process"],
        expect_exit: 2,
        expect_markers: &[
            "\"sht: tier=process\"",
            "recv: timed out — the peer is alive and silent",
            ":wat::spawn::ProcessOpts/launch",
        ],
        failing_world: "as the thread row: an unbounded wait, i.e. a HANG with no arm. \
                        And had the tier word not reached the probe, the frame here would \
                        read ThreadOpts/launch — which is why the frame is a marker.",
        timeout_ms: 20_000,
        env: &[("WAT_STARTUP_HANDSHAKE_DEADLINE_MS", "200")],
    },
    // ── excursus 001 `a-dead-runner-names-its-orphan`, SCORE "⭑⭑ THE HEADLINE — item
    // 3, still a raise" (five runs, every one):
    //   clean=[0 2 4 6 8 10 12 14]
    //   subject-starting
    //   exit=2
    //   bracket collect-loop: runner 0 crashed holding item 3: …
    // `brackets/map` still FAILS on a dead runner — it stops failing anonymously.
    //
    // ⛔ THE RUNNER INDEX IS NOT IN THE MARKER, AND THAT IS A CORRECTION TO THAT SCORE.
    // Its "five runs, every one" printed `runner 0`; this row was first written with
    // `runner 0 crashed holding item 3:` copied from it, and it went RED on its very
    // first nextest run with `runner 1`. Driven afterwards: 10 sequential runs → 10×
    // `runner 0`; 12 CONCURRENT runs → 6× `runner 0`, 6× `runner 1`. So the index is a
    // scheduling statistic that the floor's own contention perturbs (trap-door 1 — do
    // not assert a statistic), while the ITEM was 3 in all 22 observations. The item is
    // the stone's invariant; the runner position is which worker happened to take it.
    // ⭑ Had the SCORE's line been trusted as written, this row would have been an
    // intermittent floor red — the exact shape the manifest exists to avoid creating.
    Row {
        name: "a_dead_runner_names_the_item_it_orphaned",
        path: "wat-scripts/scratch-pad/probe-a-dead-runner-orphans-its-item.wat",
        args: &[],
        expect_exit: 2,
        expect_markers: &[
            // Non-vacuity: a clean map still returns, and returns the right vector.
            "\"clean=[0 2 4 6 8 10 12 14]\"",
            // The ledger names WHICH item died with the runner. Item 3, never 0,
            // never "idle (holding no item)". The runner index is deliberately NOT
            // here — see the note above.
            "crashed holding item 3:",
        ],
        failing_world: "`bracket collect-loop: runner 0 crashed: …` — the same raise, the \
                        same exit 2, with no item named (quoted as \"Today:\" in that \
                        SCORE). A row asserting only the raise or the exit code would \
                        pass on that anonymous world, which is the whole defect the stone \
                        removed.",
        timeout_ms: 20_000,
        env: &[],
    },
    // ── excursus 001 `a-service-stops-instead-of-crashing`, SCORE "⭑ THE HEADLINE —
    // the innocent client is served":
    //   a-control=Ok
    //   a-boom   =Lost
    //   b-after  =Ok
    // A raise escaping an op handler is a graceful stop for the raiser's connection
    // only: the service keeps serving, and a client that dials AFTER the raise is
    // answered.
    Row {
        name: "a_handler_raise_does_not_kill_the_service_for_everyone",
        path: "wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat",
        args: &[],
        expect_exit: 0,
        expect_markers: &[
            // Non-vacuity control — the service answers before anything raises.
            "\"a-control=Ok\"",
            // The raiser's own conn is severed. This is expected, not the defect.
            "\"a-boom   =Lost\"",
            // ⭑ THE ROW: an innocent second client, dialling after the raise, is served.
            "\"b-after  =Ok\"",
        ],
        failing_world: "`b-dial=connect-REFUSED` — the dead service could not even be \
                        dialled, so `b-after  =Ok` was never printed (that exact line is \
                        the failure this probe's own v2 recorded in the SCORE; the label \
                        `b-dial` is printed by the probe's own connect-failure arm, \
                        which is still in the file).",
        timeout_ms: 20_000,
        env: &[],
    },
];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Look a row up by name. Panics loudly if the `#[test]` fn and the manifest have
/// drifted apart — a test pointing at a row that no longer exists must not pass.
fn row(name: &str) -> &'static Row {
    MANIFEST.iter().find(|r| r.name == name).unwrap_or_else(|| {
        panic!(
            "manifest row {name:?} does not exist. A #[test] fn and MANIFEST have drifted; \
             the test must not silently become a no-op."
        )
    })
}

/// Drain one pipe into `sink` on its own thread, setting `done` at EOF. Draining in a
/// thread is not decoration: a probe that prints more than a pipe buffer (64 KB) would
/// otherwise block on write while we block on wait.
fn drain<R: Read + Send + 'static>(
    mut reader: R,
    sink: Arc<Mutex<Vec<u8>>>,
    done: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => sink.lock().expect("sink poisoned").extend_from_slice(&buf[..n]),
            }
        }
        done.store(true, Ordering::SeqCst);
    });
}

/// What happened to the process.
enum Ended {
    /// Exited on its own with this code (`None` = killed by a signal).
    Exit(Option<i32>),
    /// We SIGKILLed it at the row's deadline, after this long.
    KilledAtTimeout(Duration),
}

/// The whole run of one row: spawn, capture both pipes, bound with SIGKILL, assert.
///
/// Every failure path panics with the WHOLE captured output — never a window.
fn run_row(r: &Row) {
    let probe: PathBuf = repo_root().join(r.path);

    // Trap-door 4: a renamed probe must go RED, not quietly stop being tested.
    assert!(
        probe.is_file(),
        "MANIFEST row {name:?}: probe not found at {p}\n\
         A manifest row whose path does not exist is a FAILURE, not a skip — a renamed \
         or deleted probe must not silently stop being tested. Fix the path in MANIFEST \
         (tests/probes/scratch_pad_manifest.rs) or remove the row deliberately.",
        name = r.name,
        p = probe.display(),
    );

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wat"));
    cmd.arg(r.path) // relative, with cwd = repo root: the probe's own reported
        .current_dir(repo_root()) // locations then match a hand-run byte for byte.
        .args(r.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // ⛔⛔ Per-invocation env ONLY. And clear the handshake deadline for rows that do
    // not set it, so an ambient export in the developer's shell cannot change what a
    // row means (300 ms starves wat/test.wat's harness; 500 ms does not).
    cmd.env_remove("WAT_STARTUP_HANDSHAKE_DEADLINE_MS");
    for (k, v) in r.env {
        cmd.env(k, v);
    }

    let mut child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("MANIFEST row {}: could not spawn wat: {e}", r.name));

    let out_buf = Arc::new(Mutex::new(Vec::new()));
    let err_buf = Arc::new(Mutex::new(Vec::new()));
    let out_done = Arc::new(AtomicBool::new(false));
    let err_done = Arc::new(AtomicBool::new(false));
    drain(
        child.stdout.take().expect("stdout piped"),
        Arc::clone(&out_buf),
        Arc::clone(&out_done),
    );
    drain(
        child.stderr.take().expect("stderr piped"),
        Arc::clone(&err_buf),
        Arc::clone(&err_done),
    );

    let deadline = Duration::from_millis(r.timeout_ms);
    let started = Instant::now();
    let ended = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ended::Exit(status.code()),
            Ok(None) => {}
            Err(e) => panic!("MANIFEST row {}: try_wait failed: {e}", r.name),
        }
        let waited = started.elapsed();
        if waited >= deadline {
            // ⛔ SIGKILL, not SIGTERM: a blocked `wat` ignores SIGTERM (125 s measured).
            let _ = child.kill();
            let _ = child.wait();
            break Ended::KilledAtTimeout(waited);
        }
        std::thread::sleep(Duration::from_millis(20));
    };

    // Give the readers a bounded grace to reach EOF. Bounded, not a join: on the
    // process tier a forked grandchild can outlive its parent holding the same pipe,
    // and an unbounded join there would turn a fast FAIL into a hang.
    let grace = Instant::now();
    while (!out_done.load(Ordering::SeqCst) || !err_done.load(Ordering::SeqCst))
        && grace.elapsed() < Duration::from_millis(2_000)
    {
        std::thread::sleep(Duration::from_millis(20));
    }

    let stdout = String::from_utf8_lossy(&out_buf.lock().expect("stdout sink").clone()).into_owned();
    let stderr = String::from_utf8_lossy(&err_buf.lock().expect("stderr sink").clone()).into_owned();
    let combined = format!("{stdout}{stderr}");

    // One place builds the failure report, so no arm can accidentally window it.
    let report = |reason: String| -> String {
        let env: Vec<String> = r
            .env
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!(
            "\n⛔ MANIFEST ROW FAILED — {name}\n\
             {reason}\n\
             \n\
             row      {name}\n\
             probe    {path}\n\
             argv     [{args}]\n\
             env      [{env}]  (WAT_STARTUP_HANDSHAKE_DEADLINE_MS is cleared unless listed)\n\
             timeout  {timeout} ms (SIGKILL)\n\
             expected exit {exit}\n\
             markers  {markers:?}\n\
             \n\
             What the world this row fences against printed instead:\n  {failing}\n\
             \n\
             ── the WHOLE captured output follows, both pipes, untruncated ──\n\
             ── stdout ({olen} bytes) ──\n{stdout}\n\
             ── stderr ({elen} bytes) ──\n{stderr}\n\
             ── end of captured output ──\n",
            name = r.name,
            reason = reason,
            path = r.path,
            args = r.args.join(" "),
            env = env.join(" "),
            timeout = r.timeout_ms,
            exit = r.expect_exit,
            markers = r.expect_markers,
            failing = r.failing_world,
            olen = stdout.len(),
            elen = stderr.len(),
            stdout = stdout,
            stderr = stderr,
        )
    };

    match ended {
        Ended::KilledAtTimeout(waited) => panic!(
            "{}",
            report(format!(
                "TIMED OUT after {:.3} s and was SIGKILLed. A timeout is a FAIL, never a \
                 skip — and whatever the probe had printed before the kill is below, which \
                 is the only evidence this run will ever produce.",
                waited.as_secs_f64()
            ))
        ),
        Ended::Exit(None) => panic!(
            "{}",
            report(
                "DIED ON A SIGNAL (no exit code). It was not us — our own kill path reports \
                 as a timeout. A signal death is a real failure."
                    .to_string()
            )
        ),
        Ended::Exit(Some(code)) => {
            if code != r.expect_exit {
                panic!(
                    "{}",
                    report(format!(
                        "EXITED {code}, expected {}. The exit code alone never proves a row \
                         (see the markers), but a changed one means the probe no longer does \
                         what the manifest says it does.",
                        r.expect_exit
                    ))
                );
            }
            let missing: Vec<&str> = r
                .expect_markers
                .iter()
                .copied()
                .filter(|m| !combined.contains(m))
                .collect();
            if !missing.is_empty() {
                panic!(
                    "{}",
                    report(format!(
                        "exit {code} was as expected, but {} of {} MARKERS ARE ABSENT from \
                         stdout+stderr:\n{}\n\
                         An exit code is not an invariant. Each marker names something only \
                         the passing world prints.",
                        missing.len(),
                        r.expect_markers.len(),
                        missing
                            .iter()
                            .map(|m| format!("           MISSING  {m:?}"))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    ))
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ONE TEST PER ROW. Never a loop over MANIFEST: a loop stops at the first failure
// and hides the rest, and a per-row test names itself in nextest's Summary.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn two_faults_before_stop_are_faced_at_all_four_surfaces() {
    run_row(row("two_faults_before_stop_are_faced_at_all_four_surfaces"));
}

#[test]
fn a_silent_child_cannot_hang_launch_on_the_thread_tier() {
    run_row(row("a_silent_child_cannot_hang_launch_on_the_thread_tier"));
}

#[test]
fn a_silent_child_cannot_hang_launch_on_the_process_tier() {
    run_row(row("a_silent_child_cannot_hang_launch_on_the_process_tier"));
}

#[test]
fn a_dead_runner_names_the_item_it_orphaned() {
    run_row(row("a_dead_runner_names_the_item_it_orphaned"));
}

#[test]
fn a_handler_raise_does_not_kill_the_service_for_everyone() {
    run_row(row("a_handler_raise_does_not_kill_the_service_for_everyone"));
}

/// ⭑ THE FENCE ON THE RUNNER ITSELF — a row with no test of its own is not tested.
///
/// The failure this prevents is specific and silent: append a `Row` to [`MANIFEST`],
/// forget the `#[test]`, and the manifest reads as if the probe were guarded while
/// nothing runs it. Keyed on the property (a `fn <name>(` exists in this file), not on
/// a count, so it cannot be satisfied by renumbering.
///
/// It reads its own source with `include_str!`, which is the same file this test is
/// written in — the only instrument that can see the `#[test]` fns from inside.
#[test]
fn every_manifest_row_has_its_own_test() {
    let src = include_str!("scratch_pad_manifest.rs");
    assert!(!MANIFEST.is_empty(), "the manifest is empty");

    let mut orphans = Vec::new();
    for r in MANIFEST {
        if !src.contains(&format!("fn {}(", r.name)) {
            orphans.push(r.name);
        }
    }
    assert!(
        orphans.is_empty(),
        "these MANIFEST rows have no `#[test] fn` of their own, so NOTHING RUNS THEM:\n{}\n\
         Add one `#[test] fn <row name>() {{ run_row(row(\"<row name>\")); }}` per row. \
         One test per row is deliberate: a loop over rows stops at the first failure and \
         hides every later one.",
        orphans
            .iter()
            .map(|n| format!("  {n}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    // A row whose test exists but whose PROBE does not know it is in the floor is the
    // other half of the same hazard — see `every_manifest_probe_says_it_is_in_the_floor`.

    // Duplicate names would make `row()` return the first and leave the second dead.
    let mut names: Vec<&str> = MANIFEST.iter().map(|r| r.name).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(
        before,
        names.len(),
        "duplicate row names in MANIFEST — `row()` would return the first and the other \
         would never run"
    );
}

/// ⭑ THE OTHER HALF OF THE FENCE — a probe in the floor must SAY it is in the floor.
///
/// The hazard is specific: these four `.wat` files read as scratch probes, because that is
/// what they were until this stone. Someone edits one by hand (relabels a println, tunes a
/// number), the floor goes red somewhere they were not looking, and the connection between
/// the two is a path in a panic message they have to go and read. The annotation is what
/// they see FIRST, in the file they are already editing.
///
/// Gated on the **property** — the probe's own text names the runner that runs it — rather
/// than on a hand-maintained list, so an annotation cannot quietly become decoration and a
/// NEW row cannot ship without one. It reads only the header region: the note belongs where
/// an editor lands, not buried at the bottom.
#[test]
fn every_manifest_probe_says_it_is_in_the_floor() {
    /// The runner's own path, which is what a probe must point back at.
    const RUNNER: &str = "tests/probes/scratch_pad_manifest.rs";
    /// How far into the file the note must appear. Generous, but bounded: a note below
    /// this is not where someone editing the file will see it.
    const HEADER_LINES: usize = 60;

    let mut silent = Vec::new();
    for r in MANIFEST {
        let probe = repo_root().join(r.path);
        let text = std::fs::read_to_string(&probe)
            .unwrap_or_else(|e| panic!("row {}: cannot read {}: {e}", r.name, probe.display()));
        let header: String = text.lines().take(HEADER_LINES).collect::<Vec<_>>().join("\n");
        if !header.contains(RUNNER) {
            silent.push(r.path);
        }
    }
    silent.sort_unstable();
    silent.dedup();
    assert!(
        silent.is_empty(),
        "these probes are RUN BY THE FLOOR but their first {HEADER_LINES} lines do not say \
         so:\n{}\n\
         Add a note naming `{RUNNER}` and the row, so the next person to edit the probe \
         learns from the FILE that a change there can redden the floor — not from a panic \
         message after the fact.",
        silent
            .iter()
            .map(|p| format!("  {p}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}
