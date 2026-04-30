# Arc 106 — backlog

## Status

| Slice | State | Notes |
|---|---|---|
| 1 — substrate | ready | setpgid + wat handlers in fork child + CHILD_PID→CHILD_PGID + killpg. |
| 2 — purge the flake | obvious in shape, blocked by 1 | rewrite sigterm test for polling contract; verify 100× clean. |
| 3 — cascade depth | obvious in shape, blocked by 1+2 | new test: parent forks grandchild, both poll, both exit 0 on cascade. |
| 4 — INSCRIPTION + USER-GUIDE + 058 + memory | blocked on 1+2+3 | record contracts, contracts the substrate now upholds, the discipline behind it. |

## Sub-fogs

### Slice 1

**1a — what's the canonical wat-handler installer?**
The cli installs handlers in `crates/wat-cli/src/lib.rs::install_signal_handlers()`. Slice 1 needs the SAME handler set in the fork child. The handler logic lives in cli; the fork child lives in the substrate (`src/fork.rs`). Cleanest: hoist the handler installation into a substrate-level helper that both the cli and fork child call. Otherwise: duplicate the handler set in fork.rs.

Resolves at code time. Read `install_signal_handlers` body, decide on hoist vs duplicate.

**1b — KERNEL_STOPPED state at fork time.**
The child's `KERNEL_STOPPED` is COW-copied from the parent at fork. If the parent already had `true` (e.g., the cli is in shutdown when a wat program forks a grandchild), the grandchild starts with `true` and immediately polls itself out of any loop. **Is this correct?** Yes — the cli is dying; new children inheriting the stop signal IS the cascade we want. Document the property; don't fight it.

**1c — setpgid failure mode.**
`libc::setpgid(0, 0)` can fail if the calling process is already a session leader (would return EPERM). The cli's child should never be a session leader (cli doesn't call setsid), so EPERM shouldn't happen. Defensive: if setpgid fails, `_exit(EXIT_STARTUP_ERROR)` — non-recoverable, the cascade contract is broken. Surface the error code via stderr first.

### Slice 2

**2a — READY lock-step protocol.**
The wat program prints "READY" before entering the polling loop. Test reads stdout one line; asserts "READY"; only then sends SIGTERM. By READY:
- cli forked the child (CHILD_PGID set)
- cli installed wat handlers
- child setpgid'd into its own group
- child installed wat handlers
- child loaded program; reached `:user::main`'s body
- child println'd "READY"

So when SIGTERM is sent, every cascade prerequisite is settled. No race window.

**2b — what's the right test runner discipline for the polling test?**
Wrap in `wat::fork::run_in_fork` (per chapter 29's "Reuse before invent"). Each test runs in a fresh forked subprocess so the cargo-test harness's stdin/stdout/stderr / signal masks / parent process state can't contaminate. Same shape `tests/wat_harness_deps.rs` uses for OnceLock isolation.

**2c — verifying zero flake.**
Run the test 100× via shell loop. Zero failures. If even one failure surfaces, the race is somewhere we missed and slice 1 is incomplete. Don't paper over.

### Slice 3

**3a — how does the parent fork the grandchild?**
`:wat::kernel::fork-program-ast` returns a `:wat::kernel::ForkedChild` with the grandchild's pid, stdin, stdout, stderr, plus a `ChildHandle` for `wait-child`. The parent wat program calls this once, holds the handle, polls `stopped?` and waits.

**3b — what does the parent do on stopped?**
Parent observes `stopped?` → calls `wait-child` on the grandchild's handle (the grandchild also observed stopped? via cascade, returned cleanly, exited 0; parent's waitpid sees WIFEXITED 0) → returns ().

**3c — lock-step shape for two processes.**
Both processes print READY markers ("PARENT READY" + "GRANDCHILD READY"). Test reads two lines, sends SIGTERM, asserts both exit 0 via the parent's wait_with_output (the grandchild's exit is observed via the parent's wait-child; the parent's exit is observed via the cli's waitpid).

**3d — pgid inheritance verification.**
The grandchild MUST inherit the parent's pgid (no setpgid call in the grandchild's child_branch_from_source). A mistake here would make the grandchild its own group, breaking the cascade. Verify with `ps -o pid,pgid` inspection during the test, OR simpler: trust the kernel's default fork inheritance (POSIX-mandated). The test's assertion (grandchild exits 0 on cascade) is the final proof.

### Slice 4

**4a — INSCRIPTION shape.**
Same as every prior arc: what shipped per slice with commit refs, sub-fog resolutions named alongside the code, divergences from DESIGN if any.

**4b — USER-GUIDE updates.**
- Section on signal handling: the polling contract now works through fork; cite arc 106.
- Section on fork: cascade is mandatory; cite arc 106.
- Section on shutdown: clean shutdown via stopped? polling is the canonical pattern; cite arc 106.

**4c — 058 FOUNDATION-CHANGELOG row.**
Lab repo, same shape as every prior arc's row. Quote the arc; link the INSCRIPTION.

**4d — memory entries.**
- `feedback_tests_not_flaky.md` — the discipline itself.
- `project_signal_cascade.md` — the substrate's contract for fork-cascade signals via process groups.
- Update `feedback_no_polling_loops.md` if the polling contract changes anything about the user-instruction "don't `until grep ...; do sleep`" pattern.

## Open questions resolved before code

None. The path is clear — slice 1 implements; slice 2 verifies; slice 3 deepens; slice 4 records.
