# DESIGN — an unexpected errno is not a clean drain

Small stone, one decision. Follow-up to `a signal is an fd` (STRUCK, `5094221aa`), named in that
stone's commit rather than fixed inside it.

## The collapse

`drain_signalfd` (`src/runtime.rs:305-343`) has four distinguishable outcomes and two exits:

| outcome | meaning | today |
|---|---|---|
| `EAGAIN` / `EWOULDBLOCK` | the drain is complete | `break` → return false |
| `EINTR` | interrupted, retry | `continue` ✅ |
| any other errno | **the signalfd cannot be read** | `break` → return false — *identical to done* |
| `n == 0` | undocumented for a signalfd | `break` → return false — *identical to done* |

**Can two different worlds print this line?** Yes: a clean drain and a broken signalfd exit the same
way, silently, with the same return value.

## Why it is not cosmetic

The worker's loop (`runtime.rs:484-505`) reacts to `false` by polling again:

```
poll → signalfd POLLIN → drain returns false → stop_class false → poll → POLLIN → …
```

If the fd is persistently readable and the read persistently fails, that is a **busy spin at 100 % CPU
in which signals are never processed** — a process that can no longer be stopped by anything but
`SIGKILL`. In a runtime whose entire shutdown contract is *"the signal arrives as an fd event,"* an
unreadable signalfd is the one failure that must not be silent.

★ This is the same shape as the live-lock caught on the sibling stone hours earlier: **a readable fd,
an empty drain, and a loop that responds by looping.** There it was reachable and fired on a real
floor. Here it is not reachable today — and that is exactly what the state-no-test-builds looked like
the four times it bit this arc.

## Constraint, not failure

No trigger is reachable today: the worker owns the fd for process lifetime, and the read passes
exactly `size_of::<signalfd_siginfo>()`. So this is **constraint engineering** — derive the *cannot*
from what the thing is:

> The signalfd is the only way this process learns it should stop. If it cannot be read, the process
> cannot be stopped. **Continuing is not one of the available honest outcomes.**

## The one contract decision

**An unexpected errno, and `n == 0`, are substrate-fatal: raw diagnostic to fd 2, then `_exit(1)`.**

This is not a new policy — it is the file's own, twice:

```
runtime.rs:432-436   signalfd(2) failed during shutdown init  → write(2, …) ; _exit(1)
runtime.rs:446-451   pipe2(2) failed during broadcast init    → write(2, …) ; _exit(1)
                     "_exit(2): fork-safe; same rationale as signalfd failure above."
```

The drain runs on the worker thread, post-fork-safe constraints apply, and `write`/`_exit` are both
async-signal-safe — the same justification those two sites already carry.

## ★ Make the decision testable, not the syscall

The error path cannot be exercised through `drain_signalfd` — it is private and needs a broken fd.
So **separate the decision from the syscall**: a small pure function mapping an errno to
`Done | Retry | Fatal`, unit-tested directly. The loop then reads as three named outcomes rather than
two `break`s that mean different things.

That is the row's real content: *gate on the property, and make the property reachable by a test.*

## Four questions

| | |
|---|---|
| Obvious? | **YES** — three named outcomes instead of two silent ones; a reader sees which errno means what |
| Simple? | **YES** — one pure decision function, and the loop gets shorter |
| Honest? | **YES** — it is the whole point: the two worlds stop printing the same line |
| Good UX? | **YES** — an operator gets a diagnostic and an exit code instead of a wedged process at 100 % CPU |

## Out of scope — REJECTED

- **A retry budget, backoff, or spin counter.** That would paper over an unreadable fd rather than
  report it. The wait must be honest or fatal.
- **Anything else in the worker.** The demux, the mask, and the ctor are correct and STRUCK.

## Files

`src/runtime.rs` — the decision function, its unit tests, and the drain loop.
