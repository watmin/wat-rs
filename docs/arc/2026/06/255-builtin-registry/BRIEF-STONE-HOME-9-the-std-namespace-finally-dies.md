# STONE HOME-9 — `:wat::std::` finally dies: math, stat, seq

DRAWN 2026-08-27 against `fb0cdb192`.
**PRIOR ART:** `git log -1 315bbf546` (Stone F — the three-phase rename shape this stone DOES use,
unlike HOME-8) and `git log -1 fb0cdb192` (HOME-8, the two-layer home doctrine).

## ⛔ THE FINDING THAT OPENED THIS

**`:wat::std::` is ALIVE.** Fourteen verbs still dispatch in `src/runtime.rs` and answer:
`(:wat::std::math::sqrt 16.0)` → `4.0`. Arc 109 ("kill-std") deleted the `wat/std/` DIRECTORY and
swept the `.wat` stdlib on 2026-04-30/05-01; **it never swept `runtime.rs`'s Rust dispatch arms.**
All fourteen predate 109 (2026-04-19 → 04-25) — they are survivors, not growth.

⚠ `docs/COMPACTION-AMNESIA-RECOVERY.md`'s **FM 8 states this namespace is GONE.** It is true about
the directory and FALSE about the namespace, and it has been false since 109 closed — inside the
document whose job is to stop the next self acting on stale claims. **Correcting FM 8 is part of
this stone.**

## The moves — builder-ruled 2026-08-27

```
:wat::std::math::{ln log exp sqrt sin cos pi}   ->  :wat::math::*
:wat::std::stat::{mean variance stddev}         ->  :wat::stat::*
:wat::std::list::{zip window remove-at}         ->  :wat::seq::*    ⬅ AND MADE SEQABLE-GENERIC
:wat::std::list::map-with-index                 ->  DELETED (see below)
```

**Builder: *"`:wat::list::` was meant to be killed in favor of `:wat::seq::`."*** The reserved
`:wat::list::` name is NOT claimed here.

## ⛔ THE ONE CONTRACT DECISION — Vec-only is the ANOMALY, not the baseline

The four `list::` verbs route to `crate::collection::transform::eval_vec_*`, every one of which
opens with `require_vec` — `Value::Vec(xs) => Ok(xs), other => TypeMismatch`. They **reject a
`List`**, which is the type their own name claims. The implementer knew and wrote it down FOUR
times, verbatim (`src/collection/transform.rs:850, 888, 933, 1064`):

> *"This Rust function is named `eval_vec_zip` to mirror the ENFORCED value type: both inputs must
> be `Value::Vec`; actual `Value::wat__core__List` values are rejected at runtime."*

**That comment is the code confessing the namespace is wrong, not documenting a decision.**

And the substrate already ruled the other way everywhere else — `map`, `foldl`, `take`, `drop`,
`map-indexed`, `remove`, `take-while` are ALL Seqable-generic in `wat/seq.wat`, over
`Vector · PersistentVector · List · Stream`. **These four are the only seq ops in the language that
are not.** So making them generic is not a feature — it is removing a restriction nobody chose.
Cross-language agrees: Ruby's `zip` is on `Enumerable`, Python's on any iterable, Haskell's on
lists; nobody binds it to one concrete container.

⚠ **The orchestrator argued the opposite and was wrong.** It read intueri's scope disclaimer
(*"that's a behavior change — outside this spell's remit"*) as a PROHIBITION on changing behaviour.
That is a ward being modest about its own remit, not a finding. Do not inherit that reading.

## The four dispositions — each different, each measured

```
map-with-index  DELETE. `:wat::core::map-indexed` already is this, Seqable-generic, proven on a List.
                ⚠ NOT identical: arg order FLIPS (Vector,fn) -> (fn,coll), and the result is a LAZY
                Stream, not an eager Vector. `(Vector[10 20 30], fn)` -> `[10 21 32]` today.
                Every caller must be migrated deliberately, not sed'd.
remove-at       KEEP, PROMOTE. It is NOT a duplicate of `:wat::core::remove` — measured:
                remove-at(coll, i64) drops BY INDEX (`for (idx,v) in xs.iter().enumerate()`,
                transform.rs:942); remove(pred, coll) drops BY PREDICATE (seq.wat:373). Four shared
                letters, different functions. Clojure has no remove-at either — this is a real gap.
zip · window    KEEP, PROMOTE. No generic sibling exists. `window` is Clojure's `partition`.
```

## The population — measured, and stated with what the instrument can see

```
.wat corpus (the migration)   69 sites across 21 files
docs/**                      880 sites — NEVER MOVE
.rs                          ~146 — dispatch arms, handlers, comments
```

⚠ A per-verb count over ALL tracked files gave `remove-at` = 69, which is a docs artifact — one
`.diff` in `docs/arc/` carries 7 alone. **Re-derive from `git ls-files '*.wat' '*.wat.bad'`.**

⛔ **BOOTSTRAP EXPOSURE:** `wat/service.wat` (5), `wat/holon/Circular.wat` (7), `wat/holon/Log.wat`
(3), `wat/holon/Sequential.wat`, `wat/holon/Ngram.wat`. Retire before these migrate and every
program fails to load. **The three-phase order is mandatory here** — unlike HOME-8, this stone
DOES rename.

## ⛔ A LEVEL-1 LIE TO FIX, NOT MOVE

`:wat::std::math::log` is wired to `f64::ln` — the SAME function as `ln` (`runtime.rs:6385-6386`).
Measured: `log(100.0)` → `4.605170185988092`; `log10(100)` would be `2.0`. **Zero call sites in the
corpus.** It is a name that lies, waiting for its first caller. Options: register it as `log10`
(the base a bare `log` implies to most readers), or DELETE it as an unused duplicate. **Do not
carry the lie into `:wat::math::` — STOP-3.**

## Phase order — MANDATORY (this stone renames)

```
PHASE 1   register :wat::{math,stat,seq}::*.  BOTH SPELLINGS LIVE.  Nothing moves.
PHASE 2   corpus moves by wat-fix codemod (69 sites, 21 files).  Both still work.
PHASE 3   retire.  Delete the :wat::std:: arms + RetirementEntry rows naming the replacements.
```

`src/macros/eval.rs`'s `is_pure_total` has bitten FOUR consecutive stones. Measure it.

## Rooms

```
src/runtime.rs:6385-6394           the math arms (ln/log/exp/sqrt/sin/cos/pi)
src/runtime.rs:5836, 6117-6125     the list arms
src/runtime.rs:21018-21073         eval_math_unary + eval_math_pi (f64-committed: `fn(f64)->f64`)
                                   ⚠ its docstring enumerates FOUR callers; there are SIX.
src/collection/transform.rs:849+   eval_vec_{zip,window,map_with_index,remove_at} + the 4 confessions
wat/seq.wat:373                    :wat::core::remove — the by-PREDICATE sibling, for contrast
wat/seq.wat:75-90                  Seqable :- [T], extended by Vector/PersistentVector/List/Stream
src/intrinsic/string.rs            the home shape to copy
src/remedy/retirement.rs           the RetirementEntry rows Phase 3 owes
docs/COMPACTION-AMNESIA-RECOVERY.md  FM 8 — the false claim to correct
```

## STOP triggers — each REJECTS

1. **STOP-1 — you would keep the four seq verbs Vec-only.** Promote them; `require_vec` is the bug.
2. **STOP-2 — `map-with-index` callers are sed'd rather than migrated.** Arg order and laziness differ.
3. **STOP-3 — `log` is carried into `:wat::math::` still wired to `ln`.** Fix or delete; do not move.
4. **STOP-4 — you would claim `:wat::list::`.** Builder-ruled: it dies in favour of `:wat::seq::`.
5. **STOP-5 — retirement lands before the bootstrap `.wat` files migrate.**
6. **STOP-6 — `is_pure_total` needs an entry you did not measure.**

## Acceptance

```bash
# 1. all verbs RUN under the new spelling — a scratch-pad probe asserting a result for each.
# 2. ★ SEQABLE PROOF: zip/window/remove-at each accept a List AND a Vector. Show both.
#    Before this stone a List was REFUSED. Paste both outcomes per verb.
# 3. the old spelling is a CHECK error naming its replacement.
# 4. `:wat::std::` is GONE:  grep -rn ':wat::std::' src/ wat/ tests/ wat-scripts/  -> 0
# 5. map-with-index's callers migrated to map-indexed and still assert the same VALUES.
# 6. FM 8 in the recovery doc corrected, and dated.
# 7. cargo build --release --all-targets
```

## Report back with

Every verb's before/after. **Row 2's Seqable proof in full** — List and Vector, per verb. What
happened to `log`. Every `map-with-index` caller and how it was migrated. The wat-grep population vs
any text count. What `is_pure_total` needed. Anything this brief got wrong; what you did NOT do, why.
