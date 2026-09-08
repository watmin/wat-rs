# DESIGN — a delivered signal is observable

**Fixes a regression shipped at `5094221aa`** (`a signal is an fd`). Builder's ruling: make the
guarantee true, not document it away.

## The regression

`.floor/2026-09-08T18-08-14Z/` — captured, not re-run:

```
Summary [ 477.344s] 5232 tests run: 5231 passed (7 slow), 1 failed, 22 skipped
FAIL [ 2.025s] wat::kernel test::deftest_wat_tests_service_signal_observer_measures_itself
  wat-tests/service-signal-observer.wat:84:3  assert-eq failed
  actual:   [true, true, true, true, true, FALSE, true, …]
  expected: [true × 16]
```

Index 5 is `hup` from `after-hangup` — `(:wat::kernel::sighup?)`, read inside the service on the first
`observe` after SIGHUP was delivered.

**Mechanism.** Under `sigaction` the child's handler set `KERNEL_SIGHUP` when the child was scheduled,
which a full IPC round trip (~179 µs) almost always beat. Under signalfd the flag is set only when the
child's *shutdown-worker thread* is scheduled, polls, reads and dispatches — competing with the
service thread that answers `observe`.

★ The window existed before and was merely narrow; the migration widened it. It survived **two
independent green floors** — grok's and mine — and surfaced on the sixth run of the same tree.

## What the guarantee should be

> After `(:wat::kernel::signal proc …)` returns `Delivered`, a subsequent request to that process
> observes the corresponding flag.

Under handlers this held *by luck*. It must now hold *by construction*.

## The insight

The signals are **blocked**, so on delivery the kernel puts them in the process-wide **pending set
synchronously with the sender's syscall** — before `pidfd_send_signal` returns. The pending set is
therefore an authoritative record of "this process has received X", available *earlier* than any
handler could have run.

Measured on this box, not assumed:

```
sigpending() cross-thread        a process-directed pending signal IS visible from another thread
sigtimedwait({SIGHUP}, 0)        consumed SIGHUP and left SIGTERM pending      (selective)
sigtimedwait({SIGUSR1}, 0)       returns in 2 µs when nothing is pending       (never blocks)
```

★★ So the observation becomes **atomic-first, pending-set-second**, and the union has no gap:

```
t0  kill(SIGHUP)   → pending gains SIGHUP,  atomic false  → union TRUE
t1  worker drains  → pending clears,        atomic true   → union TRUE
```

Never both false after delivery. And because the atomic is checked first, the syscall happens **only
while the flag is false** — the steady state costs nothing.

## ⛔ Why the two flag families must differ

A naïve `sigpending()` union breaks `reset-sighup!`: reset clears the latch, the signal is still
pending, and the next read re-reports it. So the resettable flags must **consume**; and the stop flag
must **not**.

| flag | probe | why |
|---|---|---|
| `sighup?` `sigusr1?` `sigusr2?` | `sigtimedwait({that one}, 0)`; on success latch the atomic | **Consuming** keeps `reset-*!` honest. A single-signal set touches nothing else — measured. |
| `stopped?` | `sigpending()` membership of `{SIGINT, SIGTERM}` | **Must not consume.** If a reader dequeued SIGTERM the worker would never wake, the broadcast would never fire, and shutdown would never cascade. Safe because there is **no production `reset-stopped!`** — it is a one-way latch (only a `#[cfg(test)]` reset exists). |

★★★ That asymmetry is derived, not chosen: *consume where a reset must be able to clear the record;
never consume what another thread is obliged to see.*

## The one contract decision

**`Delivered` implies observable.** The flag readers become the authority on "has this process
received X", answering from the atomic when latched and from the kernel's pending set when not.

## Out of scope — REJECTED

- **A sleep, retry, or wait in the test.** The contract is the thing under repair; making the test
  patient would hide it.
- **Waking the worker from a reader.** That needs a wake fd, which reintroduces the self-pipe the
  previous stone retired.
- **Draining stop-class signals at the reader.** See the table — the worker must see them.

## Files

`src/intrinsic/kernel/ambient.rs` (the four readers — its module doc names all seven verbs) and/or
`src/runtime.rs` (the atomics at `:102-127`; a `KERNEL_STOPPED.load` at `:19634`). Establish which is
the live read path before editing.
