# Stone 214.4.5-fix — `:process` spawn round-trip: preserve the comms pipe fds across the fork close-sweep

> Completes Stone 4.5 (`spawn-program' :process` tier). 4.5 landed the `:thread`
> tier green; the `:process` round-trip FAILED. This stone root-causes and fixes it.
> The disconfirming probe is already on disk: `tests/comms/spawn_program_prime_process.rs`
> (`#[ignore]`'d, both tests RED at HEAD `0c2b417b`).

## The symptom (grounded 2026-06-07, this session)

`setsid timeout 180 cargo test --release --test comms spawn_program_prime_process -- --ignored --test-threads=1`

→ **0 passed; 2 failed; finished in 0.03s** (fails FAST, NOT a deadlock):

- `spawn_program_prime_process_echo_round_trip` panics at `:101` — `peer.recv() must return echo result: RecvError`. The parent's send succeeds; the child never echoes back.
- `spawn_program_prime_process_sandbox_pure_fn_accepted` panics at `:182` — `recv must return doubled value: RecvError`.

Under the default (parallel) test threads, `echo_round_trip` instead panics at `:95` — `peer.send("42") must succeed: SendError("42")`. Same root cause; only *which side observes the broken pipe first* shifts with timing.

## Root cause — the child closes its own comms data-pipes

`spawn_process_peer` (`src/kernel/spawn.rs:396`) forks via `spawn_lifelined_any`. The child closure's first act is `crate::fork::child_post_fork_init(lifeline_r_raw)` (`spawn.rs:401`), which runs `close_inherited_fds_above_stdio(&[lifeline_r_raw])` (`fork.rs:524`). That sweep closes **every** inherited fd > 2 except the lifeline.

But the `:process` child needs two more fds: the comms **data**-pipes — `input_rx`'s read-end and `output_tx`'s write-end (the `comms::process::pair::<String>()` ends moved into the closure). They are neither stdio nor the lifeline, so the sweep closes them. The child's first `input_rx.recv()` then reads a dead fd → the `Err(RecvError) => _exit(0)` clean-EOF branch (`spawn.rs:415`); the parent's `output_rx.recv()` returns `RecvError`.

**Why the OLD stack didn't hit this:** the legacy `spawn_process`/`fork-program-ast` path (`fork.rs:616-655`, `966-1043`) uses **stdio** (fd 0/1/2) as its IPC channels (the stdout/stderr/exit-code triangle, recovery doc §13) and dups them onto 0/1/2 *before* the sweep — fd ≤ 2 survives. The new comms-peer model uses *dedicated* data-pipes that can't all land on 0/1/2, so it needs skip-list preservation the generic path never provided.

**Two compounding latent defects** (failure-engineering — eliminate the class, not just this site):
1. `close_inherited_fds_above_stdio` honors only `skip[0]` (`fork.rs:413`) — a silent single-fd limitation; `skip[1..]` is ignored. A multi-fd caller is silently betrayed. (Dark-class per `feedback_silent_swallow_is_dark_class`.)
2. `comms::process::Sender`/`Receiver` expose **no public raw-fd accessor** — preservation is impossible without reaching past the public API.

## The complete fd set (grounded against the PASSING 4.4 test)

The passing 4.4 test (`tests/comms/peer_process_round_trip.rs`) forks via
`spawn_lifelined` and its child **never calls `child_post_fork_init`** — so NO
close-sweep runs and *every* endpoint fd survives (COW copy). That is the empirical
proof the fix must reproduce: give the 4.5 child the SAME fd-survival.

Each endpoint's owned fds (read from `src/comms/process.rs`):
- **`Sender<T>`** — `write_fd` only (send is a plain `write`; no ring).
- **`Receiver<T>`** — `read_fd` **+ the io_uring `ring` fd** (`ring: RefCell<IoUring>`, `process.rs:265`; `IoUring: AsRawFd`). recv uses io_uring, so the ring fd must survive too — preserving only `read_fd` would still break `recv`.

So the child must preserve, across the sweep: `input_rx`'s {read_fd, ring fd} **and** `output_tx`'s {write_fd}.

## The fix (4 parts; both touched files are WARDED homes → re-ward)

1. **`src/comms/process.rs`** — add a public accessor returning the COMPLETE owned fd set of each endpoint (e.g. `pub fn raw_fds(&self) -> Vec<RawFd>`): `Sender` → `[write_fd]`; `Receiver` → `[read_fd, ring.borrow().as_raw_fd()]`. This is the portable, intentional surface for "preserve every fd this end owns across a fork." (intueri names it during the ward.)
2. **`src/fork.rs` — `close_inherited_fds_above_stdio`**: honor the FULL sorted skip-list via a multi-range sweep (sort + dedup `skip`, sweep the gaps between preserved fds). Annihilates latent defect #1 — `skip[1..]` can no longer be silently dropped.
3. **`src/fork.rs`** — add `child_post_fork_init_preserving(lifeline_r_raw: i32, extra_preserved: &[i32])` (the bare `child_post_fork_init` becomes `…_preserving(l, &[])`); thread `extra_preserved` into the close-sweep skip-list. Verify `init_shutdown_signal_with_inputs` does not re-clobber the preserved fds.
4. **`src/kernel/spawn.rs`** — the `:process` child collects `input_rx.raw_fds()` ∪ `output_tx.raw_fds()` and calls `child_post_fork_init_preserving(lifeline_r_raw, &preserved)` instead of the bare init.

## Secondary fix — the test binary forks from a multi-threaded parent

Cargo runs the two `#[test]` fns concurrently (2 threads), each `fork()`s. The test file's doc *claims* the binary is "single-threaded at startup" — false under parallel test threads. After part-1-4, single-threaded passes; the parallel case must be serialized. Options: (a) `integration-run.sh` already runs per-binary under setsid+timeout — add `--test-threads=1` for the comms process probes; (b) collapse the two probes into one `#[test]` that does both round-trips sequentially. Decide at strike time; (a) is the lighter touch and matches the existing envelope. This is fork-in-multithreaded-parent hygiene (recovery doc FM 7-ter neighbourhood), distinct from the core fd bug.

## Four questions

- **Obvious?** YES — a forked child that closes the very pipes it must read/write is a clear, named fd-lifecycle bug; the fix preserves exactly those two fds.
- **Simple?** YES — atomic parts: (1) accessor, (2) multi-fd sweep, (3) preserving init, (4) call it. Each independently verifiable.
- **Honest?** YES — fixes the class (multi-fd skip-list) not just the site; re-wards the two warded homes; names the secondary test-harness fix rather than hiding it.
- **Good UX?** YES — the comms `as_raw_fd` accessor is the honest, reusable preservation surface for every future fork-crossing peer (4.6+ wiring reuses it).

## Cadence

Probe already RED on disk (the two failing tests). → BRIEF + EXPECTATIONS (`model:"sonnet"`) → baseline re-run → spawn sonnet → SCORE vs own re-run (single-threaded GREEN + the multi-fd sweep verified) → re-ward `comms/` + `fork.rs` homes (drift-check the vigilatum stamps) → commit + push. THEN 4.6 polymorphic verbs.
