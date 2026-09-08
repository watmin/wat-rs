# EXPECTATIONS — `sigaction`, not `signal`

Written **before** the strike. A declaration: **"nothing moved" is the pass condition.**

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ no `libc::signal` remains | `grep -rn 'libc::signal(' src/` | **no hits** |
| 2 | ★ flags are explicit | read the five installs | `sa_flags` written literally; `SA_RESTART` **set**, preserving today |
| 3 | `sa_mask` is deliberate | read the installs | set explicitly, with a comment saying why |
| 4 | the return is checked | read the installs | a failing `sigaction` is not discarded |
| 5 | handler bodies untouched | `git diff src/process/child.rs` | only the install block moved |
| 6 | the "unresolved" comment is answered | `grep -n 'deliberately unresolved' src/io.rs` | replaced by a pointer to the install site |
| 7 | ★ nothing observable moved | `scripts/floor.sh` | **the same result as before this stone**, arm for arm |
| 8 | the circuit is unaffected | `… circuit.wat` no-args | every field identical |
| 9 | the floor | `scripts/floor.sh` | **read the Summary line** |

⚠ **Row 7 is the stone.** If stone 1 has **not** landed, `probe_arc278_partial_frame_residue` is
**still expected to fail** — and that is a *pass* for this stone. Clearing that RED is stone 1's job,
and if this stone appears to fix it, something changed that should not have.

⚠ **Row 2 preserves a default rather than choosing one.** The value of `SA_RESTART` is a separate
ruling; this stone only makes it visible.

## Runtime prediction

**30–45 minutes.** One function, five installs, one comment, plus a Rust rebuild. The floor is the
long pole.

## Trap-doors named in advance

- **`sigaction` is `unsafe` and takes a zeroed `struct sigaction`.** A partially-initialised
  `sa_mask` is undefined behaviour, not a warning.
- **`sa_sigaction` vs `sa_handler`** — with no `SA_SIGINFO` the handler field is `sa_handler`;
  writing the wrong union member compiles and misbehaves.
- **Do not use `SA_RESETHAND`.** That is System V one-shot semantics, the *other* half of `signal()`'s
  ambiguity, and it is not what glibc does today.
- **`CLONE_CLEAR_SIGHAND` in `clone3`** already resets handlers in children. Verify this install runs
  where the current one does — the child re-installs after clone.

## What this stone does NOT claim

⚠ **It does not fix the RED.** Stone 1 does.

⚠ **It does not change signal delivery semantics** — it writes down the ones already in force.

⚠ **It does not fold signals into the reactor.** `signalfd` belongs with stone 3.
