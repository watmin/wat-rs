# Arc 213 stone α — EXPECTATIONS

## Independent prediction

- **Runtime band:** 30-60 min Mode A. Substrate primitive minting with libc syscall plumbing — bigger than walker migrations but bounded (one new helper + types + smoke probe).
- **LOC changed:** ~150-250 (new types ~60, helper ~80, smoke probe ~80, possible lib.rs re-exports ~3)
- **New files:** 2 (smoke probe + SCORE)
- **Surprises expected:** MEDIUM — libc crate's clone3 / clone_args / CLONE_PIDFD bindings may have version-specific quirks; pidfd_send_signal may require manual syscall wrapping if not in libc; OwnedFd integration with kernel-returned raw fd is the kind of plumbing that has small footguns.

## Honest-delta watch

This is FOUNDATION minting. Three risk surfaces:

1. **clone3 plumbing:** the `libc::clone_args` struct may have fields that need careful initialization (set_tid, exit_signal, cgroup, etc.). Default-init to 0 for unused fields. If libc crate doesn't expose `clone_args` or `SYS_clone3`, sonnet may need a fallback to inline syscall declarations.

2. **pidfd_send_signal:** kernel syscall (Linux 5.1+); may not be in `libc` crate as a direct fn. If not, sonnet uses `libc::syscall(libc::SYS_pidfd_send_signal, ...)`.

3. **waitid(P_PIDFD):** the `P_PIDFD` constant (Linux 5.4+) should be in libc. The `siginfo_t` extraction of exit status is the standard waitid pattern.

If any of these have non-obvious plumbing issues, sonnet documents in SCORE rather than inventing workarounds. STOP-trigger 1 fires for clone3 unavailability; trigger 3 for syntactic plumbing issues.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `Pidfd` type minted with Drop, Send, Sync, NO from_pid constructor | YES |
| 2 | `LifelineWriter` type minted with Drop | YES |
| 3 | `spawn_lifelined` helper uses clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND | YES |
| 4 | Lifeline pipe created pre-fork; inherited atomically via clone3 | YES |
| 5 | setpgid(0, 0) in child post-fork | YES |
| 6 | `Pidfd::poll_exit` / `wait_status` / `try_wait` / `send_signal` methods all use kernel-direct syscalls (no /proc) | YES |
| 7 | Smoke probe: `pidfd_observes_normal_exit` passes (Exited(42)) | YES |
| 8 | Smoke probe: `pidfd_observes_signal_exit` passes (Signaled(SIGTERM)) | YES |
| 9 | cargo build --release clean | YES |
| 10 | Zero existing-code modifications (purely additive) | YES |
| 11 | SCORE inscribes any libc/syscall plumbing subtleties | YES |

## Mode classification

- **Mode A:** all criteria satisfied; primitive ready for β/γ/δ/ε to migrate to
- **Mode B (acceptable):** clone3 plumbing has a libc-version-specific issue surfaced honestly; types + helper drafted with TODO marker; SCORE describes the plumbing gap for orchestrator decision (possibly bump libc dep in a separate stone)
- **Mode C:** STOP rule broken (migrated existing code, removed libc::fork publicness, scope-crept into β)

## Calibration metadata

- **Orchestrator confidence:** HIGH on the design (the protocol is well-articulated in arc 213 DESIGN); MEDIUM on first-attempt Mode A (substrate primitive minting always has plumbing edge cases).
- **Risk factors:**
  - libc crate version may not expose all needed symbols (likely needs manual syscall wrappers for clone3 or pidfd_send_signal)
  - OwnedFd wrapping the kernel-returned raw pidfd needs careful unsafe handling
  - waitid siginfo_t extraction has historical quirks (use libc::WIFEXITED + WEXITSTATUS macros)
- **Why this matters:** every subsequent arc 213 stone (β through ζ) builds on this primitive. The Pidfd type IS the canonical handle; spawn_lifelined IS the canonical fork; both must be sound before migration begins. Also: arc 212 ζ-newtype-wall references this primitive as the L2 typestate-equivalent precedent — getting Pidfd right makes ζ tractable.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

This stone was chosen as the immediate next move because:
- Shipping α first makes arc 212 ζ more tractable (provides a concrete L2 typestate-equivalent worked example: private fd, canonical-only constructor)
- Shipping arc 212 ζ first does NOT make α more tractable (the minting is independent of walker enforcement)

The shipping order produces a concrete substrate-imposed-not-followed precedent BEFORE the substrate has to articulate the same pattern at a different layer.

## Cross-references

- Arc 213 DESIGN § "Scope EXPANDED 2026-05-18" — the locked stone chain; this stone is α
- Arc 213 DESIGN § "The substrate's goal-state Linux 5.3+ process primitives" — the canonical protocol this primitive implements
- Arc 213 DESIGN § "L2 enforcement (substrate-imposed-not-followed for fork primitives)" — the eventual ζ enforcement; this stone is the mint, not the enforce
- Arc 212 DESIGN § "Locked stone chain (L0 → L4 trajectory)" — ζ-newtype-wall (the sibling L2 stone that benefits from this primitive as precedent)
- `feedback_tractability_tiebreaker` — the decision discipline that ordered α before arc 212 ζ
- `feedback_no_windows` — the Linux-first commitment unlocking these primitives
- INTERSTITIAL § 2026-05-18 (post-PURGE) — the doctrine moment + Linux 5.3+ commitment
- `tests/probe_lifeline_pipe_proof.rs` — the existing manual-pipe-management probe that demonstrates the lifeline mechanism in isolation; spawn_lifelined absorbs this pattern into a canonical helper
- `src/fork.rs` — the file this stone augments (additive only)
- BRIEF-213-ALPHA-MINT-PIDFD-PRIMITIVE.md — the brief itself
