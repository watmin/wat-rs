# FINDING — the `where` fence's interior is not type-checked. At all.

**Found 2026-09-09**, as a side effect of migrating the corpus away from the join blowup. ⛔ **This is
a correctness hole, and it is more serious than the performance finding that uncovered it.**

## The proof — one predicate, two positions

`string::=` applied to an `i64`-typed field:

```wat
(:wat::core::defrecord :tg::N [k <- :wat::core::i64])

;; A — inside a where fence
:when [(:tg::N (?k <- :k))
       (:tg::N (?j <- :k))
       (:wat::rete::where (:wat::rete::core::string::= ?k "nope"))]

;; B — the same predicate, inline in the condition
:when [(:tg::N (?k <- :k))
       (:tg::N (?j <- :k) (:wat::rete::core::string::= ?k "nope"))]
```

| position | result |
|---|---|
| **A — fence** | ⛔ **ACCEPTED SILENTLY.** Program loads, prints `"loaded"`, exit 0, no diagnostic |
| **B — inline** | ✅ `ReteCheckErrors` at startup |

## It is not theoretical — it is already in the corpus, in a teaching file

`wat-scripts/fixes/to-faithful-clojure-net.wat`, rule `g3-genuine`, carries exactly this defect: a
`string::=` comparator over `i64`-typed fields. **It has never been type-checked**, because it sits
inside a fence. It surfaced only when a codemod hoisted it inline and the clause compiler rejected it
with `ConstraintTypeMismatch`.

⛔ **That file is a RECORDED MIGRATION** — one of the exemplars anyone writing a codemod reads first.

## The site, and what its comment actually means

`src/rete/validate/mod.rs:282` (and its clause-level twin at `:456`):

```rust
// Design call 3 — a `where` fence's outer shape is already confirmed by the
// classifier (2-item, `:wat::rete::where` head); its interior expr is out of scope.
ReteClauseShape::Where(_) => {}
```

The comment reads as a scoping decision about **shape**. In practice it means the interior receives
**no type checking whatsoever**. The fence is a hole in the type system, not merely a missed
optimisation. ⭐ The `compute` host called this arm *"the hole"* and was more right than its own
report argued — it diagnosed a **performance** gap and had found a **soundness** one.

## Why fifty-plus ward casts did not find it

`vigilia-2026-09-07-rete` cites `validate/` in twelve rows; **eight wards read the file** — `purgare`,
`conferre`, `sequi`, `perspicere`, `probare`, `excusare`, `intueri`, `solvere`. **Not one report
mentions `Design call 3` or `Where(_)`.**

- **`peragrare` — whose subject this is — was mustered out.** `README.md:46`: *"`peragrare` | a
  load-bearing instrument AND its corpus | **NO here** — fires on target 3."* The validator **is** a
  load-bearing instrument and its `Where` branch **is** an unvisited cell.
- The eight that did read it were each aimed elsewhere: the arm is **not dead** (`purgare` — it is a
  deliberate no-op), **contradicts no spec** (`conferre` — nothing claims it validates), and **carries
  a real justification** (`probare` — the comment *is* substance).

⭐⭐ **It looks exactly like what it is: a considered decision that the corpus outgrew. No inward lens
is aimed at *"this was right when it was written."***

## The cure, and why it is separable

**Type-check the fence interior exactly as an inline clause is type-checked.**

⭐ **This is INDEPENDENT of the hoisting/expressivity question** parked in
`DESIGN-widen-the-clause-then-refuse-the-fence.md`, and it should not wait for it:

- **No expressivity objection exists.** Nobody is arguing for the right to write `string::=` on an
  `i64`. Widening the *inline* grammar is contested; type-checking the *fence* is not.
- **No corpus migration is implied.** The predicate stays exactly where the author wrote it.
- **It would have caught `g3-genuine` the day it was written**, years of ward casts earlier.

⚠ **Expect it to find more than one.** `g3-genuine` was found by accident, by a codemod that happened
to hoist it. **Nothing has ever looked at the other fence interiors.** The first run of this check is
a census of a population nobody has counted.
