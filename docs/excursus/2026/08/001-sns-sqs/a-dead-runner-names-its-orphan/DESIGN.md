# DESIGN — a dead runner names its orphan

Builder: *"Minimal I guess... Queue is stdlib long term..."* — choosing **(a-minimal)** after the
load-order finding closed the queue-backed route for now.

**Drawn 2026-09-15. NOT STRUCK.** ⛔ **NO CONTRACT CHANGE.** `brackets/map` still fails on a dead
runner. It stops failing *anonymously*.

## WHY — measured, and the probe confirmed the reading this time

`wat-scripts/scratch-pad/probe-a-dead-runner-orphans-its-item.wat` (committed). A work fn that raises
on item 3, so a runner dies **holding a real item** — userland cannot kill a lineage (arc 259 S2d), so
this is the reachable death, and it is exactly the remote-host case:

```
clean=[0 2 4 6 8 10 12 14]                                control — a clean map returns
AssertionFailure "runner: dying on item 3"                the runner dies
AssertionFailure "bracket collect-loop: runner 0 crashed: …"   the map dies with it
```

**It raises; it does not hang.** ⭑ Three outcomes were possible — raise, hang, short answer — and they
are three different stones. The probe says which.

⛔ **And the failure is anonymous.** `collect-loop`'s entire state is
`[peers items pairs-acc cursor collected m]` (`wat/bracket.wat:594`). **Nothing records which runner
holds which item.** The dispatch advances the cursor *"regardless of outcome"* (its own comment,
`:611`). So the message can say *runner 0 crashed* and can never say **which item died with it**.

★ That is why the raise is there, and it is honest: `collect-loop` **cannot recover by construction**.
Not knowing what to re-send, its only alternatives are to hang or to return a short answer silently.
**Removing the raise without the ledger would produce one of those two** — the same trade nearly made
on the publisher.

## ⛔ THE ONE CONTRACT DECISION

**Record `peer-pos → item-index` at dispatch, and name the orphan in the failure. Do not re-dispatch.**

The three arms still raise. The message changes from *"runner 0 crashed"* to *"runner 0 crashed holding
item 47"*. Nothing about `brackets/map`'s promise moves: it is still exactly-once-or-die.

⭑ **This is the prerequisite for every future option, and it is the only part needing no ruling.**
Re-dispatch (at-least-once, an item may run **twice**) is a public-surface change and the builder's to
rule. A queue-backed map is blocked on load order (below). Both need the ledger first; neither is this
stone.

## Why the queue route is not "later", it is BLOCKED — and promotion does not fix it

```
wat/spawn.wat   171     declares :wat::bracket::PoolMsg (:227) and a PoolMsg-typed peer (:412)
wat/bracket.wat 177     ← brackets loads HERE
wat/service.wat 341     the defservice macro
wat/query.wat   444     the Store contract
```

A `wat/queue.wat` is a `defservice` that `:peers [:wat::query::Store]`, so it could only load at
**≥473** — ~300 positions *after* brackets. **A stdlib file cannot name a type declared after it**
(`the-gate-outcome-outlives-its-file`). ⚠ `sqs.wat` is dependency-clean (it names only `:wat::` and
`:queue::`), so **promotion is the easy half and it does not touch the hard half.** Moving brackets
later is blocked too: `spawn.wat` pins the brackets *vocabulary* six positions ahead of it.

⭑ The queue belongs in the stdlib for its own reasons — it is the most chaos-hardened thing we own —
but it would **not** deliver `brackets/map`, and ruling it on that basis would rule it on a false
premise.

## Out of scope = REJECTED

- **Re-dispatching the orphan.** At-least-once, an item runs twice, public surface. The builder's ruling.
- **Making `brackets/map` queue-backed.** Load-order blocked; see above.
- **The other two arms' wording.** `Closed` and `Malformed` get the same orphan naming — they have the
  same blindness — but their *dispositions* are unchanged and deliberate (`:638`–`:641` argues Malformed
  loudly, and it is right to).
- **Anything outside `wat/bracket.wat`.**

## Blast radius

```
wat/bracket.wat   collect-loop's state + the 3 arms' messages (:626 :632 :642)
elsewhere         0 expected — VERIFY. ⛔ bracket.wat is STDLIB, frozen into the binary at
                  build time: read wat/fix.wat's BOOTSTRAP/STASH-DANCE header before editing.
```

## Trap-doors named up front

1. ⛔⛔ **The `AssertionFailure`'s reported location is WRONG.** The probe's trace says
   `bracket.wat:624` for a raise that lives at **`:633`** — `:624` is the tail recursion. **Do not
   navigate by the location in a failure; navigate by the message.** ⚠ Second time today a line
   coordinate in this corpus proved unreliable (the other: D5's CONTROL A pinned to a line that
   drifted, passing vacuously ever since).
2. ⛔ **The ledger must not become a fourth slot on a nested Tuple.** `collect-loop`'s params are
   already six wide; `the-waiter-folds-carry-a-named-aggregate` closed exactly this shape. A named
   aggregate or a parallel vector indexed by `peer-pos`, and say which.
3. ⚠ **`peer-pos` is a select index, not a stable runner id.** `select` returns a position into
   `peers`; if that vector is ever compacted, positions shift. Verify it is stable for the pool's
   lifetime, or key the ledger on something that is.
4. **Do not silence the raise.** Removing it without re-dispatch turns a loud failure into a hang or a
   short answer — measured above as the only alternatives.
5. **`cargo build --release` does not compile tests.**
