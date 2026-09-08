# EXPECTATIONS — a signal is an fd

Written **before** the strike. **This stone changes the substrate's signal delivery.** The floor is
the gate; two rows are the ones that would hide a catastrophe.

Rows state what must be true, not where to look.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **SIGUSR1/2/HUP do NOT stop the process** | new probe | after `SIGUSR1`: `(sigusr1?)` is true, `(stopped?)` is **false**, the process is **alive** and still serving |
| 2 | ⛔ **SIGINT and SIGTERM still stop it** | new probe | `(stopped?)` true; a send blocked on a full pipe returns `SendError::Shutdown` |
| 3 | ★ **no handler is installed for the five** | new probe | `sigaction(sig, NULL, &old)` shows `SIG_DFL` for each, **and** each is in the blocked mask |
| 4 | ★ **the mask is blocked before any thread exists** | new probe | a freshly spawned thread reports all five blocked in its own `pthread_sigmask` |
| 5 | **the signalfd is atomic at creation** | read the diff | `SFD_CLOEXEC \| SFD_NONBLOCK` in the `signalfd` call, not set afterward |
| 6 | **the wake pipe is retired** | grep + diff | no production writer remains; `request_kernel_stop` keeps its atomic store and loses the write |
| 7 | **the lifeline still cascades** | existing lifeline probes | parent death → child shuts down, unchanged |
| 8 | **a spawned child still stops** | existing spawn/process probes | `SIGTERM` to a spawned runtime stops it; the child owns its own signalfd |
| 9 | **the transport's stop arms still fire** | `probe_arc278_partial_frame_residue`, `probe_arc278_send_poll_arm` | PASS, residue in **~3 s** — read the duration |
| 10 | **the tie-break still holds** | `the_senders_tie_break_is_a_property` | PASS — it still speaks when it can, stops when it cannot |
| 11 | **the comment stops being aspirational** | read `runtime.rs:421` | it names `signalfd` as load-bearing; now the code uses it |
| 12 | **the floor holds** | `scripts/floor.sh` | Summary line: 5227 passed (5226 + the new probe), 22 skipped, **0 FAIL, 0 TIMEOUT**, quiet box |

## The rows that would hide a catastrophe

⛔ **Row 1 is the stone.** The worker currently `break`s on *any* input fd becoming ready
(`runtime.rs:426-431`), which is safe only because SIGUSR1/2/HUP never touch the wake pipe. A signalfd
carries all five on one fd. **Without a demultiplex, `SIGUSR1` becomes a shutdown** — and no existing
test sends SIGUSR1 to a live server, so the floor would stay green over it. Prove the negative:
`(stopped?)` **false** and the process **still serving** after SIGUSR1.

⛔ **Row 4 is an ordering trap, not a coding detail.** A thread spawned *before* the mask is set does
not inherit it; that thread can then take a signal with no handler installed and the default action
kills the process. It would be intermittent, load-dependent, and would present as a mystery death.

★ **Row 3 proves the old mechanism is gone**, not merely bypassed. Handlers left installed alongside
a signalfd is two mechanisms racing for one signal.

★ **Row 9 is the regression oracle** we have been using all day. `partial_frame_residue` is the arm
that hung; a 20 s PASS is a failure wearing a pass's clothes.

## Runtime prediction

**2–3 hours.** The largest stone in this line: a process-wide mask with an ordering constraint, a new
fd type, a worker loop that changes shape from *break-on-any* to *demultiplex-and-continue*, five
handlers retired, and a pipe removed. The floor is the long pole.

## Trap-doors named in advance

- **`signalfd_siginfo` is 128 bytes and several may be queued.** Read in a loop until `EAGAIN`; one
  read is not one signal.
- **Blocked signals are not lost, they are pending** — and *coalesced*. Two SIGUSR1s may arrive as
  one. The existing contract is a polled boolean, so coalescing is already its semantics; do not add
  counting.
- **The mask survives `execve`.** Here the exec'd image is our own (`execveat` on an O_PATH fd of
  `current_exe`), so it re-establishes both. Verify the child path rather than assuming it.
- **`CLONE_CLEAR_SIGHAND` resets handlers, not the mask.** The clone3→exec window therefore holds
  signals *pending* instead of fatal-by-default — better than today, and worth stating in the SCORE.
- **A fork child needs its own signalfd**, exactly as it needs its own wake pipe today
  (`init_shutdown_signal_with_inputs` is already fork-aware — follow that guard).
- **`SIGKILL` and `SIGSTOP` cannot be blocked or read.** Nothing changes for them; do not pretend
  otherwise in the docs.

## What this stone does NOT claim

⚠ It does **not** retire the shutdown broadcast fan-out. Parent-death and an in-process trigger are
not signals; there is no signalfd for them to arrive on.
⚠ It does **not** touch `src/io.rs`, and does **not** introduce `eventfd`.
⚠ It does **not** change the wat-visible surface: `(:wat::kernel::stopped?)` and the `sigusr1?`
family behave exactly as before.
