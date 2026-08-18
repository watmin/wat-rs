# DESIGN STONE — 118.B4 · close the THREE doors. `next` becomes the only way to advance a Stream.

**Route B, stone 4 of 5.** B1 minted `Seqable<T>`; B2b migrated the six walkers; B2c/B2d opened both
dispatch doors; B3 (`b1d876f6`) deleted BOTH memos — which fixed `distinct`'s OOM and flattened
retention to 0.38 B/elem, and **unmasked for user code the 3× walk it had been hiding for the stdlib.**

B4 is the wall B3's honest cost note promised. **Builder's ruling, 2026-08-18:** *"we do not do
conventions - we do walls - users may not make mistakes in wat."*

> ## ⚠ AMENDED 2026-08-18, SAME DAY — THE STONE IS THREE STRIKES, NOT ONE
>
> The first draft closed the doors and codemodded the corpus, and **it would have opened a hole.**
> The builder asked *"do we just redefine nth to be `(first (drop X n))`?"* — the answer is no (that
> puts the forbidden idiom inside the one door), **but the question exposed what the first draft
> missed:** `(first (drop X n))` is not a clumsy `nth`, it is **the only general positional lookup
> the language has**, and B4 closes it. See *THE HOLE* below. The stone now has a piece that mints
> the door before the wall takes the old one away.

## What it does — three strikes, in order

```
B4-i    WIDEN `nth` to Seqable — mint the general positional door.   (nothing closes yet)
B4-ii   CODEMOD the corpus:  (first (drop X n))  ->  (nth X n)       (R21 wat-fix)
B4-iii  CLOSE the three doors.                                       (the wall)
```

Each is green on its own, so each floor result is a clean signal. Bundled, a red cannot be
attributed.

## B4-iii, the wall itself — the three doors

`first`, `rest`, and `empty?` stop accepting a `Stream<T>`. `:wat::stream::next` → `NextOutcome<T>`
becomes the single door through which a lazy sequence advances.

```
first    StreamContainer::indexable()   seq_container.rs:165   Stream => true  →  FALSE
rest     StreamContainer::has_tail()    seq_container.rs:184   Stream => true  →  FALSE
empty?   StreamContainer::measurable()  — ALREADY false for Stream.
         The gate is not missing; runtime.rs:17337 is a hand-written arm that ROUTES AROUND IT.
```

`seq_container.rs` documents its own bypass in the `measurable()` doc comment: *"`empty?` IS
supported via realize (handled directly in eval_empty's Stream arm, **not routed through this
gate**)."* So the `empty?` runtime half is a **deletion**, not a new mechanism.

**Both halves are required — checker AND runtime.** This is measured, not assumed: the dry run below
failed at *runtime*, inside macro expansion, where the static type was not known to be a Stream. A
checker-only wall would have let the stdlib's own violation through.

## ⛔ THE HOLE — why B4-i exists, measured

```
(first (drop X n))    `drop` takes  Seqable<T>   →  works on ANY sequence
(nth X n)             `nth` takes   Vector<T>    →  Vector ONLY
```

`wat/core.wat:1406` — `(:wat::core::defn :wat::core::nth<T> [v <- :wat::core::Vector<T> i <- :i64]`.
Proven live at HEAD:

```
(nth (PersistentVector 10 20 30) 1)
  ⇒ #wat.check/TypeMismatch ":wat::core::nth: parameter #1 expects :wat::core::Vector<?7909>;
                             got :wat::core::PersistentVector<wat::core::i64>"
```

⚠ **The List case is UNPROVEN** — my probe failed on my own `(List :i64 …)` syntax, not on `nth`.
Do not repeat "nth refuses a List" as measured; measure it in B4-i.

**So closing `first`-on-Stream removes the language's only general positional lookup.** The 44 corpus
sites never feel it — every receiver is an `ast->children` result, i.e. a `Vector<WatAST>` (18 `ch`,
9 `fn-ch`, 5 inline `(ast->children …)`, 3 `stop-fn-ch`, 3 `hibernate-fn-ch`, 2 `init-fn-ch`, 2
`def-ch`, 2 `arg-ch`). **A user holding a PersistentVector and an index would have no door at all.**

★ **B4-i is expressible today only because B2c bought it.** A surface-typed `Seqable<T>` clause arm
dispatches as of `13b27e8d`; a week ago it silently never fired. The shape:

```
indexable / gettable containers  →  (Option/expect (get v i) "nth: index out of range")   O(1), unchanged
Stream                           →  a `next`-walk, O(n), raising by name at exhaustion
```

O(n) on a Stream is honest, not a regression: `seq_container.rs:65` already states *"Stream has no
O(1) nth"*, and the alternative — pretending otherwise — is the dishonesty the arc exists to remove.

## ★ THE BLAST RADIUS IS MEASURED, NOT PREDICTED

The two capability bits were flipped on a scratch copy and the floor was run (reverted; tree clean;
floor re-verified 4747/4747 afterwards).

```
cargo build --release          CLEAN, 31.45s  ← ⛔ SEE THE TRAP BELOW
scripts/floor.sh               4747 run, 1802 passed, 2945 FAILED, 19 skipped   exit=100
```

**2,945 failures are ONE site cascading.** Every arm carries the identical cause:

```
#wat.runtime/TypeMismatch
  :message ":wat::core::first: expected tuple, Vec, List, or PersistentVector,
            got wat::stream::Stream `<lazy-seq>`"
  :location wat/service.wat:468:41
```

`wat/service.wat:468` sits inside `defservice`'s macro body, so `wat/cache.wat:195`'s `defservice`
fails to expand, the stdlib never loads, and 2,945 tests die downstream of one call. **The number is
a cascade depth, not a violation count.** Do not brief it as 2,945 sites.

## B4-ii — the codemod is NOT a pure rename

```wat
;; wat/service.wat:468
init-params-vec (:wat::core::first (:wat::core::drop init-fn-ch 1))
```

Equivalence measured at HEAD, in range and at n=0 — identical. **Out of range, they diverge:**

```
(nth v 7)                  ⇒ raises "nth: index out of range"   rc=2
(first (drop v 7))         ⇒ nil, silently                      rc=0
```

That second line **is the tracked B5 hole, live, on a Vector** — not only on a Stream. `drop` yields
an empty Stream, `first` on it returns bare `nil`, and the program carries on with `nil` where an
element should be. The codemod closes it, which is a strict improvement and the totality direction
(task #64) — **but it is a behaviour change and the strike must treat it as one.** Any site whose
surrounding code branches on that `nil` is a STOP, not a guess.

★ **R21 applies: this is a wat-fix codemod, not hand-edits.** A grep — *not a census* — puts the
idiom at **44 hits across 13 files**: `wat/service.wat` 10 · `wat/lint.wat` 6 · `wat/fix.wat` 5 ·
`wat/bracket.wat` 4 · `wat/deporder.wat` 1 · 18 in `wat-scripts/`. **The strike owes a form-tree
census before it quotes a number** — grep cannot tell a call from a comment, and this project has
been wrong five times doing exactly that.
`[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`

## Why the wall — measured

`wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat`, 5 elements, no memo:

```
A  next-only                  6 FORCED  = n+1   1x per cell
B  empty? + next             11 FORCED  = 2n+1  2x per cell
C  empty? + first + next     16 FORCED  = 3n+1  3x per cell
```

⛔ **Walk C uses NO `rest` and pays the full 3×.** This refuted the "close `rest` only" option
outright — `next` is itself a tail source, so a partial close changes how the user spells the 3×
walk, not what it costs. Closing two of three leaves 2× (row B). **Only closing all three leaves
`next` as the sole door.**

## The four questions

- **Obvious? YES.** One rule: *a Stream advances through `next`.* A reader counts the `next` calls
  and knows the force count.
- **Simple? YES.** One property — a Stream is not freely re-forceable — expressed through the
  capability table arc 278 built to be the single source of truth. Two flips and one deletion.
- **Honest? YES.** It states the true cost (forcing is not free) and refuses to hide it. It also
  states its own limit, below, and B4-i means it does not take a capability away without replacing it.
- **Good UX? YES.** Nothing is lost: `NextOutcome::Exhausted` already answers what `empty?` was being
  asked, in the same force that yields the value; and `nth` covers positional lookup for every
  container that had it through the old idiom.

## ⚠ THE TRAP — THE COMPILER DOES NOT CATCH THE DEAD ARMS

`cargo build --release` was **clean** with both bits flipped. `StreamContainer`'s exhaustiveness
guarantee protects against a container being *forgotten*; it does **not** fire when a container's arm
goes *dead*, because the arms sit inside a `match` guarded by an `if container.indexable()`.

So the now-unreachable `StreamContainer::Stream` arms in `eval_positional_accessor` (`runtime.rs`
~15456) and `eval_rest` (`collection/eval.rs`, the `realize` arm) **must be hand-converted** to
`unreachable!("indexable() gate excludes Stream")` / `unreachable!("has_tail() gate excludes
Stream")` — the house pattern already used two lines away for `HashSet` and `Tuple`. Nothing will
remind you. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## The ONE contract decision, pinned

**The diagnostic hands the user the door.** A refusal that only says "Stream not accepted" teaches
nothing; a user who reaches for `first` on a lazy seq must be told, in the error, that
`:wat::stream::next` returns `NextOutcome<T>` = `Item(value, rest) | Exhausted` and yields value and
tail in one force. Both the checker error and the runtime `TypeMismatch` carry it.

## ★ IT KILLS A TRACKED HOLE FOR FREE

`wat/seq.wat` (~:597) documents the B5 hole in its own comment: *"`first` on an exhausted Stream
returns a bare `nil`"* — reached by `reductions`' and `reduce`'s 2-arity Stream arms before B2.
B4-ii closes it on the Vector path and B4-iii makes the Stream path unreachable. The stone must
rewrite that comment; a comment that survives its subject is the rot this project keeps paying for.

## ACCEPTANCE

| strike | | assertion | instrument |
|---|---|---|---|
| i | 1 | `nth` answers on Vector, PersistentVector, List **and** Stream | a new `wat-tests/` deftest per container |
| i | 2 | `nth` on a Stream is **one force per cell visited**, and raises by name past the end | the force-shape probe, extended |
| i | 3 | Vector/PV stay O(1) — no walk introduced on the indexable path | read the dispatch; a bench only if the read is ambiguous |
| ii | 4 | the corpus census of `(first (drop X n))` returns **ZERO** | a NEW form-tree census in `scratch-pad/` |
| ii | 5 | the codemod is **idempotent** — second run is a no-op | dry-run on a `/tmp` copy + `diff` |
| iii | 6 | walk C is **refused**, and the message names `next` | a `.bad` fixture beside the probe |
| iii | 7 | walk A still yields **6 FORCED for 5 elements** | `probe-118B4-forces-per-element-by-walk-shape.wat` |

Plus, per strike: floor ≥4747/0, clippy 0, ignores 13.

⚠ **Every run capped.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s>`

## Rooms beyond the three flips

- `runtime.rs:17337` — `eval_empty`'s Stream arm: **delete**, falling to the existing
  `measurable()` gate.
- `empty?` has **no checker arm at all** (`∀T. T -> bool`, arc 237.7b-i, `check.rs:19906`), and
  neither does `length`. The compile-time half means **a new arm in `infer_list` routing through
  `measurable()`** — the same shape `rest` already has at `check.rs:4487`. ★ **Name the cost:** arc
  255 is working to shrink `infer_list`'s 141 hand-written keyword arms, and this adds one. It is an
  existing pattern extended, not a new concept, but it moves that count the wrong way and 255 should
  hear it from this stone rather than discover it.
- `wat/core.wat:1393–1407` — `nth`'s header already carefully distinguishes total-CONTRACT from
  partial-FUNCTION. B4-i widens the type; **that header's argument survives and must not be
  rewritten away**, only extended with the Stream row's cost.
- `seq_container.rs`'s capability-matrix doc comment and the three `Stream` rationale comments.
- `wat/seq.wat` ~:597 — the B5-hole comment, above.

## Out of scope — affirmative cuts

- **`(do (next s) (next s))` still forces twice.** Nothing short of linear types stops it. What this
  stone buys is that **every force is a visible `next`**, countable by reading the source, where
  today three verbs that each look free hide the cost. Stated so it is never claimed as more.
- **`map`/`filter`/`foldr` over a Stream** — the rest of `mappable()`'s gap. Tracked, unowned.
- **`into` absorbing the drain; `stream->pvec`/`stream->vec` retiring** — B5.
- **`HashSet/conj`'s O(n²) full clone** — independent, unowned.
- **Widening `get` itself, or minting `Seqable/get`** — B4-i widens `nth`'s signature and reuses the
  existing `get` on the indexable path. `get`'s own family is not this stone's business.
