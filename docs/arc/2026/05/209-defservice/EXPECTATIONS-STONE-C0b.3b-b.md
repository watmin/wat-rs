# EXPECTATIONS — Stone C0b.3b-b (written before the strike)

Independent scorecard. The Inquisitor verifies every row by its own re-run before any commit.
The disconfirm is the probe set (RED→GREEN); the regression rows guard the untouched surface.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | The stranger is BOUNCED (the gate is live) | `cargo test --release -p wat --test probe_arc209_c0b3bb_bounced -- --test-threads=1` | `2 passed` (`stranger_is_bounced` flips RED→GREEN; `owner_served_via_birth_seed` stays GREEN) |
| 2 | `allow'`/`deny'` exist + tier-gate | `cargo test --release -p wat --test probe_arc209_c0b3bb_verbs -- --test-threads=1` | `2 passed` (process: 42; thread: process-tier error) |
| 3 | The `authorizes` decision (uid + pid branches) | `cargo test --release -p wat --lib kernel::listener -- --test-threads=1` | `1 passed` (empty→no, allowed→yes, wrong pid→no, wrong uid→no) |
| 4 | c0b3aii unbroken (birth-seed serves its owner) | `cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1` | `1 passed` (105 — STILL green, untouched) |
| 5 | No comms regression (peer_cred/3b-a intact) | `cargo test --release -p wat --test comms -- --test-threads=1` | all pass |
| 6 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (4 known — ZERO new) |
| 7 | Full surface compiles | `cargo test --release --workspace --no-run` | clean |
| 8 | Blast radius confined | `git diff --stat` | only `src/kernel/listener.rs` + `src/runtime.rs` + `src/check.rs` (+ the 2 probes already on disk) |

## Runtime prediction

15–25 min. Three additive surfaces (a struct field + 3 methods + a birth-seed line; a ~6-line
gate; two eval fns + two infer fns + four head arms) — no structural rework, no `comms` change.
The forking probes dominate wall-clock.

## Trap-doors named

- **STOP-1 (the load-bearing premise):** if `owner_served_via_birth_seed` or c0b3aii goes RED,
  `getppid()` ≠ the owner in the service child — the birth-seed is wrong. Grounded clone3-direct
  (`clone.rs:388`, no `CLONE_PARENT`) + fork-without-exec (`spawn.rs:632`) → expected owner. If
  it bounces the owner, STOP and report (do not loosen the gate to `uid`-only).
- **`continue` target:** the gate's `continue` must re-enter the `loop` at `runtime.rs:24207`
  (re-accept), exactly the existing `WouldBlock` shape — not the outer `poll'` recursion.
- **`nil` / `Listener'<S,R>` type spelling** in `infer_allow_prime`: mirror `infer_accept_prime`
  exactly; confirm the nil-return spelling against another `-> :wat::core::nil` intrinsic.
- **Thread-tier error text:** `eval_allow_prime`'s thread-branch reason MUST contain
  `process-tier` (the verbs probe asserts the substring).

## Honest-delta slots (filled at SCORE time)

- Did `stranger_is_bounced` flip cleanly, or any surprise in the bounce path (e.g. the stranger
  surviving the dropped stream)? —
- Did c0b3aii + nursery hold exactly (`895/4`, ZERO new)? Diff stat confined to the 3 files? —
- Any STOP triggered? —
