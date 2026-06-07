# BRIEF — Stone 214.4.5-fix: preserve the `:process` child's comms fds across the fork close-sweep

You are completing the `:process` tier of `spawn-program'`. The `:thread` tier is
green; the `:process` round-trip currently fails because the forked child closes
the very pipes it needs. This brief fixes that. Full design + grounding:
`docs/arc/2026/05/214-concurrency-toolkit/DESIGN-STONE-4.5-FIX-PROCESS-FD-PRESERVATION.md`.

## The mechanism (already root-caused)

`spawn_process_peer` (`src/kernel/spawn.rs`) forks; the child's first act is
`crate::fork::child_post_fork_init(lifeline_r_raw)`, whose
`close_inherited_fds_above_stdio(&[lifeline_r_raw])` closes every fd > 2 except the
lifeline — including the child's comms pipe + io_uring ring fds. The child's
`recv` then hits a dead fd. The fix preserves those fds across the sweep.

The PASSING reference is `tests/comms/peer_process_round_trip.rs` — its child runs
NO close-sweep, so all endpoint fds survive and the round-trip works. Reproduce
that fd-survival, this time *through* `child_post_fork_init`.

## The work (4 parts + re-ward)

**Part 1 — `src/comms/process.rs`: expose each endpoint's complete owned fd set.**
- `Sender<T>` owns `write_fd`. `Receiver<T>` owns `read_fd` AND `ring: RefCell<IoUring>` (`IoUring: AsRawFd`).
- Add a public accessor on each returning every raw fd it owns:
  - `Sender::raw_fds(&self) -> Vec<std::os::fd::RawFd>` → `vec![self.write_fd.as_raw_fd()]`
  - `Receiver::raw_fds(&self) -> Vec<std::os::fd::RawFd>` → `vec![self.read_fd.as_raw_fd(), self.ring.borrow().as_raw_fd()]`
- Both `Receiver`'s `read_fd` and `ring` fd matter: `recv` uses io_uring, so the ring fd must be preserved too.

**Part 2 — `src/fork.rs`: `close_inherited_fds_above_stdio` honors the FULL skip-list.**
Today it preserves only `skip[0]` (`fork.rs:413`). Make it preserve every fd in `skip`: sort + dedup the kept fds, then sweep each gap between them (and `[3, first-1]` and `[last+1, MAX]`). Every fd in `skip` survives; everything else > 2 closes. This is a fork child (single-threaded), so the range sweep is race-free as documented.

**Part 3 — `src/fork.rs`: add `child_post_fork_init_preserving(lifeline_r_raw: i32, extra_preserved: &[i32])`.**
Same body as `child_post_fork_init` (silent panic hook → setpgid → close-sweep → shutdown signal → signal handlers), but the close-sweep's skip-list is `[lifeline_r_raw] ∪ extra_preserved`. Re-express the existing `child_post_fork_init` as `child_post_fork_init_preserving(l, &[])` so there is one implementation. Confirm `init_shutdown_signal_with_inputs` (step 4) and the signal-handler install (step 5) do not reopen onto a preserved fd number.

**Part 4 — `src/kernel/spawn.rs`: the `:process` child preserves its comms fds.**
In the child closure, before/at the init call, collect `input_rx.raw_fds()` and `output_tx.raw_fds()` into one `Vec<i32>` and call `child_post_fork_init_preserving(lifeline_r_raw, &preserved)` instead of the bare `child_post_fork_init`. (The fds are valid in the child — clone3 copies the parent fd table, same numbers.)

**Part 5 — serialize the two `:process` probes.**
`tests/comms/spawn_program_prime_process.rs` has two `#[test]` fns that each fork; cargo runs them on parallel threads, so they fork from a multi-threaded parent. Make them run serially: in `scripts/integration-run.sh`, run the comms binary's process-tier probes with `--test-threads=1` (or, if cleaner, collapse the two probes into one `#[test]` that does both round-trips sequentially — your call, whichever reads better). Keep the `#[ignore]` attributes; they run via `integration-run.sh` / `--ignored`.

**Re-ward.** `src/comms/process.rs` and `src/fork.rs` are warded homes carrying `vigilatum` stamps. After the change, drift-check both: update the module docs to reflect the new accessor + multi-fd sweep, and confirm the stamp still holds (clippy clean in-home is part of the stamp).

## Verification (run these; report exact numbers)

- `cargo build --release` — clean.
- `cargo clippy --release` — 0 warnings in the touched files.
- Library suite (lib-safe): `cargo test --release --lib -p wat` — expect the green band (~940/0/1).
- The fixed probes, SINGLE-THREADED:
  `setsid timeout 180 cargo test --release --test comms spawn_program_prime_process -- --ignored --test-threads=1`
  → expect **2 passed; 0 failed**.
- The 4.4 reference still green:
  `setsid timeout 120 cargo test --release --test comms peer_process_round_trip -- --ignored`
  → **1 passed**.
- Report the before/after of `grep -n "skip\[0\]" src/fork.rs` (should be gone) and the new accessor signatures.

Do NOT commit — leave the tree dirty for the orchestrator to score, re-ward, and commit. Report what you changed, the exact test output, and any surprise honestly.
