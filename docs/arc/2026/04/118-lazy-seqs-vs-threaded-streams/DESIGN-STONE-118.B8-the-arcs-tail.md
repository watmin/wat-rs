# DESIGN STONE — 118.B8 · THE ARC'S TAIL. Three owed items, and 118 inscribes.

**Builder's ruling, 2026-08-19:** *"we do the three.... dorun is bad .... we ship them.... that's why
we are here...."*

Route B is complete (B1→B7 + B6b). The arc is not, because **three stones wrote the word "tracked"
and it pointed at nothing.** This stone discharges all three, and it is the last thing between arc
118 and its INSCRIPTION.

## How the three were found

The mandatory wrap-proof sweep over every `DESIGN-STONE-118.*` file. Five deferral phrases; two are
honest, three are not:

| stone | the cut | where it points |
|---|---|---|
| B4-0 | "a macro body cannot call any wat-defined function — *tracked separately*" | ✓ **task #107.** Real. |
| B5 | "map/filter/foldr over a Stream — *still unowned*" | ✓ **resolved** — measured 2026-08-18: `map`/`filter`/`take`/`drop`/`remove` all accept a Stream; `foldr` retired by B6b |
| B3 | the class census — *"Owed, tracked"* | ⛔ **nowhere** |
| B5 | `dorun`'s build-and-bin — *"tracked, not here"* | ⛔ **nowhere** |
| B2→B5 | `extract_lazyable_elem` — *"B5 / a Rust stone"* | ⛔ **nowhere** |

★ **The tell is the word itself.** "Tracked" did a citation's work three times without being one.
That is how a deferral launders into DONE, and inscribing over it would be FM 11 with the auditor as
the violator. `[[feedback_nothing_blocks_it_is_not_a_work_item]]`

---

## PART 1 — `dorun` stops building a Vector to throw it away

`wat/seq.wat:209`, today:

```wat
(:wat::core::defn :wat::core::dorun<T> [coll <- :wat::stream::Stream<T>] -> :wat::core::nil
  (:wat::core::do (:wat::core::into [] coll) nil))
```

It materializes every element into a Vector, then discards the Vector. **That is O(n) memory for a
verb whose entire contract is "walk for effects, keep nothing."** B5 made the waste *fast*
(529ms → 22ms); it did not make it *absent*.

**The replacement is a tail-recursive `next` walk that retains nothing:**

```wat
(:wat::core::defn :wat::core::dorun<T> [coll <- :wat::stream::Stream<T>] -> :wat::core::nil
  (:wat::core::match (:wat::stream::next coll)
    ((:wat::stream::NextOutcome::Item _value rest) (:wat::core::dorun rest))
    (:wat::stream::NextOutcome::Exhausted nil)))
```

★ **This is a COMPLEXITY change, not a tidy-up** — the same class as B3's memo deletion: O(n) live
→ O(1) live. Forcing still happens (that is what `next` does, and it is what makes the side effects
run); only the retention goes.

⚠ **`doall` is NOT touched.** It returns the Vector, so `(into [] coll)` is its correct body. The two
verbs differ in exactly this and the stone must not flatten them.

## PART 2 — ⛔ `extract_lazyable_elem` IS NOT DELETED. The order expired.

**B2's instruction rested on a premise that B7 then falsified.** Its own doc still carries the order
(`src/collection/infer.rs:656-658`):

> *"Minting `:wat::core::Seqable` in the stdlib, extending these four containers, and pointing
> `join`/`map`/`filter` at it is the NEXT stone … — this function's hand-rolled four-head match is
> exactly what that stone would delete."*

`Seqable` WAS minted (B1, `488eacd0`). It did not replace the function. **Stone 118.B7 added
`Seqable` to it as a FIFTH HEAD**, with a comment stating why that arm is load-bearing:

> *"a wat verb whose parameter is declared `Seqable<T>` could not pass that parameter to
> `foldl`/`map`/`take` at all — it would have to normalise through `(Seqable/seq coll)` first,
> forcing every eager container onto the lazy path and paying for a Stream it never needed. That is
> exactly the tax stone 118.B6 removed; this arm is what stops it coming back in through the front
> door."*

Measured: **6 live call sites** (`infer.rs:734, 810, 887, 1016, 1079, 1142`) — `infer_map`,
`infer_filter`, `infer_take`, `infer_drop` and two siblings. Deleting the function means
re-hand-rolling a five-head match six times: **the exact opposite of the collapse B2 wanted.**

**THE WORK IS THE CORRECTION, NOT THE DELETION.** Rewrite the doc so it records what the function
became — the ONE door that knows the `Seqable` set, concrete heads and surface alike — and delete
the standing "would delete" instruction so no future stone obeys an order whose reason is gone.
`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`
`[[feedback_an_instruction_to_delete_needs_more_grounding_than_one_to_add]]`

## PART 3 — the class census, with an instrument that can actually see it

B3 named the shape and never counted it: **"a growing collection threaded through a lazy walk."**
`distinct` is the known instance (`wat/seq.wat:566`) —

```wat
(:wat::core::defn :wat::core::distinct-walk<T>
  [seen <- :wat::core::HashSet<T> s <- :wat::stream::Stream<T>] -> :wat::stream::Stream<T>
  ...  (:wat::core::distinct-walk (:wat::core::conj seen value) rest) ...)
```

— and because `HashSet/conj` full-clones (O(n²), measured, parked in 109), n live cells each hold an
independent full copy. **Nobody ever looked for siblings.**

⛔ **DO NOT COUNT THIS WITH GREP OR AWK.** I ran an awk pass while scoping and it returned exactly
one hit. **That number is not admissible** — splitting a nested-paren language on blank lines is a
boundary heuristic, and this project has been wrong on exactly that five separate times.
`[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`

**Use the form-tree reader** — `read-string` → `with-children`, the instrument that got B4-ii's
census right across 487 files on the first run. Copy `wat-scripts/scratch-pad/census-first-of-drop.wat`
for the shape; it already documents the structural-match discipline and the stdin-path-list usage.

**The structural predicate:** a `defn`/`defclause` that (a) recurses on itself, and (b) passes, in
that recursive call, a parameter of collection type that has been grown by `conj`/`assoc` from the
same parameter.

⚠ **TWO POPULATIONS, and a grep saw only one.** `stream::lazy` appears in exactly one file
(`wat/seq.wat`) — that is an honest narrowing of the *lazy-wrapped* walkers. But **a walker can
recurse on `next` without wrapping in `lazy`**, and that population is uncounted. The census must
cover both, over all of `wat/`.

**The deliverable is an INVENTORY, not a fix.** Each sibling gets a named disposition. If the count
is 1, that is a finding worth writing down — it means `distinct` is the whole class and 109's parked
`HashSet/conj` note covers it. `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

---

## The four questions

- **Obvious? YES.** Three items the arc said it owed, discharged where it owed them.
- **Simple? YES.** Each part is one thing: a verb's body, a doc comment, a census. They share a floor
  and a purpose and touch no common code.
- **Honest? YES.** It is the stone that stops "tracked" from meaning nothing — and Part 2 REFUSES the
  deletion its own arc ordered, because the reason expired. Shipping the order unexamined would have
  been the dishonest move that looked like compliance.
- **Good UX? YES.** `dorun` stops being a memory trap; the next reader of `extract_lazyable_elem`
  is told what it is instead of being told to delete it.

## ⚠ THE TRAPS

**T1 — `dorun` is in `is_pure_total`** (`src/macros/eval.rs:565`), deliberately, in the 118.2a block
for verbs that must run at MACRO-EXPANSION time. Its body goes from one `into` call to a
**self-recursive** walk. **Does recursion work in the macro-expansion evaluator the way it does at
runtime?** Task #107 records that a macro body cannot call wat-defined functions at all in some
positions. If expand-time `dorun` breaks, that is a STOP and a finding, not something to work around.

**T2 — the recursive call MUST sit in the `match` Item arm's TAIL position.** `wat/seq.wat:158`
records this for the drain, proven by `probe-118B-match-tco-drain.wat` with a non-tail sibling
control that SIGSEGVs at the same depth. Nesting `dorun`'s recursion inside any argument silently
makes it O(n)-stack.

**T3 — `dorun` has ZERO callers today** (only its own `defn` and the `is_pure_total` row). ⛔ That is
NOT an argument about its value and NOT a reason to inflate this into a perf win — it is a
correctness-of-shape stone. `insert-all` would have measured zero the day it landed.
`[[feedback_no_consumers_does_not_mean_dead]]`

**T4 — the goldens carve-out will probably fire.** Two `.edn` fixtures pin `src/check.rs` lines and
five pin `src/runtime.rs`; the complete census is `grep -rl ':file "src/' tests/` → **8** fixtures
(runtime.rs ×5, check.rs ×2, freeze.rs ×1). Part 2 edits a doc comment in `infer.rs` (no golden), but
if any `src/` line count moves, ratify the same way B6b did: prove every changed field is a `:line`,
every `:col` identical, and the delta reconciles against that file's measured net insertion.

## ACCEPTANCE

| | assertion | instrument |
|---|---|---|
| 1 | ★ `dorun`'s peak RSS is **FLAT** in n | the retention methodology of `probe-118B-dorun-retention-slope.wat`, pointed at `dorun`, at 100k/200k/400k/800k |
| 2 | `dorun` still forces every element (effects run, in order) | a probe counting side effects for n elements → exactly n |
| 3 | `doall` is UNCHANGED | read the diff |
| 4 | `extract_lazyable_elem` is **still there**, all 6 call sites intact, doc corrected | read the diff |
| 5 | the census runs from a FORM TREE and reports an inventory | the committed `.wat` census + its output |
| 6 | floor · clippy · ignores | **orchestrator**: ≥4772/0, 19 skipped · 0 · 13 |

Row 1 is the one that proves Part 1 was worth doing. A `dorun` that is merely *faster* is B5's
result; a `dorun` that is *flat* is this stone's.

## Out of scope — affirmative cuts, each with a home that EXISTS

- **`HashSet/conj`'s O(n²) full-clone** — measured, written up in
  `109/NOTE-hashset-conj-full-clones-per-insert.md`. Builder: *"we chase it later."* Part 3 may
  find more callers of it; finding them does not fix it.
- **The rete right fold / `reverse`'s purity ruling** — **task #109**, held at the builder's call
  while a rete refactor is in flight; and 255 owns purity per its 2026-08-15 ruling.
- **The `:wat::` blanket-accept** — **task #110**, and it is arc **255's founding hole**
  (`255/REALIZATIONS.md:18`), deleted by stone **255.1b-iv**. Not this arc's.
- **Whether `dorun` should accept a Seqable rather than only a Stream** — a surface question, not a
  retention one. Untouched here; no home needed because nothing is owed on it.
