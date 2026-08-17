# MEASURED — the lair for stone 118.B. Written 2026-08-17 against `5e5e219e`.

The study that precedes drawing stone B. Everything below was run this session on
`target/release/wat` (verified fresh: no `src/*.rs` newer than the binary). Nothing here is
quoted from the seam — the seam's own closing alarm is **DO NOT DESIGN THE STREAM TIER FROM
READING**, and two of its numbers turned out to describe a narrower population than they read as.

---

## 1. THE ROOMS — the complete walk surface, and it is smaller than feared

**`Stream<` appears in `wat/` in exactly ONE file: `wat/seq.wat`.** (The only other hit repo-wide
is a prose comment in `wat/kernel/channel.wat:30`.) Zero Stream-typed signatures exist in
`wat-tests/`, `wat-scripts/`, or `tests/`.

| # | walker | where | shape |
|---|---|---|---|
| 1 | `stream->pvec` | `wat/seq.wat:102` | **the drain.** tail-recursive, `if` + three-call |
| 2 | `reduce-stream` | `:180` | tail-recursive, `if` + three-call |
| 3 | `interpose-stream` | `:412` | `stream/lazy` + three-call |
| 4 | `keep-stream` | `:454` | `stream/lazy` + three-call |
| 5 | `keep-indexed-stream` | `:481` | `stream/lazy` + three-call |
| 6 | `map-indexed-stream` | `:512` | `stream/lazy` + three-call |
| 7 | `dedupe-stream` | `:540` | `stream/lazy` + three-call |
| 8 | `distinct-stream` | `:571` | `stream/lazy` + three-call |

**Eight walkers, not seven.** The seam says "the 7 `-stream` twins"; that count is right for the
`-stream` SUFFIX but misses `stream->pvec`, which does not carry the suffix and is the most
important one on the list.

★ **`stream->pvec` is the single materializer for the entire language.** `doall` · `dorun` ·
`mapv` · `filterv` · every `into` Stream clause · `stream->vec` all funnel into it. Migrating that
ONE function converts every eager drain in wat.

⚠ My first census of this table returned **zero** — `defn [a-z-]*-stream` cannot match
`defn :wat::core::reduce-stream`, because the head is namespaced. A positive control caught it in
one command. `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

Also corrected by count: `keep`'s identical-bodied `defclause` arms are **five**
(`Vector` · `List` · `PersistentVector<T>` · `Stream<T>` · bare `PersistentVector`), not four.

---

## 2. TRAP 1 — TCO under `match`. PROBED, DISARMED, WITH A CONTROL.

Stone B rewrites `stream->pvec` from `if` to `match`. `stream->pvec`'s own doc claims "TCO
trampoline keeps this O(1) Rust-stack regardless of stream length". **If `match` did not carry a
tail position, that migration would silently convert an O(1)-stack drain into an O(n)-stack one**
— and per tasks #58/#86 the failure here is a SILENT SIGSEGV, not a located raise. It would pass
every small test and die in production.

`eval_match_tail` exists (`src/runtime.rs:4560`, dispatched `:4309`). That is a READ. Here is the
run:

```
probe-118B-match-tco-drain.wat        (tail)      prints 200000   exit=0    0.98s  136,060 KB
probe-118B-match-no-tco-control.wat   (non-tail)  no output       exit=139 = SIGSEGV (128+11)
```

**The control is what makes the pass mean anything.** Same `match`, same 200,000 depth, recursion
moved out of tail position — it dies. So 200,000 is genuinely deep enough for a missing tail
position to be detected, and the tail version's completion is not "the frames happened to fit."
`[[feedback_a_green_test_can_prove_nothing]]`

**VERDICT: `match` carries a tail position. The migration is safe on this axis.**

---

## 3. TRAP 2 — there are TWO memos, not one

The seam says "the memo" throughout. There are two, and stone B must dispose of both:

| | `src/stream/mod.rs` | thunk is | backs |
|---|---|---|---|
| `LazyCell.forced` | **:66** | a **wat** `Arc<Function>` + captured env | `(:wat::stream::lazy …)` — every wat-written generator |
| `NativeLazyCell.forced` | **:124** | a **Rust** closure | `map` / `take` / `drop` intrinsics |

Both are `Arc<OnceLock<Arc<Stream>>>`; `realize` writes both (`:192`, `:200`). A stone that
deletes one and reports "the memo is gone" would be a false claim about half the tier — and, per
§5 below, the wat half is the expensive one by 9×.

---

## 4. TRAP 3 — the walk surface is UNBOUNDED by signature, and the three doors are NOT uniform

`first` / `rest` / `empty?` are Rust-native and dispatch on the VALUE's kind, not on a declared
parameter type. So **any** call site holding a Stream can three-call-walk it, whether or not it
declares `Stream<T>` anywhere. The §1 table bounds the *stdlib*; it does not and cannot bound the
language. A census here is structurally incapable of being complete —
`[[feedback_impose_the_check_and_read_the_screams]]` applies: the honest instrument is a wall,
not a survey.

But the three doors close by three DIFFERENT mechanisms, and one of them cannot close at all:

| verb | checker gate | runtime arm | can it be made compile-time-unrepresentable? |
|---|---|---|---|
| `first` | `StreamContainer::indexable()` → Stream **true** (`seq_container.rs:166`), consumed at `check.rs:9129` | `eval_first` | **YES** — flip one capability bit |
| `rest` | `StreamContainer::has_tail()` → Stream **true** (`:185`), consumed at `check.rs:4489` | `eval_rest` (`collection/eval.rs:1793`) | **YES** — flip one capability bit |
| `empty?` | ⛔ **NONE.** Its scheme is `∀T. T -> bool` (`check.rs:19836`) — no keyword-head arm, no container gate | `eval_empty` (`runtime.rs:17064`) — a **hardcoded `if let` that bypasses the capability table entirely**, with a comment saying it cannot route through `measurable()` | **NO** — not without changing its type scheme |

★ This is the **checker-permissive / runtime-refusing** class, found a fourth time. `empty?` is
the odd door: the capability table is the "single source of truth" everywhere else, and `empty?`
on a Stream is hand-written *above* it.

⚠ Related, cosmetic but a lie on disk: `rest`'s own TypeMismatch text reads
`expected: "Vec<T>, List<T>, PersistentVector<T>, or WatAST"` — it omits Stream, which it accepts.

### And the derivation that decides the shape

The defect is **not** `first`. A single `(first s)` is one call, one force, and is harmless. The
defect is the **sequence** — and no static check can see a sequence without dataflow.

So the question is not "which door is dangerous" but: *once the memo is gone, what does any door
that forces a cell and hands back a re-forceable handle cost?* Answer: `(first s)` then `(next s)`
is two forces of the same cell — user code runs twice. Keeping `first` and closing `rest` does not
save it.

**Derived, not preferred: a single-pass stream's READ and ADVANCE are one act. Any API that
separates them is a lie about what the thing is.** `next` is the only shape where forcing and
advancing cannot be split, which is why it needs no cache. That argues all three doors close —
but **this is a dialect ruling, not a measurement, and it is the builder's.** Clojure's
`first`/`rest` do work on lazy seqs; wat's Stream is explicitly not Clojure's lazy-seq (arc 118 R1,
`NON BIS IN IDEM FLVMEN`), and the builder has already framed Streams as Ruby Enumerators —
which expose `next`, not `first`/`rest`.

---

## 5. ★ THE MEASUREMENT — the seam's number describes the CHEAPER of two populations

Instrument: `wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat`, committed so the
number outlives the session (`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`).
It reduces to a single `i64`, so the accumulator's own retention is O(1) and every byte above the
floor is the stream chain. Non-vacuity is the printed sum, which must equal `n*(n-1)/2` **exactly**
— so one line proves the walk ran AND visited exactly n elements.

⚠ An earlier draft proved non-vacuity with `(length (into [] …))`. maxRSS is a **peak**: that
scaffolding allocates more than the subject and would have masked the entire signal. The
instrument would have reported itself.

### Slope, wat-closure generator (`probe-118B-dorun-retention-slope.wat`)

```
n          sum (exact)        maxRSS KB      wall
100,000    4999950000  OK       356,380       1.02s
200,000    19999900000 OK       675,096       1.88s
400,000    79999800000 OK     1,312,572       3.27s
800,000    319999600000 OK    2,587,964       7.06s
```

Linear. **3,188 B/element** (from the 100k→800k span), fixed footprint ≈ 37.6 MB.

### The three populations, same n=400,000, identical exact sums

```
A  eager Vector, no Stream at all          63,120 KB   0.69s   ← the floor
B  native map chain over that Vector      200,284 KB   3.99s   ← NativeLazyCell
C  wat-closure generator + take          1,312,628 KB  3.90s   ← LazyCell
```

```
B − A  =   137,164 KB / 400,000  =    343 B/element
C − A  = 1,249,508 KB / 400,000  =  3,124 B/element      ← 9.1× population B
```

**The seam carries ~297 B/element. That is population B** — the native `map` chain — and my 343 B
corroborates it. **It is not the number for population C**, the wat-written generator, which is
**9.1× worse.**

★ And population C is *exactly the idiom the builder described* — the Ruby paginated-Enumerator
pattern, a producer yielding items on demand. The tier's headline cost was being quoted from its
cheap half. Extrapolated to 10M elements: **B ≈ 3.4 GB, C ≈ 31 GB.**

Time is not free either: B costs **5.8× the wall** of A for identical arithmetic.

⚠ **State what the instrument can see:** these are peak-RSS numbers from `/usr/bin/time` on a
single machine, one run per point, with the allocator's own behaviour included. The SLOPE is the
claim; the absolute footprint is not portable.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

---

---

## 6. ★★ THE REFRAME — native walkers NEVER had the three-call defect

Read the loops that actually run: `lazy_take_stream` (`src/collection/transform.rs:181`) and
`eval_vec_drop`'s loop (`:246`, `:248`) call `crate::stream::realize` **once** per cell and
destructure the `Cons`. One force, one element. Every native walker has always done this.

**The three-call walk is a wat-side-only disease.** It exists because wat's whole Stream API was
`empty?`/`first`/`rest` — three verbs, three independent `realize` calls on one cell. Rust never
lacked the fused pull. `next` (118.11a) is wat finally being given it.

This changes what stone B *is*. §1's eight walkers are all wat. Under a route that moves those
walks to Rust, they are **deleted**, and the defect goes with them — so B's "migrate 8 walkers to
`next`" would be effort spent on code queued for removal.

## 7. THE RULING THAT ALREADY EXISTS — and its expired premise

The builder asked whether the `-stream` / `stream->` names are a crutch. **This project already
ruled that they are**, 2026-07-31, on the builder's own challenge
(`278/DESIGN-STONE-seq-traversal-one-door.md`):

> *"The twins are a workaround for the missing type, not a pattern."*

That stone chose **native** as the cure, scoring the `Seqable` route a flat **NO on Simple** on
three named blockers. **All three were refuted or dissolved by stone 118.3-B (`a15f4ea9`)** — and
the artifact that FILES the decision was never updated:

```
109-kill-std/NOTE-seqable-has-no-name-in-wat.md    REFUTED/DISSOLVED × 0   (last touched 2026-07-31)
src/collection/infer.rs                            REFUTED/DISSOLVED × 3   (2026-08-17)
```

`a15f4ea9` is a descendant of `ac26d172`. So the record said *blocked* while the code said
*refuted* — a live stale blocker, the second of this class in two days
(`[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`). The note is **amended as of today**
with per-claim annotations; the original text is preserved as filed.

**The fork is therefore re-posed, UNRULED**, both routes four-questioned against the numbers in §5:
`DESIGN-118.B-the-route-fork.md`.

## 8. WHAT THIS DOES NOT SETTLE

- **That deleting the memos reaches O(1) for population C.** Open — and see the correction below,
  because the record's account of this is wrong in a way that has been propagating.

  ⛔ **CORRECTION.** Four docs carry *"the prediction that removing the memo alone reaches O(1) was
  wrong; it reached eager parity."* **It was right.** `DESIGN-118.10`'s own table: memo-on 585 B/elem,
  memo-off **288** B/elem, eager `mapv` with **no stream at all** **288** B/elem. Memo-off equals a
  program containing no stream — **zero stream retention**, exactly as predicted. "Eager parity" is
  the success condition, because `into`/`mapv` materialize by contract and are O(n) under *any*
  implementation. That instrument could never have shown total O(1).
  **What remains genuinely open is population C** (the wat-closure generator, 3,124 B/elem, §5) —
  never run memo-off, and it is the population the builder's own idiom uses.
  Full argument: `DESIGN-118.B-the-route-fork.md` § shared premise.
- **Whether the three doors close.** A dialect ruling, and the builder's (§4).
- **`keep-stream`'s `None` arm** recurses directly inside its own `lazy` body, so a long run of
  dropped elements recurses in Rust. Pre-existing, unmeasured, and NOT stone B's scope — but it is
  the same silent-SIGSEGV class as tasks #58/#86 and should not be discovered by a user.
- **The `-stream` twins' migration is mechanical** (`if`+three-call → `match (next s)`), but it is
  8 functions across one file, and R21 says a multi-site structural `.wat` rewrite is a **wat-fix
  codemod**, not hand edits.
