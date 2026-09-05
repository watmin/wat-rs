# SCORE — a client has a deadline

**STRUCK.** Executor: grok, 2026-09-05. Every row re-run by me.

```
Summary [ 363.494s] 5214 tests run: 5214 passed (4 slow), 15 skipped
FLOOR=0        my own run · circuit distinct=8000; dup=0; seen-dups=0 ×5
```

## ★ ROW 2 — the retry lands on a fresh peer, shown not asserted

My own runs, 3/3:

```
timeout=yes ; discarded=yes ; redial=Connected ; retry-on=fresh
```

The first peer is dropped in an inner `let`; the retry's `send` returns `Sent` on a **second**
`connect`. That was the trap that converted a death into a hang twice this campaign, and it is closed
by demonstration rather than by claim.

## ★ ROW 3 — `seen-dups = 0`, five runs, and that is row 5 passing

I wrote that row with **no expected value** because a client that times out and retries is retrying
work the server may already have done. It came back **zero**, five times, untuned.

**The deadline did not fire in health** — which is exactly what row 5 demanded of it. A deadline that
produced duplicates in a healthy system would have been a fault injector wearing a fix's name.

So: **R2 is still what moves `seen-dups`.** This stone did not steal its job; it made its job
survivable.

| # | row | result |
|---|---|---|
| 1 | ★★ silent server times out | ✅ probe green — `fast=reply; never=TIMED-OUT` |
| 2 | ★★ retry on a fresh peer | ✅ 3/3, my runs |
| 3 | ★★ `seen-dups` | ✅ **0 ×5**, untuned |
| 4 | ★ the invariant | ✅ `distinct=8000; dup=0` ×5 |
| 5 | deadline silent in health | ✅ — and `circuit.wat:1405` says why the *probe's* deadline is honest: *"Silent Hold never replies, so the server is not behaving"* |
| 6 | every `ServiceEvent` arm named | ✅ **all eight**, no wildcard: `Message` (idx 0 reply / idx 1 deadline), `Closed`/`Lost` (idx 0 redial-no-ack / idx 1 timeout), `Shutdown`/`Admin`/`Connection` assert, `Malformed`/`Rejected` split by idx |
| 7 | scope | ✅ `circuit.wat` only; `Queue/receive` still `:UpTo 250`; no `src/`, no `recv` deadline |
| 8 | the floor | ✅ **5214/5214, my run** |

Exhaustion reports `depth=3; attempts=3; elapsed={ms}` — not "gave up."

## ⛔ THE DELTA IS A SUBSTRATE FINDING — the fifth locus asymmetry

> *"A process impl cannot call a same-file `defn` (`UnknownCallee :fanout::redial-seen`). Thread was
> fine."*

A **userland** top-level `defn` is not reachable from a **process-locus** service impl, though it is
from a thread-locus one. The stdlib's own `send-keep-serving?` (R1) *is* reachable from the generated
loop in a child — so this is about userland files crossing the fork, not about `defn` as such.

The workaround: the helper became a **typed local `fn` inside `-tick`**. And a `ClaimWait :Impure`
record holding a `Peer` failed the same way, while a local
`(Tuple peer (Option resp) i64)` type-checks across the fork.

★ **Fifth locus asymmetry this campaign** — after the duration-0 timer, `Closed`-vs-`TIMED-OUT`, the
coerce arms green at thread, and the frame cap not tearing at thread. **S37.** Five instances is a
property of the substrate, not a run of bad luck, and every one of them was found by a strike rather
than by reading.

## ⚠ ONE THING I WOULD WANT CHECKED

Row 2's proof is `:user::deadline-redial-is-fresh` — a wat function, wired into the circuit's `main`.
`cargo nextest -E 'test(redial_is_fresh)'` finds **no test**. The `wat_scripts_fixes_load` gate
type-checks it and never runs it, which is the gap `dfacde23c` already recorded: *"a future edit that
breaks the flag goes GREEN on the floor."*

**The freshness proof may not be floor-gated.** If it is not, it will rot silently — and it is the
one row that closes a failure mode this arc paid for twice. **S38.**

## Still open

**R2** — the drop in `send-keep-serving?`. A client can now survive it: it will time out, discard,
redial, and retry, and *that* retry is what moves `seen-dups`. The two stones compose exactly as
drawn.
