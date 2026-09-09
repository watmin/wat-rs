# DESIGN — `sigaction`, not `signal`

**`libc::signal()` becomes `libc::sigaction()` with explicit flags.** `src/process/child.rs`.
Stone 2 of 3 from `FINDING-the-writes-kept-the-1970s.md`.

## WHY — the least-specified interface in POSIX, in a Linux-only runtime

`src/process/child.rs:74-95` installs **five** handlers with `libc::signal()`:

```rust
libc::signal(libc::SIGINT,  substrate_on_stop_signal  as …);
libc::signal(libc::SIGTERM, substrate_on_stop_signal  as …);
libc::signal(libc::SIGUSR1, substrate_on_sigusr1      as …);
libc::signal(libc::SIGUSR2, substrate_on_sigusr2      as …);
libc::signal(libc::SIGHUP,  substrate_on_sighup       as …);
```

★★★ **`signal()` is the least-specified signal interface there is.** Its semantics differ between
System V (one-shot, no restart) and BSD (persistent, `SA_RESTART`) — an ambiguity so bad that
`signal(2)` itself says *"avoid its use: use `sigaction(2)` instead."* glibc resolves it to BSD, so
every handler here silently carries **`SA_RESTART`**, and no line of code says so.

⚠ That invisible flag is one of the three ingredients in the RED: it is why `EINTR` never reaches a
blocked write. `src/io.rs:670` even names the question and leaves it open —
*"whether EINTR ever reaches here under this repo's signal handlers is **deliberately unresolved**."*
**With `sigaction` it stops being unresolved and starts being declared.**

## ⛔ AND IT CONTRADICTS THE PLATFORM FLOOR

```
src/process/mod.rs:9    Linux 5.3+ (clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND), 5.9+
src/process/mod.rs:13   "Production use requires Linux 5.9+; the 6.x floor is …"
```

The builder: *"we are a linux programming language... it screams to me that we are not sticking
that."* A runtime that hard-requires **`CLONE_CLEAR_SIGHAND`** — a flag whose entire job is to
control handler inheritance across `clone3` — has no portability argument left for the 1970s
interface that cannot express its own flags.

## ⛔ THE ONE CONTRACT DECISION — the flags become explicit, not the behaviour

`sigaction` with the flag set **written down**. Whether `SA_RESTART` stays or goes is a **separate,
deliberate choice** — and this stone's default is to **preserve today's behaviour exactly**
(`SA_RESTART` set), so it changes nothing observable.

★★★ **This stone is a declaration, not a behaviour change.** It converts an inherited default into a
stated one. Changing the value is a different stone with its own gate, and doing both at once would
make a floor red unattributable.

⚠ **Clearing `SA_RESTART` is explicitly NOT the fix for the RED.** EINTR-based cancellation is racy —
the signal can land between the check and the syscall and the write blocks anyway. Stone 1 is the
fix. If stone 1 lands first, this stone's `SA_RESTART` choice stops being load-bearing at all, which
is the right order.

## WHAT THIS IS AND IS NOT

⚠ **No behaviour change is intended, and that is the gate:** the floor is green and the RED probe's
status is *unchanged by this stone alone* (still red if stone 1 has not landed; still green if it
has).

⚠ **Async-signal-safety is unchanged.** The handlers keep their bodies; `child.rs:30-40` documents
why they are safe (`AtomicBool::store` + `libc::write` to an open fd). Nothing in that reasoning
depends on which installer was used.

## OUT OF SCOPE — REJECTED

- **Clearing `SA_RESTART`.** A behaviour change; its own stone, and probably unnecessary once stone 1
  removes the reliance on EINTR entirely.
- **`signalfd`.** Linux-native and arguably the right long-term shape — but the substrate already has
  a shutdown broadcast **fd** doing that job. Folding signals into the reactor belongs with stone 3.
- **The handler bodies.** Untouched.
