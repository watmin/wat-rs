# UNADOPTED — capabilities we BUILT and then did not USE

> **Why this file exists (builder, 2026-08-01):** *"we forgot to actually use several of our
> solutions and i have to force them through."* Three times in one day the answer was already on
> the disk and had to be dragged in by hand. That is not three lapses; it is one **class**, and the
> class has no owner until it has a file.

## The class

A capability is **built** when it compiles, passes its differential, and has a doc. It is
**adopted** when something that is not its own test calls it. The gap between those two is
invisible: the tests are green, the doc is written, the arc closes — and nothing uses it.

Worse, an unadopted capability is **unproven in the way that matters**. `ALIVS ARGVIT` — the real
consumer is the probe. A verb exercised only by its own differential has been proven consistent
with itself, never proven useful, and never driven into the corner where its design is wrong.

**This is NOT a death list.** `[[feedback_no_consumers_does_not_mean_dead]]`: zero consumers is not
evidence of deadness, and the tiebreaker is a *ruling on what the thing IS*, which is the builder's.
Every row here needs a disposition — **adopt**, or **record why not** — never a silent deletion.

## The ledger

| capability | built | adopted? | disposition |
|---|---|---|---|
| `insert-all` / `insert-all'` (batch stage, native, one rebuild not N) | before 2026-08-01 | **NO — zero of nine grid axes called it**, 38 per-fact `insert` sites | **ADOPTED 2026-08-01** (`ee4bfcc7`) after the builder forced it. Perf was small (single-digit ms — the axes stage thousands of facts, not hundreds of thousands); the *finding* was that it was unused. |
| `into (Vector<T>, PersistentVector<T>)` — the mirror clause | flagged owed 2026-08-01 (`a5d868b4`) | **NO** — forgotten within the hour, then tripped `probe-derive-decomposition.wat` because `query-by-type-string` returns a PV | **ADOPTED 2026-08-01** (`e12acc8d`) — and it turned out to be the fix for the quadratic drain. The owed item WAS the solution. |
| batch **derive** ("FIX THE HARNESS (batch seed/derive)") | prescribed in the record **2026-07-04** | **NO — sat for a month.** Both halves named; neither done | seed half adopted `ee4bfcc7`; derive half solved differently by `e12acc8d` (the O(n) drain) |
| `Vector/extend` (native one-shot, either source kind) | 2026-08-01 `e12acc8d` | **YES** — `stream->vec` is its consumer, immediately | — |
| `PersistentVector/concat` | 2026-08-01 `a5d868b4` | partially — `into`'s PV clause; PV×PV path exercised only by its own test | **OWED**: `into` still has no PV×PV clause, so that arm has no consumer but its test |
| coverage gate (`docs/COVERAGE-RUNE.md` + `scripts/coverage-gate.sh`) | arc 252 | **NO — specified, built, never run.** Zero `rune:coverage` across all nine `src/rete/*.rs` | **OPEN** — both checklist boxes still unticked |
| the promoting map (array→trie at 8) | **not built** — third door named 2026-08-01, shut | n/a | **OPEN** — see the seam; carries a hard `Hash`/`Eq` constraint |

## How to detect the class, rather than remember it

Remembering is what failed. The detectable form:

1. **A public verb whose only callers are its own tests is UNADOPTED.** Grep-able: for each
   `defn`/`defclause` in `wat/`, count call sites outside `wat/<its own file>` and `wat-tests/`.
   `insert-all` would have shown zero and been caught the day it landed.
2. **An "OWED" line in a commit message or design doc with no task is a line that will be lost.**
   The `into` mirror clause was written down *in the commit that created the gap* and still went
   missing for an hour. Owed ⇒ a task, immediately, or it is not owed, it is forgotten.
3. **A prescription in the record with a date and no commit is the same defect at a longer scale.**
   "FIX THE HARNESS (batch seed/derive)" carried a date, an imperative, and a month of silence.

The lint in (1) is the extirpare rung worth building: a *check* beats the *convention* of
remembering. It must report an inventory needing dispositions — never auto-fail, never propose a
deletion, because `no_consumers_does_not_mean_dead`.

## The rule this file encodes

**A stone is not done when it is green. It is done when something that is not its own test calls
it.** If nothing does, the closing note says so out loud and names the consumer that will — or
records why there is none.
