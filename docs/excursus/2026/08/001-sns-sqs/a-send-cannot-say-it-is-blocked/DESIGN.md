# DESIGN — a send cannot say it is blocked

**Drawn 2026-09-15. NOT STRUCK, and it must not be struck without a builder ruling** (§THE RULING
OWED). Sibling of `a-wait-that-should-be-bounded` / `both-ends-bound-the-same-handshake`, which closed
the `recv` half of the builder's invariant: *"both sides needs ot be able to time out and recover"*,
*"no one is allowed to crash on a recoverable error."*

## Why now: the recv half is CLOSED, so `send` is the surface

`a-wait-that-should-be-bounded/FINDING-the-classification.md` read all 23 live bare-`recv` sites in
context: **4 BOUND** (all four struck — `7686bea24`, `ab419aaa3`), **11 TIMER**, **7 PARK**, **1
UNKNOWN** (`recv-all-loop`, whose bound is a return-contract ruling). **Zero BOUND sites remain.**

The waiter census re-run at `ab419aaa3` gives the next surface:

```
send yes=0  no=215  UNKNOWN=0        ← corpus-wide
live (wat + wat-scripts/service) send sites = 32
try-send yes=5                        ← the ONLY send-side bound in the corpus, and its bound is ZERO
```

## ⭐ THE FINDING THAT MAKES THIS DIFFERENT FROM THE RECV HALF — MEASURED, NOT ARGUED

`wat-scripts/scratch-pad/probe-a-full-inbox-blocks-send-forever.wat`, process tier:

| t | output |
|---|---|
| ≈2 s | `full-at=283` — the inbox is durably full |
| 5 / 10 / 20 s | still one line — **the plain `send` is BLOCKED**, ≥ 38 s |
| at the kill | `plain-send=Stopped` — it returned **only** because `timeout`'s SIGTERM killed the service |

⛔ **`SendOutcome` is `Sent | Closed | Lost | Stopped`. No variant describes "blocked / not draining".**
Every recv site the last two stones touched already had a `TimedOut` arm written and unreachable — arc
109's *painted brick*, `NOTE-an-outcome-variant-no-primitive-can-construct.md`. **The send side has no
arm at all.** The value the blocked site finally observed names the peer's **death**, so a caller cannot
distinguish *"I blocked 38 s and then it died"* from *"it was already stopped"*. That is not a usage gap
the way the recv asymmetry was; it is a **missing form**.

⚠ **And the first fixture refuted itself — record it, because it will mislead the next reader too.**
Fixture 1 reused `probe-crash-surface-try-send-wouldblock.wat`'s service (a handler that returns
immediately without replying). `try-send` reported `WouldBlock` at **n=376** and a plain `send`
immediately after returned **`Sent`**, in 1 s total. The reason: a `defservice` serve loop **does**
drain — it recvs every Op and dispatches; what that handler never did was *reply*.
⭑ **`TrySendOutcome::WouldBlock` does NOT imply the receiver has stopped draining.** It can be
transient socket pressure. Only a handler that **parks** (fixture 2) makes the inbox durably full.

⚠ One asymmetry in the *other* direction, kept because it bounds how bad this is: the blocked `send`
**did** return when its peer died, where a blocked bare `recv` ignored SIGTERM for 125 s.

## ⛔⛔ THE RULING OWED — three mechanisms, and none is a code-reading

A bounded `send` **does not exist in any form**. The corpus offers no shape to copy, which is why this
DESIGN stops here instead of naming an implementation:

| # | mechanism | cost |
|---|---|---|
| a | **retry-loop around `try-send` + wall clock** (wat only, no `src/`) | busy-waits; and `WouldBlock` is proven NOT to mean "not draining", so the loop's exit condition is measuring the wrong thing |
| b | **`send-by-deadline` primitive** + a new `SendOutcome::TimedOut` | reaches `src/`; a new variant on `SendOutcome` forces an arm at **every** send match in the corpus — 215 sites |
| c | **`SendByDeadlineOutcome`**, a separate enum, leaving `SendOutcome` alone | no churn on the 215; two enums for one operation, which is the drift `both-ends-bound-the-same-handshake` was struck against |

⭑ (b) is the honest shape and the expensive one; (c) is cheap and duplicates; (a) needs no ruling and
may not work. **This is the builder's call**, exactly as `RecvOutcome::Stopped` (325 arms) is.

## What this stone IS, if struck as drawn: a classification, REPORT-ONLY

The mirror of `a-wait-that-should-be-bounded`: read all **32 live** `send` sites in context and class
each **BOUND** (a blocked send would hang something that should report) / **PARK** (blocking is the
loop) / **REPLY** (a serve-loop answer — dropping it is worse than blocking) / **UNKNOWN**. No `.wat`,
no `.rs`. The classification is what makes the ruling above cheap to take; taking it first would be
choosing a mechanism for sites nobody has read.

```
wat/service.wat      17   the generated serve loop + client methods + call-by-deadline's own send
wat/bracket.wat       8   map/reduce runners and collect-loop
wat/spawn.wat         2   both ends of the handshake just bounded on the recv side
wat/test.wat          1   run-thread
wat-scripts/…         4   circuit 2 (deadline-redial-is-fresh) · sqs 2 (park-receive!)
```

⚠ `wat/spawn.wat:527` and `:597` are the **send halves of the very handshake `ab419aaa3` bounded on the
recv side** — `launch` sends the ship, then waits. Bounding the wait and not the ship is half a fence.
Start there.

## Trap-doors named up front

1. ⛔ **Do not reuse fixture 1.** `WouldBlock` without a parked handler is transient; a stone that
   builds on it will measure nothing and report a bound that isn't.
2. ⛔ **Do not pipe a blocking probe to `tail`/`head`.** It hid the whole first measurement: no output
   is indistinguishable from a block. Redirect to a file and read it WHILE IT RUNS.
3. **`try-send`'s 5 `yes` rows are not 5 bounded sends** — 4 are scratch-pad probes; the only live one
   is `wat/service.wat:2581`, the serve loop's reply.
4. **Count with the census, not by hand.** `wat-scripts/census-waiter-bounds.wat` prints bounded sites
   by name; re-run it rather than adjusting its numbers (it moved 679 → 681 this session).
5. **A `send` that blocks is not a crash.** It is the same class as the recv work — unkillable-by-SIGTERM
   hang — and the invariant that covers it is the builder's *"both sides"*, not a crash census.

---

# ⭑ THE FOUR QUESTIONS, RUN 2026-09-15 (builder: *"four-questions"*) — and they MOVED the ruling

Two things changed by running them. Both are corrections to the table above, left in place rather than
edited away so the reasoning is auditable.

## The measurement the questions needed first: the FOUR BLIND SITES

A paren-balanced reader over all **32** live `send` sites (not a fixed window — the first attempt used
one and reported 0 wildcards everywhere, repeating §0's own defect in
`a-wait-that-should-be-bounded/FINDING-the-classification.md`, because `(_` at END OF LINE is the shape
`call-by-deadline` uses):

```
exhaustive SendOutcome arms   28   ← a new variant breaks these LOUDLY: the compiler finds them
wildcard `_` arm              2    wat-scripts/fanout/circuit.wat:3819 · wat/service.wat:4281
outcome NOT MATCHED at all    2    wat-scripts/queue/sqs.wat:2059, :2064  (park-receive!)
```

⛔ **`wat/service.wat:4281` is `call-by-deadline`'s own send** — the single most important send site in
the corpus — and it is one of the two wildcards. `sqs.wat:2059`/`:2064` discard the `SendOutcome`
entirely, so a blocked send there is invisible **by construction**.

## Q1 Obvious · Q2 Simple · Q3 Honest · Q4 Good UX (in order; UX is the tiebreaker, not load-bearing)

| | (a) try-send retry loop | (b) `send-by-deadline` + `SendOutcome::TimedOut` | (c) separate `SendByDeadlineOutcome` |
|---|---|---|---|
| **Obvious** | ✅ with a caveat — a reader must not read ONE `WouldBlock` as "stuck" | ✅✅ the exact mirror of `recv-by-deadline`; anyone who read the last two stones knows it | ⚠ two enums for one operation; a reader must stop and ask why |
| **Simple** | ✅ three existing atoms, no new form | ✅ two atomic pieces (primitive, then codemod) — and **28 exhaustive sites mean the COMPILER finds them**, the one technique that worked all session; exception NAMED and bounded at 2 | ✅ diff-simple, concept-complex |
| **Honest** | ✅ **CORRECTED — see below** | ✅✅ repairs the **missing form**; the variant is primitive-reachable → satisfies arc 109's refined doctrine | ⛔ **FAILS** — leaves `SendOutcome` unable to say "blocked" while a sibling can |
| **Good UX** | one helper-local outcome; does not generalise | ✅ one enum per operation, symmetric with recv | moot (failed Q3) |

⛔ **(c) is OUT on Q3, not on taste.** Obvious + Simple + Honest must hold before UX matters.

### ⚠ THE CORRECTION Q3 FORCED — this DESIGN's own §THE RULING OWED was wrong about (a)

The table above says (a) *"busy-waits; and `WouldBlock` is proven NOT to mean 'not draining', so the
loop's exit condition is measuring the wrong thing."* **The second half of that is false**, and it
conflates two claims:

- ONE `WouldBlock` does not mean the receiver stopped draining — that is fixture 1's measured negative,
  and it stands.
- *"Still `WouldBlock` after N ms"* **is** precisely the observation wanted. A deadline loop over
  repeated `WouldBlock` is a sound bound, not a wrong measurement.

⭑ And (a) has a property (b) does **not**: `try-send` guarantees **nothing was sent** on `WouldBlock`, so
it cannot leave a surplus frame on the wire — the desync hazard that ruled out re-ask in
`the-gate-methods-face-an-outcome/DESIGN.md`. (a) is honest. It loses to (b) on Q1/Q4, not on Q3.

## ⭐ THE SECOND-LEVEL DECISION: SPLIT. (b) is the mechanism and it must NOT GO FIRST.

1. **Does a stepping stone make the next step more tractable?** YES. The 32-site classification is
   report-only and tells us how many sites want a bound **at all** — before anyone pays 215 arms for it.
2. **Is there a dependency that must land first to be ERGONOMIC?** YES, and it is the sharp one: **the 4
   blind sites must face their `SendOutcome` BEFORE the new variant lands.** Otherwise `TimedOut` arrives
   into two wildcards and two discards and is **silently ignored at the site that matters most**
   (`call-by-deadline`). ⭑ That is the painted brick arriving from the OPPOSITE direction — not a variant
   nothing constructs, but a variant nothing **reads**. Arc 109's doctrine covers the first and is silent
   on the second; this is the refinement that sweep should have asked for.
3. **Complexity composition?** Split — each piece verifies on its own.

**Recommended order:** classify the 32 → make the 4 blind sites face the outcome (small, no new form) →
then (b) with its codemod.

## What the four questions did NOT decide

`recv-all`'s return contract (`a-wait-that-should-be-bounded/FINDING-the-classification.md` §6, amended).
It needs to carry a **partial `acc`** alongside the timeout, so it is not this variant and not this
stone — a third outcome shape, or a new `LociDiedError` variant in an enum whose own comment
(`wat/spawn.wat:651`) admits it has outgrown its name.
