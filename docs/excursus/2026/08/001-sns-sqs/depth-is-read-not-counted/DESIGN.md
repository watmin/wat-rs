# DESIGN — depth is read, not counted

**The queue's `visible` / `unacked` stop being maintained and start being derived.**
Scope: `wat-scripts/queue/sqs.wat` only.

## WHY — the counters cannot be correct, and this is not a missing update site

`take` (`sqs.wat:161-166`) scans `by-visible-at` for `isk <= now`. **A never-received row and an
expired lease are the same thing by construction** — correct SQS semantics, and a clean model:
a row's `isk` IS its visible-at instant, and the entire visible/unacked distinction is
`isk <= now` versus `isk > now`.

★ So **visibility expiry is not an event.** A message becomes visible because the clock moved;
no code runs. A counter can only be updated by code that runs. The two `i64`s are therefore
mirroring a transition that never executes — a category error, not a bug to patch.

Measured, `probe-depth-derived-from-the-index.wat`:

```
took=1
LEASE-LIVE     derived=[2/1]  counters=[2/1]  agree=yes
LEASE-EXPIRED  derived=[3/0]  counters=[2/1]  agree=NO
```

The counters are **right while the lease is live**. That is why this survived.

★ And `sqs.wat:604` already carries the confession: `f1 = if f0 <= 0 then 0 else f0 - 1`.
Someone knew the counter could go negative and clamped the symptom.

## WHAT IT COSTS TODAY — two hangs, one root

Drift inflates `unacked`, and both consumers of the pair break:

| consumer | site | failure |
|---|---|---|
| the drain | `circuit.wat:789` — `visible == 0 AND unacked == 0` | unsatisfiable after any redelivery → `drained-never … elapsed=63565` |
| **the cap gate** | `sqs.wat:256`, reported at `:439`/`:468` | `never-accepted; depth=744; cap=64` |

★ **`depth=744` against `cap=64` is impossible for real rows** — the gate refuses sends above
the cap, so depth can never exceed it. That number is pure drift, and it means the leak
**falsely closes the queue to new sends.** Two of this campaign's unexplained hangs, one cause.

## THE ALGORITHM

Two range scans on the index the queue already maintains and already queries:

```
visible = |rows of q with isk in [0, now]|
total   = |rows of q with isk in [0, +inf)|
unacked = total - visible
```

## ⛔ THE ONE CONTRACT DECISION

**The two `:ephemeral` fields are DELETED, and every depth question is answered by counting the
index at the moment it is asked.** Not "kept and also corrected" — kept-and-corrected leaves the
class alive, and there is no event at which to correct them.

The scan passes **`limit = cap + 1`**, never `cap`: a queue holding more rows than its cap is a
real defect, and a limit of exactly `cap` would silently truncate it into a correct-looking
number. `cap + 1` makes the overflow *visible*. No silent caps.

## WHY IT IS AFFORDABLE — the builder's argument, written down

A `defservice` is a **serializing actor**. An arm runs to completion before the next message is
dispatched, so the arm may do both scans and reply with no window in which anything changes
underneath. There is no atomicity problem to solve; there never was.

## ⚠ THE TRAP DOOR, NAMED BEFORE THE STRIKE

**The cap gate is on the SEND path, and send is hot** — the circuit publishes 8000 of them.
Today the gate is two field reads; after this it is a store round-trip. That is the one place
this stone can cost real time, and the EXPECTATIONS carry a five-run publish row against a
recorded baseline for exactly that reason.

**If publish regresses materially, that is a FINDING to report, not a reason to keep the
counters** — a fast wrong number is not a substitute for a slow right one. The fallback, if it
comes to that, is a decision for the builder, not for the executor.

## FILES

`wat-scripts/queue/sqs.wat` only.

## OUT OF SCOPE = REJECTED

- **The consumer stranding** (`circuit.wat:491` — a `Dup` emits no outcome). A real, separate
  defect. It is not this stone and must not be repaired inside it, or the number this stone
  produces cannot be attributed.
- **Touching `circuit.wat`'s drain condition.** `visible == 0 AND unacked == 0` is *correct*
  once the values are true. Changing both sides at once means neither is demonstrated.
- **`wat/query/mem.wat`, the store.** The index already exists and already answers this.
- **A count operation on the Store surface.** Scanning and counting is enough at `cap`-bounded
  depth; a new surface verb is a bigger stone with its own justification.
- **The nextest override** added for the drop cells. It has its own expiry, tied to this stone
  landing — remove it *after* this is green, not inside it.
