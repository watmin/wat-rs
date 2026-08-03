# BRIEF — P2: mint `:wat::kernel::signal`, `Signal`, and `SignalOutcome`

**Stone:** `DESIGN-STONE-process-signal-owner-to-child.md` — fully ruled, no open questions.
**RED probe (worked reference, run it first):** `docs/arc/2026/06/278-rules-engine/probes/red-owner-signals-child.wat`
**Scope:** the mint only. The tests are P3/P4 and are NOT in this strike.

## The work, in one paragraph

An owner that spawned a child process has no way to signal it. The kernel mechanism already exists
and is already generic — `Pidfd::send_signal(sig: i32)` (`src/process/clone.rs:195`), whose only
caller hardcodes `SIGKILL` on the Drop path. The pidfd already lives in the process peer
(`src/kernel/peer.rs:539`). Mint the wat surface over it: one verb, one closed signal enum, one
outcome enum, and the must-use gate that forbids dropping the outcome.

## Read in order

1. `docs/arc/2026/06/278-rules-engine/probes/red-owner-signals-child.wat` — **run it.** It is RED
   today and names the exact gap. Its header records the measured output and, importantly, that the
   **runtime is the arbiter, not `--check`** (`--check` returns exit 0 on a missing head — positive-
   controlled). This file must run to completion when you are done.
2. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-process-signal-owner-to-child.md` § *The shape* —
   the ratified names, the three tiers, and why each was chosen. Do not re-derive it.
3. `src/process/clone.rs:185-216` — `Pidfd::send_signal` and the `pid()` doc that forbids `kill(pid)`.
4. `src/kernel/peer.rs:530-560` — where the pidfd sits in the bundle.
5. `src/types.rs:1711-1760` — `CloseOutcome`: the closest shipped sibling, non-parametric, three
   variants, one carrying `Failure`. Your `SignalOutcome` mirrors this shape.
6. `src/check.rs:7014-7075` — `MUST_USE_TYPES` (non-parametric, colon-Path) vs
   `MUST_USE_PARAMETRIC_HEADS`. `SignalOutcome` is non-parametric, so it joins the **first** list,
   beside `CloseOutcome`.
7. `wat/spawn.wat:238-250` — `Process` derives `Peer`. The verb takes `Process`, not `Peer`.

## What to build

**The signal enum** — `:wat::kernel::Signal`, six variants, no fields. Name verified free.

| tier | variant | POSIX |
|---|---|---|
| flag | `User1` | SIGUSR1 |
| flag | `User2` | SIGUSR2 |
| flag | `Hangup` | SIGHUP |
| stop | `Interrupt` | SIGINT |
| stop | `Terminate` | SIGTERM |
| kill | `Kill` | SIGKILL |

**Its doc comment carries the three-tier table from the stone, verbatim** — who observes each signal
and how (`(sigusr1?)` / `(stopped?)` / owner-only via `CloseOutcome::Signaled`). That table is the
only honest home for two facts the names cannot carry: that `Interrupt` and `Terminate` land on the
same predicate, and that `Kill` has no child-side observable at all.

**The outcome enum** — `:wat::kernel::SignalOutcome`, non-parametric:

```clojure
Delivered                    ;; the kernel accepted it for that process
Gone                         ;; the child had already exited (ESRCH)  — see STOP-2
Failed[cause <- Failure]     ;; io failure
```

**The verb** — `:wat::kernel::signal`, taking `Process<I,O>` and a `Signal`, returning
`SignalOutcome`. No prime. No `{:restricted-to …}` — holding the `Process` is the capability.
Delivery routes through `Pidfd::send_signal`.

**The gate** — add `":wat::kernel::SignalOutcome"` to `MUST_USE_TYPES` so a dropped outcome is a
compile error in both discard doors (`do`-non-final and `let`-`_`).

**One WHY comment** on `CloseOutcome::Signaled` (or on `Signal`) bridging the asymmetry: the send
side is a closed enum because we choose what is sendable; the receive side is a bare `i64` because
we do not control what kills you. Without it, a reader touching both sees one concept spelled two
ways with nothing joining them.

## STOP triggers — ship nothing, surface the gap

- **STOP-1 — the verb goes on `Process`.** If the implementation path pushes toward `Peer` (a shared
  derive, a convenience, codegen), STOP. A `Peer` verb is partial: a thread peer has no process to
  signal.
- **STOP-2 — `Gone` must earn BOTH its existence and its shape.** Prove ESRCH is reachable through a
  pidfd for a child that exited but was not reaped. If it is not reachable, do not mint the arm —
  two arms and a raise is a correct answer. If it is reachable, mint it **with no field** unless you
  can show the cause varies; a field whose value never differs is one the reader carries for
  nothing. Report which you found.
- **STOP-3 — never `kill(pid, sig)`.** Route through `Pidfd::send_signal`. `clone.rs:215-216`
  documents why. A `libc::kill` in the diff is a rejected strike.
- **STOP-4 — `Kill` sends only; it does not reap.** `ChildHandle::Drop` stays the only unconditional
  SIGKILL+reap path (`handle.rs:17`). A `Kill` that reaps destroys the reason the variant exists:
  killing and then still inspecting the exit status.
- **STOP-5 — do not touch the signal flags or the runtime's measurement surface.** `KERNEL_SIGUSR1`
  and siblings stay process-global statics. This strike adds a send path; it changes nothing about
  how a program observes.
- **STOP-6 — no `_` wildcard arm on either enum.**
- **STOP-7 — EINVAL and EBADF stay raises.** They are must-never-happen (the enum makes a bad signal
  unrepresentable; a closed pidfd is a substrate bug), not handleable conditions.

## Done means

- The RED probe runs to completion instead of dying on an unknown function.
- A dropped `SignalOutcome` is a compile error — demonstrate it in **both** discard doors.
- `cargo nextest run --release` Summary line reports zero new failures against the floor.
- `cargo clippy` clean.
- STOP-2's finding reported explicitly, whichever way it went.

Do not commit; do not push. Report what you built, what you measured, and every STOP you hit.
