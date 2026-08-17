# DESIGN — 118.B, the route fork. ⚠ UNRULED. Written 2026-08-17 against `5e5e219e`.

> **Nothing here is chosen.** This document re-poses a decision whose recorded premise expired. If
> you are reading it looking for what we decided: we have not. The ruling is the builder's.

## What is being decided

Where the language's sequence walks live — and therefore whether the eight wat-side Stream walkers
are **migrated** to `next` or **deleted**.

The builder's question, 2026-08-17: *"do we need these `-stream` and `stream->` names at all? are
these a crutch we thought we needed?"*

**Answer to that question, and it is not the fork: NO, we do not need them, and this project already
ruled so** — on the builder's own challenge, 2026-07-31, in
`278/DESIGN-STONE-seq-traversal-one-door.md`:

> *"The twins are a workaround for the missing type, not a pattern."*

Both routes below delete all seven `-stream` twins. The fork is only **what replaces them.**

## Why the fork is open again

Arc 278 scored `Seqable` a flat **NO on Simple** and chose native. That NO rested on exactly three
blockers. **All three are dead** — refuted or dissolved by stone 118.3-B (`a15f4ea9`), measured
against the disk. See `109-kill-std/NOTE-seqable-has-no-name-in-wat.md`, amended today with the
per-claim annotations.

A 4–0 verdict whose sole disqualifier has evaporated is not a verdict; it is an inheritance.
`[[feedback_four_questions_cannot_see_a_shared_premise]]`

## ★ The reframe that makes this urgent — native walkers NEVER had the defect

Measured this session, by reading the loops that actually run: `lazy_take_stream`
(`src/collection/transform.rs:181`) and `eval_vec_drop`'s loop (`:246`, `:248`) call
`crate::stream::realize` **once** per cell and destructure the `Cons`. One force, one element.

**The three-call walk is a wat-side-only disease.** It exists because wat's entire Stream API was
`empty?` / `first` / `rest` — three verbs, three independent `realize` calls on one cell. Rust always
had the fused pull. `next` (118.11a) is wat finally being given the primitive Rust never lacked.

Consequence, and it is why this cannot wait: **stone B as currently drawn — "migrate the 8 walkers
to `next`" — is route-dependent work.** Under NATIVE those walkers are deleted, and migrating them
first is effort spent on code queued for removal.

## The eight walkers (measured, `wat/seq.wat` is the only file)

`stream->pvec:102` (**the drain** — every eager materializer funnels through it) · `reduce-stream:180`
· `interpose-stream:412` · `keep-stream:454` · `keep-indexed-stream:481` · `map-indexed-stream:512`
· `dedupe-stream:540` · `distinct-stream:571`.

Full evidence: `MEASURED-118.B-the-lair.md`.

---

# ROUTE 1 — NATIVE. The walks move to Rust; the wat walkers are deleted.

Each remaining lazy verb (`keep`, `keep-indexed`, `map-indexed`, `dedupe`, `distinct`, `interpose`,
plus `reduce`'s Stream arm and the `into` drain) becomes a Rust intrinsic dispatching on
`StreamContainer` at runtime and `extract_lazyable_elem` at check time — the shape `map`, `take`,
`drop`, `filter`, and `seqable->stream` already have. The twins and their ~29 identical `defclause`
arms are deleted outright.

**Obvious? YES.** It is already what five shipped verbs do. A sixth is not a new idea, and the
resulting signatures are unchanged for every caller.

**Simple? YES.** One body per verb, no new concept, no new type-system machinery. It deletes the
twins and the arms and adds nothing the substrate does not already run.

**Honest? NO.** This is the route's real cost and it must not be soft-pedalled. Native reaches the
**check** rung of the ladder, never `no-form` — the 278 stone says so in its own words: *"nothing
stops a new wat-level stage with per-container arms and a `rest`-walk tomorrow, and it would be
quadratic and green."* The rule "sequence verbs live in Rust" is a **convention**, and this project
has recorded what conventions do: `[[feedback_a_house_convention_can_be_the_mechanism_that_built_the_pile]]`.
The 278 stone tracked a lint to convert it to a wall; that lint does not exist. Worse, the route's
own filed risk is on disk and unaddressed: *"278 removes the pain that would motivate this… this
degrades from 'fixes a real defect' to 'improves legibility' — the class of work that never gets
scheduled."* Choosing native a second time is choosing it knowing that.

**Good UX?** *Unreached* — Honest failed. (Had it been reached: YES for callers, whose signatures
do not move; NO for anyone extending the language, who must write Rust to add a lazy stage to a
language that advertises self-hosting.)

---

# ROUTE 2 — SEQABLE. The type gets a name; each verb becomes one wat clause.

Mint `:wat::core::Seqable<T>` as a `defsurface`, `extend-type` the four containers, and let each
lazy verb be a single `defn` over `Seqable<T>` whose body walks with `next`. `extract_lazyable_elem`'s
hardcoded four-head match is deleted — `infer.rs`'s own doc already says that function *"is exactly
what that stone would delete."*

**Obvious? YES.** It is Clojure's `ISeq`, which is the stated familiarity target, and R28's own
model of what a surface is. One `keep`, not five arms plus a twin.

**Simple? YES — and this is the reversal.** In 278 this was the flat NO. The three blockers that
produced it are dead. What remains is a ~20-line `defsurface` plus four `extend-type`s, on a
mechanism (parametric surface satisfaction) that landed yesterday and is green.

**Honest? YES.** It reaches the **no-form** rung, which native cannot: a sequence verb that never
names a container cannot hand-roll a per-container walk. The difference the 109 note draws is
exact — *"'sequence verbs should be native' versus 'a sequence verb cannot see a container.'"*
It also deletes the concept's duplicate spelling: today "what is seqable" lives in the checker AND
implicitly in ~29 `defclause` arms.

**Good UX? YES.** A user can add a lazy stage in wat, in the language they are writing, without
touching Rust. Under native they cannot.

**⚠ The cost, measured today and not hidden:** the walks stay interpreted. At n=400,000, identical
exact sums, the wat-closure path is **3,124 B/element** against the native path's **343 B** — 9.1× —
and **5.8× the wall clock**. That is population C in `MEASURED-118.B-the-lair.md`. Route 2 keeps the
language's own sequence verbs in the expensive tier **until the bytecode compiler lands.**

---

## ★ THE SHARED PREMISE — what neither route's four questions can see

The four questions discriminate BETWEEN options; they never validate what BOTH rest on.
`[[feedback_four_questions_cannot_see_a_shared_premise]]`

Both routes rest on: **once no three-call walker remains, both memos can be deleted and retention
goes to O(1).** That is **UNPROVEN**. It is the prediction the whole tier is aimed at, and the last
prediction in this area — that removing the memo alone reaches O(1) — was **wrong**; it reached
eager parity.

If that premise is false, neither route delivers the memory fix, and the fork was about code
organization all along. **It should be probed before either route is struck**, and it is cheap:
build a throwaway no-memo variant and run the four-point RSS series that already exists
(`wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat`).

Also shared, and true: `next` ships under both routes. Users write their own producers and consumers
either way, so the pull primitive is not route-dependent. 118.11a was correctly scoped.

## What is common to both, and therefore buildable NOW

1. **`next` exists.** Landed, `0d651715`.
2. **The three doors** (`first`/`rest`/`empty?` on a Stream) are a 3× hazard for *user* code the
   moment the memos die, wherever the stdlib's walks live. That question is route-independent —
   and it is a **dialect ruling**, also the builder's. ⚠ `empty?` cannot be walled at compile time
   without changing its `∀T. T -> bool` scheme; `first` and `rest` are one capability bit each.
3. **`stream->pvec` / `stream->vec` should not be public `:wat::core::` names** whatever replaces
   them. Both say *"internal helper"* in their own doc comments while sitting in the user-facing
   namespace. Clojure has no such name; it has `into` and `vec`.
4. **The no-memo probe** above — it validates the shared premise and is owed either way.

## Recommendation — and it is a recommendation, not a ruling

**Route 2.** Route 1 fails Honest, and it fails it on a defect this project has already named and
paid for: it ships a convention where a wall is possible, and its own filed risk says the follow-up
never gets scheduled. Route 2 deletes strictly more than route 1 (the same twins and arms, **plus**
`extract_lazyable_elem`), moves the concept out of the host and into the language, and reaches the
rung native cannot.

The honest counter is the 9.1× — **route 1 is the performance answer today, route 2 is the language
answer, and route 2 becomes the performance answer when the bytecode compiler lands**, which is the
direction already named: *"the surface will be our expression language for optimized code it
produces… interpreted wat has a death sentence."*

If the ruling is route 2, stone B's migration is real work and its 8-walker scope stands. If route 1,
stone B shrinks to a deletion and the migration should never be written.

## What this document does NOT decide

- Whether the three doors close (dialect, builder's).
- Whether the memos can actually die (unproven premise — probe it).
- `keep-stream`'s `None` arm recurses in Rust on a long run of drops — pre-existing, unmeasured,
  same silent-SIGSEGV class as tasks #58/#86, and out of scope for either route.
- Bootstrap circularity keeps `map`/`take`/`drop` native under **both** routes
  (`:wat::core::defn`'s own macro body calls them at expansion time). Neither route touches that.
