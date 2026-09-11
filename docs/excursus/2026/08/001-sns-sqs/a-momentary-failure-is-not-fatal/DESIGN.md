# DESIGN — a momentary failure is not fatal

Builder, on being shown that a 1 % transport fault kills the run:
*"we induced a crash..... that's our next target... you must not be able to crash us with
momentary failues..."* — then, on being told the blast radius:
*"we do not fear becoming more correct - we have a wide range of tools at our disposal."*
And the context that sets the target: *"i'm probably 2-3 months out from killing raise entirely...
raises are placeholders until we know better."*

**Drawn 2026-09-10. NOT STRUCK.**

## Why

A single undecodable frame — momentary, on a connection that redials fine — kills two processes.
Measured, reproducible in 15 s (`the-induced-failure-rate-is-measured/FINDING.md`):

```
./target/release/wat wat-scripts/fanout/circuit.wat 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500
SCRUTINEE: queue::Queue::Reply.Failed (1 field(s))
```

The root is a **tier confusion in a type**. `Reply::Failed [cause <- :wat::kernel::Failure]` is
synthesized onto every `<S>::Reply` (`src/types.rs:3831`, `RESERVED_FAILURE_VARIANT`). It says
*"your message did not decode"* — a **transport** fact — and it lives in the enum whose every other
variant says *"here is the answer to your op"* — an **op** fact. Because it is a member of the op
reply type, every client match must consider it, and therefore **can forget it.** The macro forgets
it: `reply-failed-kw` appears at exactly two sites in `wat/service.wat` (`:2416`, `:2435`) and
**both construct it on the serve side. The macro never matches it anywhere.**

⛔ And `service.wat:1231` records the behaviour that was never built: *"the generated client method
surfaces it as an unignorable raise carrying the cause's reason."* What ships is a bare
`PatternMatchFailed` **with no cause at all** — the exact opposite. ★ An unimplemented raise is a
silent failure; an unhandled enum variant is a compile error. **The mechanism chosen to guarantee
visibility is the one that lost it.**

## The three dispositions — the taxonomy the corpus has never drawn

Every non-answer belongs to exactly one, chosen by what the caller *can do*:

| | outcomes | disposition |
|---|---|---|
| **RETRY** | `TimedOut` · `Stopped` · `Closed`-then-redial-ok | the peer is alive; the same call may succeed |
| **REPORT, FINAL** | `Malformed` | deterministic — retrying reproduces it exactly; *we* are the defect |
| **REPORT, GONE** | `Lost`-then-redial-failed | the peer is dead; there is nothing to return |

★ **RETRY vs REPORT-FINAL is the distinction that was missing, and it is why `Failed` became
fatal.** A timeout is momentary and retry is correct; a decode failure is deterministic and retry is
futile. With only one disposition available, the one chosen was death.

⛔ And the generated owner `stop` collapses the other direction: it raises on **all four**
non-`Message` arms, two of whose own panic strings say the peer is alive —
`"…the service was ALIVE…"` (`Stopped`) and `"the peer is alive and silent"` (`TimedOut`). The
`src/types.rs` declarations agree: *"DIED and NOTHING CLOSED: the peer is ALIVE and the channel is
OPEN"*. **The code states that nothing died and then kills the process.** Arc 278 built these
variants precisely to stop conflating death with silence; the handling folds them back together.

## What it delivers

### Stone 1a — `RecvOutcome` gains `Malformed [cause <- :wat::kernel::Failure]`

Purely additive. **Nothing emits it yet**, so no behaviour changes and the floor must stay green on
its own terms. 287 arm insertions across 135 `.wat` files + 7 `.rs` sites.

⭑ **The migration is a recorded precedent, not a new risk:** `RecvOutcome::TimedOut` was added the
same way, and its codemod is on disk at **`wat-scripts/fixes/add-timedout-arm.wat`** (401 lines,
form-aware, idempotent, `--grep` census + apply). Copy it. Its own header explains why a regex
cannot do this: the discriminator is a match's **arm set**, which a regex cannot see.

⚠ One rule differs from TimedOut's. `TimedOut` is **nullary**, so its codemod had to rewrite any
Lost-arm body that mentioned the binder. `Malformed` **carries a cause of the same type `Lost`
does**, so the Lost body can be mirrored verbatim including its binder — a simpler mirror, and the
one that finally delivers "carrying the cause's reason."

### Stone 1b — the failure becomes a transport frame, and leaves `<S>::Reply`

⚠ **The constraint that shapes this:** the serve loop must put something on the **wire**, and a
`RecvOutcome` is a local value, not a frame. So the reserved failure becomes a **transport-tier
frame**, recognized by `recv` and converted to `RecvOutcome::Malformed` — the same mechanism the
death notice already uses (`recv_wire` compares a sentinel). Then `Failed` is **removed** from the
synthesized Reply: it has no form in the op type, so no op match can face it and none can forget it.

★ This is the rung the builder named — *"making an illegal state not representable"* — and it is
**simpler than what ships**: it deletes a synthesized variant. It also needs **no new gate**, because
`check.rs:6879` already refuses a match that omits a `RecvOutcome` arm. A variant cannot be
forgotten; a raise can.

### Stone 2 — the owner methods stop raising

`stop` returns a bare `T`, which is *why* it must raise: the return type has no room for "asked, not
answered yet." Give it room:

```
<S>/stop : Handle -> StopOutcome<T>
  :Stopped [state <- T]
  :Gone    [cause]                          -- redial failed; there is no state
  :GaveUp  [waited-ms, last <- RecvOutcome]  -- alive, never answered within budget
```

body = a **bounded re-ask**: send `Admin::Stop`; on `TimedOut`/`Stopped` retry while budget remains;
on `Closed`/`Lost` redial, retry if the redial succeeds, `Gone` if it fails; budget exhausted →
`GaveUp{waited, last}`.

Two properties make it correct rather than merely raise-free:

- **Bounded by wall clock, never by an attempt count, and it names which bound it hit.** That is
  verbatim the gateable property already owed as ruling #1 for the three `circuit.wat` pollers.
  `stop` is a **fourth** member of that class — gate it with them, do not repair it beside them.
- **`GaveUp` carries `last`.** Losing *which* outcome ended it is how today's crash cost an hour;
  `PatternMatchFailed` could not name its own scrutinee until this session fixed it.

`hibernate` moves with it (same shape, 4 call sites). `grant` returns `nil` and matches
`peer-process`, a different shape — **measure before assuming it moves.**

Call sites, measured: **`/stop` 31 · `/hibernate` 4 · `/grant` 45.**

## The one contract decision

**`Malformed` is REPORT-FINAL, never retried, at every site.** A decode failure means the sender is
wrong; retrying transmits the same bad bytes and fails identically. Any generated or hand-written
handler that retries `Malformed` is a defect — that is the invariant a gate would hold, and it is
the reason `Malformed` must not be folded into `Lost` (which *is* retried, via redial).

## The gateable invariant, for when raise dies

> **A generated handler may not call `assertion-failed!`. Every non-`Message` outcome is placed in
> exactly one of RETRY / REPORT-FINAL / REPORT-GONE, and every report names the outcome that
> produced it.**

Checkable today — "no `assertion-failed!` in a macro-generated body" is a grep over the expansion
(`wat/service.wat` holds 43 `assertion-failed!` calls today) — and it needs **no change** when raise
is removed, because nothing conforming to it uses raise.

## Out of scope = REJECTED

- **A checker rule requiring every Reply match to face `Failed` explicitly.** Proposed earlier this
  session and withdrawn: it is rung 2 compensating for a type that is wrong. 1b deletes the variant
  instead, and the existing exhaustiveness rule does the work for free.
- **Mapping `Failed` to `Lost`.** It is the collapse this whole design exists to undo: `Lost` is
  death and is retried by redial; `Malformed` is deterministic and must not be.
- **Touching the 6 `CallOutcome::Answered` sites in `circuit.wat`.** Harness helpers, a fixture.
  They follow the substrate, not the reverse.
- **Removing `raise` anywhere else.** The builder's own 2–3 month arc. This design must not
  *add* a raise; it does not undertake to remove the rest.

## Trap-doors named up front

1. **`Failed` is guarded as a reserved name** (`src/types.rs:3831` refuses a user-declared `Failed`).
   Removing the synthesized variant must not silently un-reserve the name, or a surface could declare
   its own `Failed` and mean something else.
2. **287 arms is a cascade, and a cascade is the progress meter** — but the codemod does not rewrite
   comments, so prose referring to `Reply::Failed` is a separate manual pass. `service.wat:1231` and
   `:2558` are **both currently false** and must be rewritten regardless.
3. **`Stopped` vs `Malformed` ordering.** Arms are tried in order (`runtime.rs:16296`); a
   codemod inserting after the last arm must not land after a `_` catch-all. TimedOut's codemod
   already handles the `_` case by leaving those matches untouched — keep that rule.
