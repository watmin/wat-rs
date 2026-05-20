# Arc 213 stone δ-1 — SCORE: ChildHandleInner pidfd field added

**Mode A.** Field minted; 3 construction sites updated; wait/kill paths unchanged. 92/92 PASS.

---

## Summary

`pub pidfd: Pidfd` added to `ChildHandleInner`. Constructor signature changed from
`new(pid: libc::pid_t, lifeline_w)` to `new(pidfd: Pidfd, lifeline_w)` with `pid`
extracted internally via `pidfd.pid()`. `pub pid: libc::pid_t` PRESERVED — libc paths
(`wait_or_cached`, `Drop::drop`, `eval_kernel_wait_child`) continue to use it unchanged.
All three γ-phase construction sites updated to pass the `Pidfd` into `ChildHandleInner`
instead of extracting `pid` and dropping the `Pidfd`.

δ-1 stores; δ-2 routes wait/kill through the stored pidfd; δ-3 retires the libc fallback
and removes `pub pid`.

---

## File changes

### `src/fork.rs`

1. **ChildHandleInner struct (lines ~184-201):** Added `pub pidfd: Pidfd` field with doc
   comment. Updated `pub pid` doc comment to note "diagnostic + libc interop until δ-3
   retires this field."

2. **ChildHandleInner::new (lines ~203-211):** Signature changed to `(pidfd: Pidfd,
   lifeline_w: Option<OwnedFd>)`. Body extracts `pid: pidfd.pid()` internally, then stores
   both `pid` and `pidfd` in the struct.

3. **Site 1 γ-1 (~line 677-680):** Removed `let pid = pidfd.pid(); drop(pidfd);`.
   `ChildHandleInner::new(pidfd, Some(lifeline_w))` — pidfd moves into handle.

4. **Site 2 γ-2 (~line 1081-1085):** Same migration. `ChildHandleInner::new(pidfd,
   Some(lifeline_w))` — pidfd moves into handle.

5. **Pidfd Debug impl:** Added manual `impl std::fmt::Debug for Pidfd` — necessary because
   `ChildHandleInner` derives `Debug` and Pidfd did not implement it. Surfaces fd (raw int)
   + pid. Used `self.fd.as_raw_fd()` which was already in scope via the existing
   `use std::os::fd::AsRawFd` import.

### `src/spawn_process.rs`

1. **Site 3 γ-3 (~line 252-255):** Removed `let pid = pidfd.pid(); drop(pidfd);`.
   `ChildHandleInner::new(pidfd, Some(lifeline_w))` — pidfd moves into handle.

2. **Import:** No `Pidfd` import needed. The `pidfd` variable's type is inferred from
   `spawn_lifelined`'s return type (defined in `fork.rs`). Adding `Pidfd` to the use line
   produced an `unused_import` warning; reverted. `ChildHandleInner` import already present.

---

## Compiler surprise: Pidfd missing Debug

`ChildHandleInner` derives `Debug`. Adding `pidfd: Pidfd` as a field triggered:

```
error[E0277]: `Pidfd` doesn't implement `Debug`
```

Fix: added manual `impl std::fmt::Debug for Pidfd` immediately after the `Pidfd` struct
definition. The impl uses `f.debug_struct("Pidfd").field("fd", &self.fd.as_raw_fd()).field("pid", &self.pid).finish()`. One syntactic-fix retry consumed. Build clean post-fix.

---

## Pidfd lifetime + Arc<ChildHandleInner> sharing — confirmation

`Pidfd` wraps `OwnedFd`. `ChildHandleInner` is wrapped in `Arc<ChildHandleInner>`. Cloning
the Arc does NOT clone the Pidfd — the fd is owned once. When the last Arc drops,
`ChildHandleInner::drop` fires → `Pidfd::drop` fires → the kernel fd closes.

This matches `lifeline_w`'s lifetime exactly (same struct, same Arc). Verified by running
the lifeline + pdeathsig probes (all PASS). No early-close hazard observed.

---

## Verification: 92/92 PASS

| Binary | Pre-δ-1 | Post-δ-1 |
|---|---|---|
| probe_pidfd_primitive | 2/2 | 2/2 |
| arc112_scheme_probe | 1/1 | 1/1 |
| arc112_slice2b_process_send_recv | 1/1 | 1/1 |
| probe_closure_body_prelude_lift | 5/5 | 5/5 |
| probe_counter_actor_process_diag | 3/3 | 3/3 |
| probe_declaration_form_lift | 6/6 | 6/6 |
| probe_def_not_special | 5/5 | 5/5 |
| probe_lifeline_orphan_clean_via_fork_program | 1/1 | 1/1 |
| probe_lifeline_orphan_clean_via_substrate | 1/1 | 1/1 |
| probe_pdeathsig_diagnostic | 1/1 | 1/1 |
| probe_pdeathsig_kills_orphan_child | 1/1 | 1/1 |
| probe_run_hermetic_no_deadlock | 2/2 | 2/2 |
| probe_spawn_process_parent_type | 3/3 | 3/3 |
| probe_spawn_process_stdin | 1/1 | 1/1 |
| probe_spawn_process_stdio | 1/1 | 1/1 |
| wat_arc170_program_contracts | 24/24 | 24/24 |
| wat_arc170_stone_a_drain_and_join | 4/4 | 4/4 |
| wat_arc208_process_io_result | 7/7 | 7/7 |
| wat_process_peer_ipc_round_trip | 3/3 | 3/3 |
| wat_harness_deps | 3/3 | 3/3 |
| probe_shutdown_cascade_crossbeam | 1/1 | 1/1 |
| probe_shutdown_cascade_pipefd | 1/1 | 1/1 |
| wat-cli wat_cli | 15/15 | 15/15 |

**Total: 92/92 GREEN post-δ-1. Zero regressions.**

---

## Notes

- No behavior changed. δ-1 stores; libc::waitpid/kill paths in wait_or_cached, Drop::drop,
  eval_kernel_wait_child are UNCHANGED.
- `pub pid: libc::pid_t` PRESERVED. δ-3 territory.
- `wait_or_cached` UNCHANGED. δ-2 territory.
- `Drop::drop` UNCHANGED. δ-2 territory.
- `eval_kernel_wait_child` UNCHANGED. δ-2 territory.
- STOP triggers: none triggered.

---

## Mode classification

**Mode A.** Field added; signature changed; 3 sites updated; Debug impl added for Pidfd
(compiler-required by derive chain); cargo build clean; 92/92 baselines preserved; SCORE
written.
