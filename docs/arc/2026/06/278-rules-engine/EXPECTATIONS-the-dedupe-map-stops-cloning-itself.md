# EXPECTATIONS — the dedupe map stops cloning itself

Written **before** the strike. This one is a **fix**, so the curve is the gate — and the curve can
refute the model.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **the slope falls** | n=500 / 1000 / 2000 fill-first | `drain`/pair growth **below +65 %** (today's) |
| 2 | correctness untouched | same runs | `distinct` = n×m and `dup=0` at **every** depth |
| 3 | dedupe semantics identical | no-args + n=2000 | `seen-recorded` and `seen-skipped` unchanged |
| 4 | the type changed | `grep -n 'claimed <-' circuit.wat` | `PersistentMap`, not `HashMap` |
| 5 | all sites moved | `grep -n 'hashmap::' circuit.wat` around `:fanout::seen` | **no `:wat::hashmap::`** left touching `claimed` |
| 6 | nothing else moved | `git diff` | **only** the `:fanout::seen` service; `:2075`/`:2082`/`:2592`/`:2680`/`:2784`/`:2841` untouched |
| 7 | no-args unchanged | `… circuit.wat` | every field identical |
| 8 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 9 | scripts load | `every_wat_scripts_file_loads` | green |
| 10 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **Row 1 is the stone, and it is allowed to fail.** If the slope does not move, the clone was not
the term, the model is dead, and that is a result. STOP-7: report it; do not reach for a second
change to rescue it.

⚠ **No claim that the curve goes flat.** The fit attributes ~60 % of the n=2000 growth to a
linear-in-N term. A partial improvement is the honest expectation, and quantifying the remainder is
the next question, not this stone's failure.

⚠ **Row 2 is the one that must never bend.** `distinct` and `dup` are the dedupe's correctness, and
this changes the data structure underneath it.

## Runtime prediction

**20–30 minutes.** One type in three places, two verb prefixes, one service, one file. The floor is
the long pole.

## Trap-doors named in advance

- **The map type is named TWICE in the fold's `Tuple`** (`:163` parameter and `:166` return).
  Changing one and not the other type-checks nowhere and is the likeliest half-edit.
- **`:wat::map::` is the PERSISTENT one.** The unmarked name is the persistent flavour by the
  builder's ruling (`src/intrinsic/hashmap.rs:7-12`); `:wat::hashmap::` is the copying one. It reads
  backwards if you expect "hashmap" to be the plain choice.
- **`get` moves too**, not just `assoc`. A `map::assoc` feeding a `hashmap::get` is a type error, but
  a missed `get` on a path that still has a `HashMap` is not.
- **Do not "tidy" the neighbouring folds.** `:2075` is the same defect and is a separate stone;
  touching it makes row 1 unreadable.

## What this stone does NOT claim

⚠ **It does not fix the class.** `circuit.wat:2075` (the `distinct` fold, in `collect`) and
`wat/query/mem.wat:165 :182 :193 :202 :224 :237 :262 :275` (the mem-store's indexes) have the **same
O(N²) shape**. Both are named, neither is touched — `wat/` is the builder's call.

⚠ **It does not explain the rest of the slope.** If row 1 passes, the remainder is the next question.

★ And if row 1 fails, this arc has its **tenth** dead mechanism — read off the substrate source
rather than guessed, which is a better class of wrong than the previous nine.
