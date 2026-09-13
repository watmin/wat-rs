# DESIGN — the store injector has teeth

Builder: *"you've named two things to work on... so... let's work on them."* **Second of the two.** The first
(`be945ff12`) gave the floor its ~30 s back; this spends some of it.

**Drawn 2026-09-13. NOT STRUCK.**

## Why — D2 applied to the other injector

`the-chaos-gate-has-teeth` (`bb993ffd5`) made an inert **disrupt** injector a red. The **store-fault** injector
got no such gate:

```
grep -rln faulting-store tests/**/*.rs   →  nothing
```

So `wat-scripts/query/faulting-store.wat` — the proxy that loses a store reply **after the write lands** — is
exercised only by a hand-run scratch probe. ⛔ **And it guards this session's most load-bearing invariant:**
D1-c's `Accepted n == real-rows`, the whole content of `04726854e`. Nothing automated holds it.

★ `the-dial-declares-its-peer`'s DESIGN already named this gap, and `accepted-is-a-count`'s SCORE named it
again. It is the third time it has been written down; this is the stone.

## ⭑⭑ THE INSTRUMENT ALREADY EXISTS AND IS ALREADY HARNESSABLE

`wat-scripts/scratch-pad/probe-accepted-is-a-count.wat` is not a print-only script — it has **three zero-arg
entry points that RETURN Strings**, with `:user::main` merely printing them:

```
:sf::run-passthrough  []  -> String      rates 0
:sf::run-die          []  -> String      die-bp   10000
:sf::run-timedout     []  -> String      drop-reply-bp 10000
```

⭑ That is exactly the shape `probe_ex001_fanout.rs` and `probe_chaos_gate_has_teeth.rs` consume
(`startup_from_source` + `FsLoader`, `apply_function`, a `field(summary,key)` splitter). **No change to the
probe is needed** — unlike D2, where `:user::chaos` returned `nil` and had to gain a returning sibling.

## ⛔ THE SIZING DECISION, and it is the one D2 taught

The probe as one program takes **20.7 s measured** — the `drop-reply` path alone waits the full **10 s** client
deadline (probed in `NOTE-a-callee-deadline-ms-is-inert.md`: a callee's `:deadline-ms` is inert, so 10 000 ms is
not shortenable). Against the floor's **40 s** per-test wall that is **1.9× headroom**.

⚠ **This floor already lost a test to that wall** (`.floor/2026-09-13T04-00-06Z/`: a 24 s test timed out at 40 s
under `nice -n 19` contention). 20.7 s under contention is the same shape.

> **So: THREE tests, one per entry point** — not one 20.7 s test. Each lands ~3 s / ~7 s / ~17 s, and nextest
> runs them in parallel, so the wall cost is the largest not the sum.

★ **State each runtime against the wall, not in seconds alone.** That is D2's second lesson, and the reason
that stone's SCORE could report a timeout honestly instead of hiding it.

## The relations — no probability anywhere

`drop-reply-bp` and `die-bp` at **10000** are 100 %: every roll fires. Nothing here is a dice throw, so every
assertion is an equality or a deterministic consequence.

| test | asserts | why it cannot flake |
|---|---|---|
| passthrough | `Accepted n`, **`n == real-rows`** | rates 0; the store answers; both are 1 |
| ⭑⭑ **drop-reply** | **`n == real-rows`** and `drops-fired > 0` | 100 % ⇒ the reply is always destroyed; the scan recovers the true count. **This is D1-c's invariant.** |
| die | client sees **`TimedOut`**, and **`real-rows == 1`** | the proxy forwards then exits: the write lands, the answer cannot come |

⭑ **`n == real-rows` is the row worth having.** It is an equality between what the queue *says* and what the
store *holds* — it cannot be satisfied by a lucky number, and it is exactly what `Accepted 0` used to violate.

## The one contract decision

> **The gate asserts the INVARIANT (`n == real-rows`), never the mechanism's counts.** `drops-fired` appears
> once, as `> 0`, to prove the fault actually fired — the `disrupt-hits`-was-pinned-at-zero lesson. Everything
> else is an equality.

## The four questions

**Obvious?** YES — the proxy exists, the probe returns Strings, the harness shape is on disk twice.
**Simple?** YES — one harness file, three tests, no change to the probe or the proxy. **Honest?** YES: it gates
the invariant this session's biggest stone rests on, and it states its runtimes against the wall rather than in
seconds. **Good UX?** YES — `Accepted n == real-rows` can no longer silently regress.

## Scope

**IN:** one harness under `tests/services/` · three tests, one per entry point · the relations above · each
runtime **stated against the 40 s wall** · the floor count restated (it is **5241**, and this will move it).

**OUT = REJECTED:**
- ⛔ **Changing the probe or the proxy.** Both are already the right shape. If a change seems needed, that is a
  finding.
- ⛔ **One combined 20.7 s test.** Named above; 1.9× headroom on a wall this floor has already hit.
- ⛔ **Asserting `drops-fired` equals a count.** `> 0` only — a count would pin a mechanism detail and this
  campaign has four errors from gating observations.
- **Shortening the 10 s wait.** Measured impossible from the callee (`NOTE-a-callee-deadline-ms-is-inert.md`);
  a caller-side deadline could, but that would change what the probe measures.

## Trap-doors

1. ⛔ **20.7 s in one test.** Split it. The wall is 40 s and contention has beaten a 24 s test here.
2. ⛔ **`5241` is the floor count**, not 5237 or 5239. Three stones have moved it; this makes four.
3. **The proxy's `die` path leaves a dead process.** Expected — it exits after forwarding. Do not "fix" it.
4. **`real-rows` is read from the REAL store**, not the proxy. That is the point: it is the independent witness.
5. **`wat-scripts/**/*.wat` is type-checked by the floor**, so leave the probe loadable.
