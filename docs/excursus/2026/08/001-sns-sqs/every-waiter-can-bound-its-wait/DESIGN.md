# DESIGN — every waiter can bound its wait (the census)

Builder: *"why do we not have symmetry here?... both sides needs ot be able to time out and recover?..."*
→ *"draw the census - enumerate every waiter."*

**Drawn 2026-09-15. NOT STRUCK.** ⛔ **REPORT-ONLY. This stone bounds nothing.**

## WHY — the asymmetry is real, and it is a USAGE gap, not a substrate gap

Measured this session. Three waiters exist; two can bound their wait and one cannot:

| waiter | bounded? | how |
|---|---|---|
| a client doing **send-then-recv** on a `Peer` | ✅ | `call-by-deadline` — **`(select [peer tmr])`**, `wat/service.wat:4283` |
| an **owner** waiting on a `Thread`/`Process` handle | ✅ | `recv-by-deadline` (arc 109's fix) |
| ⛔ a **child** waiting for an unsolicited message on its `Peer` | ❌ | **nothing.** Bare `recv` — it hangs |

⭐ **The mechanism is NOT missing.** `call-by-deadline` has been racing a `Peer` against a timer `Peer`
all along — `(:wat::kernel::select [peer tmr])` at `service.wat:4283`, picking the timer's tier from
`peer-wire?` at `:4276` so the tiers match. ⚠ **And it is the ONLY place in the corpus that does it** —
zero other sites select a real peer against a timer. The pattern exists once and was never generalised.

### How the asymmetry got in

Arc 109's NOTE found `owner-recv-loop`'s `TimedOut` arm dead and named two fixes: a `recv-by-deadline`
for `Thread`/`Process` handles, or *"`select` learning to mix an owner handle with a timer `Peer`"*. It
chose the first as *"the smaller surface"*. That was right **for the case in front of it** — the owner.
Its closing sweep then checked for **unconstructable variants** and came back empty, which is true and
**does not cover unbounded waiters.** So the sweep that would have caught this was never run: nobody
enumerated *every place something waits* and asked *can it bound that wait?*

⛔ **The live consequence, at the most leveraged point in the system.** The generated `child-main`
awaits its owner's startup ship with a **bare `recv`** and writes a `RecvOutcome::TimedOut` arm that
raises *"the peer is alive and silent"*. That arm **cannot fire** — arc 109's NOTE, line 43: *"a bare
`(:wat::kernel::recv peer)` blocks forever and can never return `TimedOut`"*, and
`recv_outcome_timedout()` is called at three sites, all inside `recv-by-deadline`. ⚠ And
`recv-by-deadline` **cannot be applied there**: it rejects a `Peer` outright (*"expected owner handle
(Thread|Process), got :wat::kernel::Peer"*, measured). **So a slow owner hangs every service at
startup, and the arm that appears to handle it is dead code reading as coverage.**

## ⛔ THE ONE CONTRACT DECISION — the invariant, and what counts as a waiter

**The invariant: every waiter can bound its wait.** The census reports each waiter and whether it can.

A **waiter** is a call site of a primitive that blocks. Seven, measured registered:

| primitive | blocks on | bounded? |
|---|---|---|
| `recv` (Peer) | a message | ❌ no bounded sibling for recv-only-on-a-Peer |
| `recv-by-deadline` (Thread\|Process) | a message | ✅ **is** the bound |
| `select` (Vector Peer) | any peer ready (`message.rs:44`) | ⚠ **only if a timer Peer is in the vector** |
| `poll` (serve loop) | the owner/admin link (`message.rs:236`) | ? — report |
| `accept` (Listener) | a connection | ? — the crash-surface matrix records *"keeping Bound alive and not dialing **hangs**"* |
| `send` | thread-tier bounded(1) capacity | ? — report; `try-send` is the non-blocking sibling |
| `readln` (stdin) | a line | ❌ — hit in anger this session: no piped record ⇒ a block, then a confusing stdio `TimedOut` |

⭑ **`try-send` is the only one that is non-blocking by construction** (it has `WouldBlock`). It is the
shape of what a bounded waiter looks like, and it belongs in the report as the positive example.

### ⭐ The one test only a FORM WALKER can do

`select` is bounded **iff some element of its peer vector is an `after` call**. That is a *structural*
question about a sibling expression — invisible to any line tool. **This is the argument for the form
walker over grep, and the census must implement it**, not approximate it.

## Out of scope = REJECTED

- **Bounding anything.** Report-only. The builder rules the sequence, exactly as with the
  recoverable-crash census (2276 → 51 in scope).
- **Building the missing primitive.** A recv-only deadline on a `Peer` is the obvious fix and it is a
  separate stone — ⚠ and possibly not needed: `(select [peer tmr])` already works on Peers, so the fix
  may be a *helper over existing parts* rather than a new intrinsic. **The census informs that; it does
  not pre-empt it.**
- **`child-main`'s dead `TimedOut` arm.** It will appear in the census. Fixing it is the follow-on.
- **Probes, tests, scratch-pad.** Reported (by `home=`), never in a migration scope — a probe that
  blocks forever is a bad probe, but it is not a shipped defect.

## Blast radius

```
wat-scripts/census-*.wat   one new census (extend a COPY; do not break the two struck ones)
everything else            0. REPORT-ONLY.
```

## Trap-doors named up front

1. ⛔⛔ **A census with no controls is a number.** Two struck censuses carry theirs; this one needs a
   PRESENT/ABSENT pair **per primitive**. ⭑ The known instances give them free: `child-main`'s bare
   `recv` must be PRESENT-as-unbounded; `call-by-deadline`'s `(select [peer tmr])` must be
   ABSENT-from-unbounded. If a primitive cannot be given both, **say so** — that is STOP-1.
2. ⛔ **The `select` test must be structural, not textual.** A `select` whose timer is built in a
   `let` above it is still bounded. Classify `UNKNOWN` rather than guess, and report the UNKNOWNs.
3. ⚠ **Do not break `census-malformed-raising.wat` or `census-recoverable-raising.wat`.** Both are
   struck and cited. ⛔ And note what the first one taught: its CONTROL A was pinned to a **line
   number** that drifted and **passed vacuously for weeks**. **Pin controls to forms or names, never
   to coordinates.**
4. **Count occurrences, not lines**, and quote matches, not totals.
5. **`readln` may not be registered under `:wat::kernel::`** — the grep for it came back 0 while the
   codemods plainly use it. **Find its real path before reporting it absent.**
