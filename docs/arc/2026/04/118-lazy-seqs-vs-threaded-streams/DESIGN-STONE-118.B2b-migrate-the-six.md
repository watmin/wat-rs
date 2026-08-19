# DESIGN STONE — 118.B2b · migrate the six remaining three-call walkers

**Route B, the stone between B2 and B3.** B2 (`b4a8f86b`) collapsed six verbs and deleted the seven
twins. It did **not** finish the file: five verbs were left untouched and `wat/seq.wat:315` says so
in its own comment. The census (`727db5ec`) names them.

## ⛔ FIRST — correct the record

`DESIGN-STONE-118.B3-delete-the-memos.md` opens with a FALSE precondition:

> *"…leaving **zero three-call Stream walkers in the corpus.** That was the precondition. It is now met."*

It is not met. That sentence was written from the third of three bad greps, before the census
existed. **Six units stand.** B3 is the document a rider reads first, so the false line is fixed as
part of this stone. `[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`

## The six, and why six for five verbs

```
$ printf '["wat/seq.wat"]\n' | ./target/release/wat wat-scripts/scratch-pad/census-three-call-stream-walks.wat
  THREE-CALL  :wat::core::remove          1 unit
  THREE-CALL  :wat::core::take-while      1
  THREE-CALL  :wat::core::drop-while      1
  THREE-CALL  :wat::core::take-nth        1
  THREE-CALL  :wat::core::reductions      2   ← 3-arity Stream arm AND 2-arity Stream arm
```

The census judges a UNIT (a `defn`, or ONE arm of a `defclause`) whose **parameter vector** names a
`Stream<` and whose body structurally calls `first`/`rest`/`empty?`. So it counts exactly the arms
that walk a Stream three times per cell — which is exactly B3's hazard. Each verb has five arms;
only the Stream arm scores. `reductions` scores twice because it genuinely has two.

## Why now — this is B3's precondition, not a tidy-up

Every one of these arms forces the same cell three times. Today the memo hides it. **Delete the
memo with these alive and each runs its upstream producer 3× per element** — and for
`(take-while p (map f xs))` that is the *user's* `f`, three times, silently.

`take-nth` takes no user function, but it is not exempt: its upstream can be lazy, so re-forcing a
cell of `(take-nth 2 (map f xs))` re-runs `f`.

## The shape — proven five times over in this same file

Public verb over `Seqable<T>` → private `-walk` over `Stream<T>`, walking with
`:wat::stream::next`. `interpose`/`keep`/`keep-indexed`/`map-indexed`/`dedupe`/`distinct` are all
already this. No new primitives, no new types, no Rust.

## ⚠ TRAP 1 — `take-nth`'s degenerate `n`, MEASURED not read

```
HEAD:  (take [] 5 (take-nth 0 [1 2 3]))  →  1,1,1,1,1
```

An infinite repeat of the head — and that is **clojure-faithful**. The mechanism: the recursive call
is `(take-nth n (drop coll n))`, dropping from the FULL coll (head included), and `drop` clamps a
negative `n` to 0 (`src/collection/transform.rs:201`). At n=0 it re-consumes the same collection
forever.

**The naive `next` rewrite — emit `value`, recurse on `(drop rest (- n 1))` — silently changes this
to `1,2,3`.** Nothing in the corpus calls `take-nth` with n≤0, so a green floor would not catch it.

**The fix, and it costs nothing:** rebuild the consumed head before dropping.

```wat
((:wat::stream::NextOutcome::Item value rest)
  (:wat::stream::cons value
    (:wat::core::take-nth-walk n
      (:wat::core::drop (:wat::stream::cons value rest) n))))
```

`(stream/cons value rest)` is a plain `Cons` — `realize` on it is the identity, so `drop` walks it
for free and every downstream cell is still forced exactly ONCE. n=0 drops nothing and yields the
same cell back: the infinite repeat, preserved. n≥1 skips `value` plus n−1 from `rest`: correct.

## ⚠ TRAP 2 — `reductions`' 2-arity on empty, and a comment that is FALSE

`wat/seq.wat:676` claims: *"an empty `coll` raises via `first`'s out-of-range failure rather than a
silent 0-arity dispatch."* Measured against HEAD:

| input | today | matches the comment? |
|---|---|---|
| empty **Vector** | RAISES — `first: sequence has 0 element(s); no element at index 0` | ✓ |
| empty **Stream** | silently yields a one-element stream **`[nil]`** | ✗ |

The comment is true for four arms and **false for the Stream arm**. It reaches `[nil]` through the
tracked B5 hole — `first` on an exhausted Stream returns a bare `nil` (confirmed this session:
`(str (first (stream/empty)))` → `"nil"`).

**This is the second instance today of a comment shipping a gap as a law.**
`[[feedback_a_comment_can_ship_a_gap_as_a_law]]`

### The disposition — and it is FORCED, not chosen

Collapsing to one `Seqable<T>` definition means one answer for all five containers. The `Exhausted`
arm must produce a `Stream<T>`, and there are exactly three candidates:

- **(a) reproduce `[nil]`** — `(stream/cons ??? (stream/empty))` needs a `T`. There is no `T` to
  hand it. **It cannot be written.** The old behaviour was only ever reachable *through* the B5
  hole; with `next` the hole is not on the path. **Obvious? YES** (the type refuses it).
- **(b) raise, named** — `assertion-failed!` with an explicit message. Delivers what the comment
  always claimed, for every container, including the one where it was a lie.
- **(c) change the return type to an outcome** — a public signature change on a verb this stone is
  not about, and the totality campaign's business (task #64). Rejected affirmatively.

**(b).** Not a preference — (a) is unrepresentable and (c) is out of scope. The language refusing to
let me reproduce the defect is the substrate doing its job.

**Honest? YES**, and say the cost out loud: for the four eager containers the raise's *message*
changes (from `first`'s incidental out-of-range text to a named `reductions` one). For a Stream a
silent wrong answer becomes a loud correct one. No caller exists to break — `reductions` has zero
call sites outside `seq.wat`.

## The four questions on the stone as a whole

- **Obvious? YES.** Five verbs adopt the shape the other six already have; one file.
- **Simple? YES.** No new primitive, no new type, no Rust, no arity change.
- **Honest? YES.** The two behaviour deltas are named above, measured before and after, not
  discovered afterwards.
- **Good UX? YES.** After this, every sequence verb in the stdlib is one definition over any
  seqable — which is what a user writing their own lazy stage has to copy.

## ⚠ STOP triggers — drawn around COMPLEXITY, not arity

B2's STOP-2 was drawn around *arity*, so the compliant path was quadratic and shipped an O(n²)
regression. `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]` The guard
here names the PROPERTY:

- **STOP-1 — any verb's ALGORITHMIC COMPLEXITY would change.** All five are O(n) single-pass today
  and must stay O(n) single-pass. A private `-walk` helper carrying state is *expected* and is not a
  STOP; re-scanning the remainder, wrapping `f` in a per-element closure, or forcing any cell twice
  IS. Keeping the public arity is not the goal — keeping the complexity is.
- **STOP-2 — `take-nth 0` stops repeating the head.** Trap 1. Baseline `1,1,1,1,1`.
- **STOP-3 — laziness breaks.** `probe_arc118_2z_takewhile_lazy` must stay green: `take-while` over
  a lazy `map` must never force the cell past the first false.
- **STOP-4 — floor red for any reason other than a line-number shift in a golden**, or `#[ignore]`
  off 13. There is no such thing as a known flake.

## ACCEPTANCE

| # | assertion | instrument |
|---|---|---|
| 1 | ★★ **the census returns ZERO** | `census-three-call-stream-walks.wat` over `wat/seq.wat` |
| 2 | ★ every baseline row byte-identical **except** the one named delta | `probe-118B-six-walkers-baseline.wat`, before/after diff |
| 3 | `take-nth 0` still `1,1,1,1,1` | row 1 of the same probe |
| 4 | `reductions/2` on empty raises a NAMED error, both containers | rows 9–10 |
| 5 | laziness holds | `probe_arc118_2z_takewhile_lazy` + `seq-of-infinite-stream-stays-lazy` |
| 6 | floor ≥ 4714 passed, 0 failed, 0 timed out | `scripts/floor.sh` Summary line |
| 7 | clippy 0 · `#[ignore]` 13 | — |

## Out of scope — affirmative cuts

- **B3 itself** — the memos stay standing in this stone. This is its precondition, not it.
- **`first`/`rest`/`empty?` accepting a Stream** — B4, the builder's dialect ruling. This stone
  removes the stdlib's *use* of the three-call walk; it does not close the door.
- **`first` on an exhausted Stream returning bare `nil`** — B5. Trap 2 is a *consequence* of it, and
  this stone routes around it rather than fixing it.
- **The five verbs having ~zero adoption** — real (`remove`, `take-while`, `drop-while`,
  `reductions`: zero call sites outside `seq.wat`; `take-nth`: three, all in one scratch probe).
  That is the UNADOPTED class, task #48. Not a reason to delete them and not this stone's business.

---

# OUTCOME — struck 2026-08-18. Everything above this line was written BEFORE the strike.

## The scorecard, on my own re-run

| # | assertion | result |
|---|---|---|
| 1 | ★★ the census returns ZERO | **ZERO — over all 491 corpus `.wat` files**, not just `seq.wat` |
| 2 | ★ baseline rows byte-identical except the named delta | **exactly one line of diff**, and it is the named one |
| 3 | `take-nth 0` still `1,1,1,1,1` | **held** |
| 4 | `reductions/2` on empty raises a NAMED error | **held**, both Vector and Stream |
| 5 | laziness holds | held (`probe_arc118_2z_takewhile_lazy` + the new lazy rows) |
| 6 | floor | **4727 tests run: 4727 passed, 0 failed, 19 skipped** |
| 7 | clippy 0 · ignores unmoved | clippy **0**; skipped **19**, identical in every floor log today |

The before/after diff, in full:

```
10d9
< "reductions/2 [] : nil"
```

## ⚠ THE PREDICTION THAT WAS WRONG — `reductions` did NOT collapse to two arms

This stone predicted `reductions` would become one `Seqable<T>` arm per arity. **It cannot**, and the
strike is what discovered why:

```
no clause of :wat::core::reductions matched (3 args);
clause 0 skipped (arg 2: expected :wat::core::Seqable<T>, got :wat::core::Vector)
```

**A `defclause` ARM typed with a surface never dispatches.** B1a taught the CHECKER that a concrete
instantiation satisfies a parametric surface; the runtime clause selector is a second door that
never learned it. A plain `defn` never reaches that selector — which is exactly why the four
single-arity verbs migrated cleanly and the one multi-arity verb did not.

So `reductions` keeps ten per-container arms, each a one-line delegation to a shared
`reductions-walk`, mirroring `:wat::core::reduce` directly above it — the same shape B2 left `reduce`
in, for the same reason. **The stone's actual purpose is unaffected:** the three-call walk is gone
from every arm, which is what B3's precondition needs.

A SECOND gap surfaced from the same half-wired surface — `Seqable/seq` on a `Vector<i64>` returns
`Stream<T>` with `T` unbound, so a surface method's result cannot feed a concrete consumer. Both are
recorded with controls in
`NOTE-118.B2b-two-doors-the-checker-opened-and-the-runtime-did-not.md`; door 1 is drawn as
`DESIGN-STONE-118.B2c-a-surface-typed-clause-arm-never-dispatches.md`.

★ **Neither was caused by this stone and neither was visible before it.** B1a's green answered the
question its instrument asked — *does a concrete type satisfy a parametric surface PARAMETER on a
`defn`?* — and was read as "surfaces work."
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## What the stone added beyond the migration

`wat-tests/core/core-seq-walkers.wat` — **13 tests**, because these five verbs had almost no runtime
coverage (three had none at all) and a green floor would have said nothing about whether the rewrite
preserved behaviour. The load-bearing one is `take-nth-0-repeats-the-head`: it is the only thing in
the corpus that can catch the trap this stone was one line away from shipping.
`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

## One thing that went wrong in the strike, and it was mine

My first cut of that test file exercised each verb's "fourth container" as
`(Seqable/seq (Vector …))` — a redundant re-wrap. It did not type-check (door 2), and because a
`deftest` file that fails to check fails EVERY test in it, the floor came back **13 red at once**.
The cause was one broken file, not thirteen failures. Replaced with `(map identity v)`, a real lazy
stage — which is the better test anyway, since lazy-over-lazy is precisely where a re-forcing walker
would run the upstream's user code more than once.
