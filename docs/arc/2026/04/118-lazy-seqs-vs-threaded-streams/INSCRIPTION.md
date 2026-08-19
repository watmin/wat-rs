# Arc 118 — Lazy seqs vs threaded streams — INSCRIPTION

**Status: SHIPPED 2026-08-19.** Every commitment the DESIGN made has landed or is affirmatively cut
below with a named home. Floor **4772/4772 · 0 FAIL · 19 skipped**, clippy **0**, ignores **13** —
measured on a quiescent tree by the orchestrator's own invocation at `f93ce061`.

**Design:** [`DESIGN.md`](./DESIGN.md) — the intent, superseded four times in flight and kept whole.
**Realizations:** [`REALIZATIONS.md`](./REALIZATIONS.md) — R1–R5.
**This file:** the shipped contract. If these disagree, this file wins.

---

## The lineage — this arc finishes one opened 2026-04-20

Arc 118's first commit reads *"refines arc 004."*
[**Arc 004 — Lazy Sequences + Pipelines**](../004-lazy-sequences-and-pipelines/INSCRIPTION.md) was
inscribed complete on **2026-04-20**, and what it shipped under that name was a thread-per-stage CSP
pipeline: `Stream<T> = (Receiver<T>, ProgramHandle<()>)`. Real combinators, real tests — and a
channel where the lazy sequence should have been. This arc's DESIGN passes the sentence in its own
words: *"built wrong, successfully."*

004 deferred the actual lazy chains with *"in-process lazy chains haven't been demanded by a
caller."* **R5** is the entry on that sentence. Four months, and the arc is closed.

## What shipped

### The substrate — a Stream is single-pass, and that is enforced by absence

- **`Value::wat__stream__Stream`** with `LazyCell` / `NativeLazyCell`. **No memoization**, by the
  builder's ruling: *"you cannot walk back a stream — if you want this, you gotta write it."*
  Re-traversal is not shipped, and **its absence IS the enforcement** — no runtime check, no policy.
- **`:wat::stream::next` → `NextOutcome<T>`** (stone 118.11a) — the pull primitive. One force per
  element. **It is the only way a Stream yields anything.**
- **`:wat::core::Seqable<T>`** (B1/B1a) — the surface naming the four containers; parametric
  satisfaction made real (B1a), and two checker doors opened to make it dispatch (B2c, B2d).

### The wall — the three-call walk is unrepresentable

`first` · `rest` · `empty?` · `nth` **all refuse a Stream** (B4-iii, `71c7e4ea`). A user cannot write
the walk that runs their function three times per element, because the form does not exist. The
refusals compose and name the door (`c7b11901`).

Getting there took the whole B4 sequence: `nth` widened to Seqable (B4-i), `nth` promoted to a native
intrinsic with a **wat oracle** (B4-0), and a **44-site self-hosted codemod** migrating the corpus off
`(first (drop x n))` (B4-ii) — idempotent, committed as the recorded migration.

### The family — one home, clojure-familiar, complete

Lazy transformers: `map` · `filter` · `remove` · `take` · `drop` · `take-while` · `drop-while` ·
`take-nth` · `interpose` · `keep` · `keep-indexed` · `map-indexed` · `dedupe` · `distinct` ·
`reductions`.
Eager materializers and forcers: `mapv` · `filterv` · `into` · `doall` · `dorun`.
Consumers: `reduce` · `foldl` · `run!`.

### The annihilations

- **`wat/stream.wat` is GONE** — the `:wat::stream::*` thread-per-pure-stage HOFs, reimplemented over
  the lazy family. Threads survive only where a stage guards mutable state.
- **`wat/list.wat` is GONE** — `:wat::list::*` retired by recorded codemod
  (`wat-scripts/fixes/rename-list-to-seq.wat`). The 9 surviving mentions are that codemod plus an
  AST-rename test using the name as a data payload.
- **Both memos deleted** (B3) — `distinct` at n=8000 went from a hard OOM above 2 GB to completing;
  retention flat over an 8× range.
- **`foldr` retired** (B6b) — it was `reverse` + `foldl` wearing a name borrowed from Haskell, where
  the verb is distinct only because it is LAZY. wat is strict. `(reduce f init (reverse coll))`.

### The performance, measured

```
the drain          529ms -> 22ms native (B5); round-trip 87% of it, map's closure 13%
dorun              O(n) live -> O(1) live (B8): 8x the input, +0.4% peak RSS
walk shapes        next-only n+1 · empty?+next 2n+1 · empty?+first+next 3n+1
nth over a Stream  quadratic by construction — 21 forces for i=0..5, vs 7 for a next-walk
```

### The census the arc owed

**44 files · 373 defns · 12 lazy-cell walkers · 4 growth hits · exactly 1 in both.**
`distinct` is the whole class, and the reason is the finding: only one verb in the surface needs
unbounded history. `MEASURED-118.B8-the-class-has-exactly-one-member.md`; the instrument is committed
and reproducible.

## The stones, in order

```
118.1   the foundation, single-pass          118.2a  the flip — core HOFs go lazy
118.2Z  the family completes                 118.3   Seqable's real fork
118.3-B parametric surface satisfaction      118.4   the contract; length/empty? ruled
118.7   the UX forms (and 118.4 corrected)   118.9   ⛔ WRONG, and said so in its own title
118.10  the pull primitive                   118.11a next + NextOutcome
B1/B1a  mint Seqable, make it satisfiable    B2      collapse to Seqable (six verbs)
B2b     migrate the six                      B2c/B2d the two checker doors
B3      delete BOTH memos                    B4-i/0/ii/iii  widen, promote, codemod, WALL
B5      the drain goes native                B6/B7   native foldl + reduce collapses 8 arms to 2
B6b     retire foldr                         B8      the arc's tail — three owed items
```

## Realizations

- **R1 — NON BIS IN IDEM FLVMEN** — the datastream; never twice in the same river.
- **R2** — the flip is a rebirth; the eager family dies and the lazy one rises from the cascade.
- **R3** — the seq family was hand-rolled in the dark because core could not see itself.
- **R4** — when worlds collide: a clojure REPL speaking to wat over the EDN wire.
- **R5 — QVAESTIO SVBSTITVTA LEGEM ELVDIT** — a substituted question evades the law. Arc 004 held
  *"Absence is signal"* and its own titular deferral in the same file, on the same day.

## Out of arc 118's scope — affirmative cuts, every one with a home that EXISTS

- **`HashSet/conj` full-clones per insert (O(n²) time).** Measured — 12/53/223/875ms at
  n=2k/4k/8k/16k. Out of arc 118's scope; the mechanism, the numbers, and the four call sites the
  census produced are recorded in
  [`109/NOTE-hashset-conj-full-clones-per-insert.md`](../109-kill-std/NOTE-hashset-conj-full-clones-per-insert.md).
  Builder's ruling: *"we chase it later."* B3 removed the memory half of the interaction; the time
  half is a separate cost and is named as such.
- **The rete sub-language has no right fold.** Retiring `foldr` removed its `Redispatch` row, and the
  rete vocabulary has no `reverse` to spell the replacement. Out of arc 118's scope — it is a rete
  vocabulary question and 255 owns purity per
  [`255/RULING-255-owns-purity-rete-only-measures-composition.md`](../../06/255-builtin-registry/RULING-255-owns-purity-rete-only-measures-composition.md).
  Tracked as **task #109**, which carries the measurement (`reverse` is pure ∧ deterministic ∧
  unconditionally total, verified by reading `eval_vec_reverse`) so it is not re-derived.
- **Any unregistered `:wat::`-prefixed head type-checks clean.** Found while retiring `foldr`, proven
  by run with a positive control. Out of arc 118's scope: it is **arc 255's founding hole**
  (`255/REALIZATIONS.md:18`), scoped at `resolve/walk.rs:257`, and stone **255.1b-iv** deletes it.
  Tracked as **task #110**, which adds the second door 255's record does not name (`check.rs:5568`).
- **A macro program body cannot call wat-defined functions.** Isolated by differential this arc
  (stash, rebuild, retest against the pre-change body — byte-identical failure; and against an
  untouched verb — same error class), so it is general and pre-existing, not introduced here. Out of
  arc 118's scope; tracked as **task #107**, which now carries all three arms of that differential.
- **`extract_lazyable_elem` is NOT deleted, and B2's instruction to delete it is struck.** Stone B7
  made `Seqable` its fifth head; the function is the one door six inference sites consult. The
  standing order is removed from its doc and the reason recorded there. Shipped in B8, not cut.

## What this arc does NOT claim

Arc 004 was not incompetent: it shipped working combinators, real tests, and two lessons with
numbered procedures — one of which is R5's hero. And the four months were not idle. 118 could not
have been built on April's substrate: it needed `Seqable`, parametric surface satisfaction,
clause-TCO, and two checker doors that did not exist. **The claim R5 makes is narrow — the arc was
deferred by a question substitution, not by difficulty.**

---

**Arc 118 — complete.** A lazy sequence in wat is a single-pass Stream, advanced only by
`:wat::stream::next`, with a wall that makes the three-call walk unrepresentable and a drain written
in Rust with its specification written in wat. `wat/stream.wat` and `wat/list.wat` are gone. The
surface is one home wide and reads the way a Clojure developer expects, except where wat's dialect
is honestly different and says so.

*Opened as arc 004 on 2026-04-20. Closed 2026-08-19.*

***NON BIS IN IDEM FLVMEN.***
