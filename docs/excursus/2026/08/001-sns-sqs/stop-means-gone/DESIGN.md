# DESIGN — stop means gone

Builder, on the race surfaced by `the-crash-surface-is-enumerated`: *"we have a race?....... wat is meant
to be lock step all times.... races are to be annihilated on detection."*

**Drawn 2026-09-12. NOT STRUCK.**

## ⛔ FIRST, THE CORRECTION I OWE — my reported rate was measured on a BUSY BOX

I reported *"4-of-5 Lost, 1-of-5 Sent"*. Both `Sent` observations were taken while a floor run was
**compiling in the background**, which violates this campaign's own standing rule that the box must be
quiet before timing. Re-measured:

```
quiet box, 30 runs     30 × Lost                       — no race visible at all
8 spinners, 20 runs     17 × Lost · 3 × Sent            — deliberately reproduced
```

**The rate was wrong. The race is real**, and it is *load-dependent* — which makes it worse than a flat
1-in-5, because it hides on an idle CI box and appears under production load.

★ The shape is also sharper than I first said. Under controlled load the common form is

```
try-send-after-stop=Sent ; send-after-stop=Lost
```

— the **`try-send` succeeds and the immediately following `send` fails**, so the teardown lands between
two adjacent statements. On a very heavily loaded box both report `Sent`.

## The mechanism — four layers, and each is on disk

```
1  wat/service.wat:2287     the serve loop's Admin::Stop arm sends Status::Stopped UP THE LINEAGE,
                            *then* terminates (returns nil, no recur) — the ack PRECEDES the exit
2  wat/service.wat:2968     the generated <S>/stop returns as soon as owner-recv-loop sees
                            Status::Stopped. The process is still winding down.
3  wat/service.wat:3669     handle-fields = [handle <- Peer  addr <- Address]
                            ⛔ the reapable lineage (Process/Thread) is NOT IN THE HANDLE AT ALL
4  wat/spawn.wat            only options builders (thread/process/…opts). No reap, join or wait.
                            close' is :wat::kernel::-restricted (src/types.rs:2105)
```

**`stop` returns an ACK, not a REAP — and has nothing to reap with even if it wanted to.** Any operation
on a separately-dialed peer after `stop` races the OS teardown of the service process.

⚠ The machinery exists in Rust and is simply unreachable from wat: `ChildHandle::Drop` kills+reaps
(`src/process/handle.rs:128`), `try_wait` (`src/process/clone.rs:171`), and `runtime.rs:25900` already
calls `bundle.peer.wait()`.

## ⭑⭑ AND THIS IS THE SAME WALL AS 4 OF THE CENSUS'S 11 UNREACHABLE CELLS

`FINDING-the-crash-surface.md` ranked *"sanctioned wat-level `close'`"* third, worth "four cells":
`SendOutcome::Closed`, `TrySendOutcome::Closed`, `CloseOutcome::Signaled`, `CloseOutcome::Failed`. **It is
also the fix for this race.** One root cause, five consequences — the ranking under-valued it because the
census was counting variants, not causes.

## ⭑ The fix needs NO new primitive: the socket close IS the exit event

`stop` already holds the lineage peer. When the service process exits, the OS closes that socket, and a
`recv` on it returns `Closed` (thread lineage) or `Lost` (process lineage) — **measured** by this
campaign's own probes (`probe-crash-surface-send-closed-thread.wat` prints
`thread-recv-after-exit=Closed`; `probe-crash-surface-send-after-exit.wat` prints Lost on a process).

So: after `Status::Stopped`, **recv once more on the lineage peer and require `Closed`/`Lost`.**

```
Admin::Stop  →  owner-recv-loop  →  Status::Stopped  →  owner-recv-loop AGAIN  →  Closed | Lost  →  Stopped
                                                                               →  TimedOut       →  GaveUp
```

★ This is an **fd event, not a sleep** — `mora`'s rule, *"time is I/O; it arrives as an fd-event or it
doesn't arrive honestly."* A `sleep` here would be a guess and would race under heavier load, which is
precisely the failure being annihilated.

★ And it reuses `owner-recv-loop`, which already faces `Closed`/`Lost`/`TimedOut`/`Stopped`/`Malformed`
and is already wall-clock-bounded by the same 10 000 ms budget. **No Handle change. No new surface.**

## The one contract decision

> **`<S>/stop` returning `StopOutcome::Stopped` means the lineage is GONE** — its socket is closed, not
> merely that it acked. `Stopped` becomes a statement about the world, not about a message.

`GaveUp` keeps its meaning and gains a second cause: the ack arrived but the lineage did not close within
the budget. That is strictly more informative than today, where such a service reports `Stopped` and then
races its own caller.

## The four questions

**Obvious?** YES — *stop means gone* is what every reader already assumes; today's behaviour is the
surprise. **Simple?** YES — one extra `owner-recv-loop` call in an existing quasiquote; no new verb, no
new field, no Rust change. **Honest?** YES, and that is the whole point: the current `Stopped` asserts
something it has not established. **Good UX?** YES — the caller stops having to know that `stop` is
asynchronous, and nobody has to write a sleep.

## ⚠ What this does NOT fix, stated so it is not smuggled in

**`TrySendOutcome::Sent` still does not mean *delivered*.** A try-send to a peer that dies a microsecond
later will report `Sent`, because `Sent` means "the local cell accepted it". This stone removes the race
**`stop` creates**; it does not make an asynchronous send synchronous. Whether `Sent` should be renamed
(`Queued`?) or documented is a **separate contract question and the builder's** — named here, not taken.

## Scope

**IN:** the generated `stop` waits for lineage close · the same for `hibernate` if it shares the shape
(**measure, do not assume** — `the-owner-faces-an-outcome` found they did) · a probe that reproduces the
race **under load** and shows it gone.

**OUT = REJECTED:**
- ⛔ **A sanctioned wat-level `close'`.** It would also fix this, and it unblocks 4 census cells — but it
  is a new kernel-namespace surface and a much larger contract. This stone shows the race can be closed
  **without** it. Keep it ranked for the four cells; do not conflate.
- ⛔ **Renaming `TrySendOutcome::Sent`.** Named above; the builder's.
- **Adding the lineage to the Handle.** Not needed — the lineage *peer* is already there, and its socket
  close is the event. A reapable Process field would be the `close'` stone's business.

## Trap-doors

1. ⛔ **A quiet box cannot see this bug.** Every verification must run **under load** — the reproducer is
   8 spin loops on this 12-CPU box, 20 iterations. A green 30-run quiet suite proves nothing; I already
   collected one and it was silent.
2. **`Closed` vs `Lost` depends on the lineage kind** — thread gives `Closed`, process gives `Lost`. Both
   mean gone. Accept either; requiring one will pass on a thread and hang on a process.
3. **Do not add a sleep.** If the implementation contains a sleep or a retry count, it is the wrong shape.
4. **The budget is shared.** `stop`'s 10 000 ms covers the ack *and now* the close. Do not double it
   silently; if a real service needs more, that is a finding.
5. **`hibernate` may or may not share the shape.** Read it. The same class of one-sided fix has bitten
   this campaign five times.
6. **`wat/service.wat` is stdlib, frozen at build time** — but no Rust change means no stash-dance
   (`wat/fix.wat`'s own escape clause).
