# BRIEF — can an io_uring write be raced?

Write one measurement-only probe that submits an `opcode::Write` which cannot complete, races it
against a second op, and reports what happens. **One new test file. No production code.**

## Read in order

1. **`DESIGN-can-an-io-uring-write-be-raced.md`** — the three outcomes and why each is informative.
2. **`FINDING-the-writes-kept-the-1970s.md`** — the defect this is downstream of, and the pipe-state
   recipe (fill completely, free a known amount, write far more than that room).
3. **`src/comms/process.rs:1295-1315`** — the existing `opcode::Read` submission: `types::Fd`,
   `.build().user_data(1)`, `ring.submission().push(&e)`, the SAFETY note on buffer lifetime.
   **This is the shape to copy.**
4. **`src/comms/process.rs`, the `opcode::PollAdd` site** — the second op you will race against.
   `user_data` is how you tell the two completions apart.
5. **`tests/comms/probe_arc278_partial_frame_residue.rs:1-45`** — a measurement-only probe's header
   discipline: state the mechanism, state what it cannot show, propose no fix.

## The work

**One probe under `tests/comms/`.** Suggested shape — adjust to what the substrate actually allows,
and **say what you adjusted**:

1. `pipe2(O_CLOEXEC)`. Fill the write end **completely** with `try_send`-style non-blocking writes
   until `EAGAIN`, counting bytes.
2. A second pipe (or an eventfd) standing in for the shutdown broadcast, **not yet readable**.
3. On a fresh ring: submit `opcode::Write` on the full pipe for a payload **larger than `PIPE_BUF`**
   (`user_data(1)`), and `opcode::PollAdd` on the broadcast fd (`user_data(2)`). Submit both.
4. Make the broadcast readable from the test thread after a short delay.
5. **Report, do not assert a conclusion:** which `user_data` completes first, with what result; then
   attempt `opcode::AsyncCancel` on the write and report whether it succeeds; then drain the ring and
   report what is left.

**Bound it.** The probe must terminate. A liveness bound with a diagnostic message — the residue
probe's `:284-297` is the exemplar, including *why* its 20 s is 1300× the measured typical.

## Blast radius

**One new file under `tests/comms/`.** No `src/` changes. No production code.

## STOP triggers

- **STOP-1** — if `opcode::Write` is not available in the pinned `io-uring` crate version, **STOP and
  report the version and what is available.** That fact alone re-shapes stone 3.
- **STOP-2** — **write no production code.** Not in `src/comms/`, not in `src/io.rs`, nowhere.
- **STOP-3** — **do not "fix" anything the probe reveals.** It buys one fact and stops.
- **STOP-4** — if the probe cannot be made to terminate, **STOP and report why.** An unbounded probe
  is worse than no probe; it will hang the floor for everyone.
- **STOP-5** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-6** — if the probe cannot construct a genuinely blocked write, **STOP and report what it
  did construct.** A probe that thinks it is measuring a blocked write and is not is the wrong-probe
  failure this arc has already paid for once.

## ⚠ On the box

Three tests already sit at **19–25 s against a 30 s terminate wall**. Adding a bounded probe to the
floor is fine; adding a slow one is not. **Keep its typical well under a second** and let the bound
be the only long path.
