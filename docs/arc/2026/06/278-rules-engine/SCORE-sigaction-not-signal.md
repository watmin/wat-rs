# SCORE — `sigaction`, not `signal`

**SCORED.** Executor: grok, 2026-09-08. A declaration: SA_RESTART is
written down. Behaviour preserved. Tree dirty with this stone plus
two others (stone 1's `src/io.rs` + `src/comms/process.rs`;
`circuit.wat` from the reconnect stone). None of those reverted or
committed.

```
Summary [ 506.055s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T05-14-09Z/`

## WHAT LANDED

`src/process/child.rs` `install_substrate_signal_handlers`: five
`libc::sigaction` installs (`SIGINT`, `SIGTERM`, `SIGUSR1`,
`SIGUSR2`, `SIGHUP`).

- `sa_flags = SA_RESTART` — glibc `signal()` BSD semantics, now
  explicit. Not cleared.
- `sa_mask` zeroed — empty, `signal()`'s equivalent (no extra signals
  blocked). Comment says so.
- No `SA_SIGINFO` — `sa_sigaction` is read as `sa_handler`.
- No `SA_RESETHAND`.
- Return checked: `sigaction` failure panics at boot with errno.

Handler bodies untouched. `CLONE_CLEAR_SIGHAND` still clears inherited
handlers; the child still re-installs via this same function after
clone/exec. STOP-5 did not fire.

`src/io.rs`: the EINTR comment now points at that install site. Stone
1's `O_NONBLOCK` machinery is intact.

`src/host/entry.rs`: the grep gate is `src/` entire. A second five-way
`libc::signal` installer lived here (`wat::main!`). It now calls
`install_substrate_signal_handlers`. Same five signals, same runtime
stores. Duplicate handler wrappers removed.

## THE GATE — nothing observable moved

| probe | before this stone | after |
|---|---|---|
| `partial_frame_residue` | 3.019 s PASS | **3.035 s** isolated, **3.047 s** floor |
| `send_poll_arm` | 0.016 s PASS | **0.025 s** isolated, **0.011 s** floor |

Unchanged, not faster. A declaration cannot alter behaviour.

no-args: `total=8000;distinct=8000;dup=0;seen-recorded=8000`.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ no `libc::signal(` in `src/` | ✅ none |
| 2 | ★ flags explicit, `SA_RESTART` set | ✅ |
| 3 | `sa_mask` deliberate | ✅ empty, commented |
| 4 | return checked | ✅ panic with errno |
| 5 | handler bodies untouched | ✅ child.rs diff is the install block |
| 6 | "unresolved" comment answered | ✅ pointer to `install_substrate_signal_handlers` |
| 7 | ★ nothing observable moved | ✅ probes same status and ~same time; floor same 5221/22 |
| 8 | circuit unaffected | ✅ no-args identity on pre-existing fields |
| 9 | the floor | ✅ `Summary [ 506.055s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

STOP-1 / STOP-2 / STOP-3 / STOP-5 / STOP-6 held. Did not re-run the
floor.

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## BLAST

```
 src/process/child.rs |  44 ++++++++-------
 src/host/entry.rs    |  33 ++---------
```

`src/io.rs` comment-only relative to stone 1. `src/comms/process.rs`
and `wat-scripts/` not edited by this stone.
