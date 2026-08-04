# BRIEF — the shutdown cohort moves into children

**Floor:** `4344 tests run: 4344 passed, 262 skipped` (my own `--release` re-run at `d217d042`).
**Board:** closes #69, and adds the fourth file #69 did not know about.

## Why

Four test files mutate **process-global shutdown state** and are safe only because
`.config/nextest.toml` forks a process per test. Under plain `cargo test` — one shared process —
three of them fire a **process-wide signal** that hits every other test in the binary, and the
fourth flips `KERNEL_STOPPED` under everyone's feet.

```
tests/channel/probe_arc170_writer_joins_lockstep.rs    libc::raise(SIGTERM)
tests/process/shutdown_cascade_memory.rs               libc::raise(SIGTERM)
tests/process/shutdown_cascade_pipefd.rs               libc::raise(SIGTERM)
tests/comms/probe_arc278_send_poll_arm.rs              request_kernel_stop()   ← landed today
```

**"The runner isolates us" is a convention, not a wall.** The fourth file's own header states it as
a guarantee — *"cannot leak into any other test's process"* — which is true under nextest and false
otherwise. That file is mine, from this afternoon; it is in this sweep on the same terms as the
other three, not exempted for being new.

`libc::raise(sig)` is `kill(getpid(), sig)`: **the measurer and the measured are one process.** The
cure is uniform — do the dangerous thing in a **child**, signal the child, read its report. A child
that dies takes its polluted globals with it.

## The two shapes

Which shape a file gets is decided by whether its subject is reachable from a wat program.

### Shape A — a child `wat` program (2 files)

`shutdown_cascade_memory.rs` (a blocked `typed_recv` on a thread-tier channel) and
`shutdown_cascade_pipefd.rs` (the same on a PipeFd-tier receiver).

**A shutdown wake IS wat-visible**: `recv_outcome_shutdown()` (`src/runtime.rs`) produces
`RecvOutcome::Lost[cause = LociDiedError::Stopped]` — distinct from `Closed` and from any other
`Lost`. So a wat child can *observe and report* the exact thing these tests assert.

**Exemplar, already green:** `tests/cli/wat_cli.rs:777` `sigterm_reaches_a_program_blocked_on_stdin`
— spawn `CARGO_BIN_EXE_wat` on a program that prints `READY` and parks, `libc::kill(child.id(),
SIGTERM)`, then a **bounded** `try_wait` poll (never a raw `wait()`), and assert on exit status.
Copy its structure.

> **⛔ PROVE THE CHILD CAN SEE IT BEFORE YOU REBUILD ANYTHING.** Write a throwaway wat program that
> parks on a `recv` from a thread peer, print what the outcome actually is when the process is
> SIGTERM'd, and confirm it is `Lost[Stopped]` — and that the child still exits cleanly enough to
> report. If it is `Closed`, or the child dies before printing, **STOP and report**: Shape A does
> not hold for that tier and the file needs Shape B instead. Do not rebuild on an assumption.
> Match the tier: a **thread** peer for the memory file, a **process** peer for the pipefd file.

### Shape B — re-exec the test binary (2 files)

`probe_arc170_writer_joins_lockstep.rs` (blocked `PipeWriter::write`) and
`probe_arc278_send_poll_arm.rs` (blocked `comms::process::Sender::send`). Both subjects are
Rust-internal with no wat door, so the child must be Rust.

Use `std::env::current_exe()` + an env marker. The child branch does the fill-and-block, reports on
stdout, and exits; the parent spawns it, waits for its `READY` line, signals it, and reads the
report.

- **No new `[[bin]]`.** `wat` and `cargo-wat` are the product; a probe binary in the manifest ships
  a test artifact. If you believe re-exec cannot work, **STOP and say why** rather than adding one.
- The child branch must be unmistakable — a named function, guarded on the env var at the top of
  the test, with a comment saying it is a child role. It must not read as dead code.

## Also in this sweep — delete the `<100ms` bound

Three sites, not the two an earlier note claimed:

```
tests/process/shutdown_cascade_memory.rs:139
tests/process/shutdown_cascade_pipefd.rs:146
tests/channel/probe_arc170_writer_joins_lockstep.rs:166
```

Nothing derives 100. It asserts **performance** while the subject is **correctness**, it is
load-sensitive, and "did it hang" is already answered by nextest's slow-timeout (15s warn / 30s
kill).

> **⛔ KEEP `probe_arc170_writer_joins_lockstep.rs:139`'s `recv_timeout(3s)`.** That is not a perf
> assertion — it is the RED-gate mechanism that makes the probe *report* instead of wedge while the
> defect is live. Deleting it would reintroduce the hang this whole cohort is about.

## STOPs

- **⛔ No `libc::raise` anywhere when you are done.** `grep -rn 'libc::raise' tests/` must return
  zero. That is the sweep's headline gate.
- **⛔ No raw `.join()` or unbounded `wait()`** on anything that may not return. Every wait bounded.
- **⛔ Do not weaken an assertion to make a rebuild pass.** If the child cannot observe what the
  in-process test observed, that is a finding — report it; do not assert something smaller.
- **⛔ Do not delete the `recv_timeout(3s)`** (above).
- **⛔ If Shape A's disconfirming probe fails, STOP.** Report which tier and what you actually saw.

## ★ THE DELIBERATE BREAK — one per rebuilt file

For each of the four, break the mechanism it names, confirm the rebuilt test goes **RED**, restore
byte-exact, confirm green. Report all four with real output.

**If any file does not redden, say so plainly.** A rebuilt test that passes whether or not the
mechanism works is worse than the original — it looks like progress and proves nothing.

## Done means

- `grep -rn 'libc::raise' tests/` → zero.
- The one remaining `request_kernel_stop()` lives in a child branch, not a test's main thread.
- All four tests assert what they asserted before, from a child.
- The three `<100ms` assertions gone; the `recv_timeout(3s)` kept.
- Four deliberate breaks, four RED outputs, four restores — all with real output.
- `cargo nextest run --release` **Summary verbatim** against `4344/4344/0/262`, arithmetic
  explained (the count may change if a rebuild merges or splits cases — explain any delta).
- `cargo clippy --release --all-targets` clean.
- Every STOP hit; if none, say so.

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every verification in the FOREGROUND and block on it. Do not commit,
push, or stash. If a verification is incomplete when you finish, say so plainly.

This sweep is long. If you run short, **finish fewer files completely** rather than all four
partially — a half-migrated cohort is worse than an un-migrated one, because it looks done.
