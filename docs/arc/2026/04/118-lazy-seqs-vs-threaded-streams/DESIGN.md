# Arc 118 — Lazy seqs vs threaded streams

**Status: ▶▶ RECLAIMED 2026-06-27 — BUILD IT, COMPLETE.** The implementation deferred 2026-05-01 is reclaimed:
**arc 295's chunk-read signed eval forced it.** Signed eval takes a length-bounded byte stream over the wire →
that stream is a lazy seq → which finally builds this arc → which annihilates the thread-per-stage HOFs. We build
118 out **fully, then return to 295** (the signed-eval rides the finished substrate). Design strategy was settled
2026-05-01 (Option C, below); now it ships.

> **The 2026-06-27 directive — NO HALF-DELIVERY.** *"we deliver lazy-seqs in their honest, obvious, simple and good
> ux forms — exactly as they would be expected to be placed and used."* The whole faithful family lands together
> (lazy transformers · eager materializers · forcers · effect-/value-consumers), four-questions-clean, placed and
> named where a Clojure dev reaches for them. Not a security-only slice with the rest owed.
>
> **⊘ SUPERSEDED → RESOLVED 2026-06-27: Surface C (clojure-faithful), one home.** The 2026-05-01 namespace
> distinction below (`:wat::seq::map` lazy / `:wat::list::map` eager) is **superseded.** Four-questioned against the
> NOW-grounded state (eager HOFs already split + duplicated: `:wat::core::{map,filter,take,drop,concat,reduce}` AND
> `:wat::list::{fold,reduce}` — `reduce` in both; `:wat::list::*` half-materialized in `wat/list.wat`), Surface C won
> all four: **`:wat::core::map` is LAZY** (= `clojure.core/map`), **`:wat::core::mapv`/`filterv` eager → vector** (=
> `clojure.core/mapv`), `reduce`/`into` eager-to-value, `iterate`/`repeat`/`cycle` lazy-∞ — **everything in
> `:wat::core::*`.** Obvious (a Clojure dev *knows* core/map is lazy) · Simple (one home; **retire `:wat::list::*`**,
> kill the duplicate `reduce`) · Honest (mirrors `clojure.core`; the namespace-signal of N was a wat-ism that lies to
> a Clojure dev) · Good-UX (*"exactly as a clojure dev expects them placed and used"* — the builder's directive).
> The flip migrates the ~36 eager `:wat::core::map`/`filter` sites to lazy (most just-consume → free; the rest →
> `mapv`/`for-each`). **Builder: *"C is the call — retire :wat::list:: and flip to clojure-faithful."*** The
> namespace-distinction section below is kept (amend-with-recognition; the reasoning taught) but is NOT the design.
>
> **Annihilation target:** `wat/stream.wat` (the `:wat::stream::*` thread-per-pure-stage HOFs — *built wrong,
> successfully*) → reimplemented over the lazy family; threads survive ONLY where a stage guards mutable state.
> **Open questions § (seq repr · termination · error-prop · seq↔list interop A/B/C · naming) must be settled first.**

> **⚡ DECIDED 2026-06-27 DURING THE BUILD — SINGLE-PASS, NO MEMOIZATION (supersedes the memoized foundation).** The
> first foundation strike (`src/seq/mod.rs`, sonnet 118.1) built `LazyCell` with an `OnceLock<Arc<Seq>>` memoize
> (Clojure-faithful: persistent, re-traversable). **The builder overrode it:** *"i do not believe we should have
> memoize at all … you cannot walk back a stream — if you want this, you gotta write it, you go solve the rewind
> buffer — core does not ship it."* So a wat lazy seq is a **single-pass STREAM, not Clojure's persistent lazy-seq:**
> - **Drop the `OnceLock`.** `LazyCell = { thunk: Arc<Function> }` only; `realize` forces and returns, **no caching.**
> - **Holding-the-head footgun EVAPORATES** (no cache to pin) — constant-memory streaming is now *unconditional*
>   (terabytes through a teacup, no "don't hold the head" caveat).
> - **Re-traversal / rewind is NOT shipped** — its **absence is the enforcement** (no runtime check, no policy). Want
>   to walk it twice? Build a rewind buffer yourself; *"you probably don't want it."* The eager re-traversable world
>   is `:wat::list::*`.
> - **Consumer discipline is STRUCTURAL, not policy:** `fold`/`reduce`/`for-each` are *implemented* tail-recursive /
>   head-dropping (the only correct way to write a streaming fold) — NO enforcement check (the footgun isn't shipped,
>   so there's nothing to police). Pin this in the HOF brief.
> - **NAMING (intueri, once green):** `stream` may be the more honest noun than `seq`/`lazy-seq` — single-pass diverges
>   from Clojure's persistent seq, so `:wat::core::seq`/`lazy-seq` reads Clojure-misleading. Weigh `stream` at the rename.
> **Action when the 118.1 sonnet lands: weigh → rip the `OnceLock` out of `LazyCell` + make `realize` non-caching →
> re-verify the probe + the laziness test → commit.** Removing memoization is strictly LESS code; the cascade +
> primitives stay valid. **✅ DONE 2026-06-27 — committed `74883c15` (single-pass, no memo; probe GREEN, gate 4089/0).**

> **⊘ SUPERSEDED → RESOLVED 2026-06-27: the TWO-WORLD SPLIT — `:wat::seq::*` (eager) + `:wat::stream::*` (lazy
> single-pass). Surface C above is itself superseded.** Once memoization was removed (single-pass), the lazy thing
> stopped being Clojure's persistent lazy-seq and became a **stream** — and Surface C's whole rationale (put `map`
> lazy in `:wat::core::*` to be *clojure-faithful*) collapsed with it: a single-pass stream named `core/map` reads as
> a persistent lazy seq, which is the lie. The governing principle, from the builder (2026-06-27): **"we do not strive
> to be clojure — we strive to be FAMILIAR. we reserve the rights to choose our own names and behaviors. wat is a
> DIALECT of clojure, not an impl."** So the bar is **familiar + internally-consistent, NOT faithful** — which
> dissolves the "this namespace lies to a Clojure dev" objection that founded Surface C's rejection of the
> namespace-split (Surface N). That objection measured against the WRONG bar (faithfulness). By wat's own right:
> - **`:wat::seq::*` = eager, materialized, re-traversable** (renamed from `:wat::list::*`; the eager
>   `:wat::core::{map,filter,take,…}` consolidate here). `seq` = "sequence" = ordered materialized collection — wat's
>   word, not Clojure's lazy-abstraction word.
> - **`:wat::stream::*` = lazy, single-pass, consumed-once** (the `lazy-seq` idea, *annihilated → reborn as stream*;
>   already the noun on disk — 295 eval-side types the byte stream `(:wat.type/Stream u8)`).
> - **The namespace IS the cost signal** — single-pass is a sharp footgun (no rewind), so making it scream at every
>   call site (`stream/map` ≠ `seq/map`) is a SAFETY feature. Surface N's namespace-signal, vindicated by single-pass.
> - **Foundation rename (fix-wat codemod, NOT a rebuild):** the committed `Value::wat__core__Seq` / `LazyCell` /
>   `:wat::core::{cons,lazy-seq,seq-empty,first,rest}` → a `Stream` family under `:wat::stream::*`; `:wat::list::*` →
>   `:wat::seq::*`. Declare the convention loudly in the docs (every dialect documents its vocabulary) — that is the
>   whole "caveat": ordinary documentation, not a debt owed to Clojure.

> **⚡ DECIDED 2026-06-27 — the PRODUCER model: the FUNCTIONAL producer is the SOLE solution; the stream surface is
> CEK-STABLE; the imperative generator is a named CEK-era ADDITIVE follow-on (NOT a thread).** Working the producer
> UX (the builder's Ruby `Enumerator.new { |y| y << 1 … }` shape) drove it to the floor:
> - **Suspension without fibers = the thunk.** A functional producer's "where was I" is a *closure over its env*
>   (`stream/lazy`), NOT a stack frame — so lazy PULL composition (`take`/`zip` over an infinite producer) works with
>   **no thread and no fiber**. We built suspension-free-of-fibers.
> - **The thread-backed generator is STRUCK.** A thread merely holds a suspended *stack* — a fiber you pay an OS
>   thread for. The moment the producer is functional, the thunk replaces the stack and the thread is pure waste —
>   exactly what this arc exists to kill. (An earlier floated "thread-backed `generate`" is retracted.)
> - **CEK-STABILITY INVARIANT (the governing design law):** *the stream surface must not change when the runtime
>   swaps to a CEK.* It rides only **closures + application** — which every evaluator has (tree-walk today, CEK later)
>   — and deliberately uses **no reified continuation** (absent now, present later — a phantom to depend on) and **no
>   thread** (rip-out-later). So the CEK migration is a **no-op for stream code.** Every future stream addition is
>   held to this law: rides closures+application → ship it; needs the K → name it a CEK-era additive follow-on; needs
>   a thread → reject (unless it genuinely guards mutable state per the arc-118 metric).
> - **The imperative yielder (`stream/generate [yield] …` with arbitrary control flow) is a CEK-era ADDITIVE
>   follow-on.** It needs a reified continuation (capture K at `yield`, resume on `pull`) — a CEK feature. When it
>   lands it produces the **same `Stream` value** (same `first`/`rest`/`map` consume it): it ADDS a constructor,
>   changes nothing existing. Until then: **don't fake it with a thread.** Known/eager items use `stream/of`;
>   stateful-lazy production waits for the CEK.
> - **Two threadless consumption directions:** PULL (consumer asks; thunk forces the tail; lazy `take`/`zip` compose
>   naturally) and PUSH (producer drives via TCO recursion, calling a `yield-fn` per item; `reduced` gives early-exit
>   so `take` can short-circuit a push producer). `for-each`/`reduce` are the push side.
> - **Honest caveat (robustness, NOT a surface change):** under today's tree-walker, *deeply*-recursive realization
>   (e.g. `filter` skipping a huge span) re-enters the Rust evaluator per step and is bounded by the Rust stack; the
>   CEK heap-allocates continuations and lifts that ceiling. The CODE is identical — "more robust after the swap,"
>   never "different after the swap."

This arc closes as DESIGN-only before arc 109 is marked resolved.
The decision is locked: **lazy seqs implemented as
closures + recursion + thunks (Option C below)**, with an
optional generator macro layer for imperative-flavored ergonomics.
Implementation work happens in a future session.

**Relationship to arc 004:** Arc 004 (Lazy Sequences and Pipelines)
already established Ruby's `Enumerator.new` + `Enumerator::Lazy` as
the conceptual reference and proposed translating it to wat by
**substituting an OS thread for each Fiber**. Arc 118 REFINES arc
004 with the user's metric: not every stage needs a thread —
threads exist to guard mutable state. Pure-functional stages
(map, filter, take) don't guard anything; they shouldn't pay the
thread cost.

The reference example arc 004 carries:

```ruby
producer = Enumerator.new do |yielder|
  loop do
    batch = fetch_next_page
    batch.each { |item| yielder << item }
    break if last_page?
  end
end

result = producer.lazy.map { |i| transform(i) }.each_slice(50).first(10)
```

Ruby's Fibers run all stages on one thread; the consumer's pull
drives the producer through `yielder << item`. Arc 004 said
"replace each Fiber with an OS thread + bounded(0) channel." Arc
118 says: do that ONLY when the stage holds state. Pure stages
collapse onto the consumer's thread.

## The question

> we only use threads to guard mutable state — the metric
>
> if the producer isn't guarding mutable state they don't need to be in a thread

The current substrate ships `:wat::stream::*` (post-slice-9d) — a
collection of HOFs over channels: `spawn-producer`, `from-receiver`,
`map`, `filter`, `inspect`, `fold`, `for-each`, `chunks`, `chunks-by`,
`take`, `flat-map`, `with-state`, `drain-items`, `collect`, `window`.

**Every one of these spawns a thread.** Each `map`, `filter`,
`inspect`, etc. is implemented as a worker thread that reads from
an upstream channel and writes to a downstream channel. Pipelines
of N stages = N threads + N channels.

The user's metric: **threads exist to guard mutable state.** When a
stage is purely functional (map: T → U, filter: T → bool, fold: a
pure reducer), there is NO state to guard. Spawning a thread is a
performance overhead with no semantic justification.

## What's needed

**Two complementary primitives:**

### Lazy sequences (pull-based, single-threaded)

```
:wat::seq::*
```

The Clojure-flavored answer: `seq` is a sequence of values that are
computed on demand. `(seq/map f xs)` returns a NEW seq whose nth
element is `(f (xs nth))` — computed when consumed, not when the
seq is constructed. Pure functional; no threads, no channels.

Operations: `map`, `filter`, `take`, `drop`, `take-while`,
`drop-while`, `concat`, `flatten`, `iterate`, `repeat`, `range`,
`reductions`, `partition`, `interleave`, `zip`, etc.

Termination: a seq ends when its source ends. Consumers iterate
with `seq/first` + `seq/rest` (or via fold/for-each which terminate
at end-of-seq).

**Cost model:** zero thread overhead; entire pipeline runs on the
consuming thread; back-pressure is implicit (the consumer pulls).

### Threaded streams (push-based, multi-threaded)

```
:wat::stream::*  (today's substrate)
```

When a stage NEEDS a thread — because it's:
- bridging a channel boundary (kernel comm, fork-program output)
- guarding mutable state (rate limiting, dedup with internal map)
- doing async I/O (file read, network read)
- using a hardware resource (mmap, ipc shm)

Threads have semantic justification.

**Cost model:** one thread per stage, channels between stages, real
parallelism, server-style back-pressure (channel bound).

## Implementation strategy — closures, NOT fibers (settled 2026-05-01)

A common reflex on hearing "lazy seqs in the Ruby Enumerator
shape" is to reach for fibers (stackful coroutines). The user
asked: "do we need to impl fibers proper to enable this?"

**No — fibers aren't needed.** Lazy seqs in Clojure/Haskell are
**closure-based, not fiber-based**. Three implementation
strategies were on the table; the gaze converges on the third.

| Strategy | How producers express themselves | Runtime cost | Substrate addition |
|---|---|---|---|
| **A — Threads** (today's `:wat::stream::*`) | imperative loop in `spawn-producer` body | OS thread + channel per stage; thread spawn ~15-50µs | already shipped; not honest for pure stages |
| **B — Fibers** | imperative `loop do … yielder << x end` (Ruby flavor) | stackful coroutine + stack-switching primitive | NEW — would require either external Rust lib or hand-rolled context-switch |
| **C — Closures + recursion** (Clojure/Haskell flavor) | recursive function returning `Cons(head, lazy-tail)` where lazy-tail is a closure | minimal — wat-rs already has closures, structs, TCO | minimal — `Seq<T>` enum (Cons \| Nil) + thunk + `force` operation |

### Why C wins the four questions

- **Obvious?** Recursion + closure is the Lisp-canonical shape.
  No new control-flow primitive; readers see "function returns
  data."
- **Simple?** Smallest substrate addition. `Seq<T>` enum is a
  struct definition + a `force` operation. No stack-switching, no
  async runtime, no fiber scheduler.
- **Honest?** Reach for the smallest mechanism that works.
  Closures + recursion is what the substrate already has;
  fibers would be a new runtime entity introduced just for
  Ruby ergonomics.
- **Good UX?** Pure functional pipelines compose naturally.
  For users who want Ruby-imperative ergonomics, a generator
  macro can rewrite `(generator ... (yield x) ...)` into the
  recursive form at macro-expand time. **Same surface, no new
  runtime.**

### Clojure's pattern, in case the recursive shape isn't familiar

```clojure
(defn naturals
  ([] (naturals 0))
  ([n] (lazy-seq (cons n (naturals (inc n))))))
```

- `lazy-seq` wraps the body in a thunk.
- `cons` returns a Cons cell with `head = n` and `tail = thunk`.
- When the consumer forces the tail, the thunk runs and returns
  another `lazy-seq` of `(cons (inc n) ...)`.
- No fiber, no yield, no suspension primitive. Just a function
  that returns data, which happens to contain a closure that,
  when forced, returns more data.

**Wat translation sketch (post-118):**

```scheme
(:wat::core::define
  (:wat::seq::naturals (n :wat::core::i64) -> :wat::seq::Seq<wat::core::i64>)
  (:wat::seq::cons-lazy
    n
    (:wat::core::lambda () -> :wat::seq::Seq<wat::core::i64>
      (:wat::seq::naturals (:wat::core::i64::+ n 1)))))
```

The `cons-lazy` constructor takes a strict head and a thunk for
the tail. Force the tail when consuming.

### Why Ruby reaches for fibers (and why wat doesn't need to)

Ruby's `loop do … yielder << x end` is **imperative control
flow** — the `loop` keyword IS a construct that has to suspend
mid-iteration. Fibers exist to make the imperative shape work.

Wat is a Lisp. Recursion IS the loop. Each "next iteration" is
literally the next call. Suspension is just "this thunk hasn't
been forced yet." The imperative-vs-recursive choice is a
language-shape decision, and Lisp's shape doesn't need fibers.

### The macro layer (optional, future)

Users who prefer Ruby ergonomics can have them via a
`(:wat::seq::generator ...)` macro that rewrites `yield`
calls into the recursive `cons-lazy` form at expand time:

```scheme
(:wat::seq::generator
  (:wat::core::let* ((batch (fetch-page)))
    (:wat::core::for-each batch
      (:wat::core::lambda (item) (:wat::seq::yield item)))))
;; macro-expands to recursive lazy-seq returning each item
```

The macro is a SURFACE convenience over the recursive runtime —
no fiber, no stack-switching, just AST rewriting.

## The design tension

How do the two interoperate? Three possibilities:

### Option A — fully separate
- `:wat::seq::*` is the pure-lazy world
- `:wat::stream::*` is the threaded world
- Conversion verbs: `seq->stream` (spawn one thread, push elements
  one-by-one onto a channel) and `stream->seq` (pull from a
  channel, lazily)
- User picks the world; verbs don't compose across worlds

### Option B — unified API, transport selected by pipeline
- One namespace; verbs are seq-flavored by default; you opt into
  threading by wrapping the source in a `(thread ...)` form
- Compiler/runtime picks transport based on whether any stage is
  marked-threaded
- Optimization-driven; user thinks in seqs

### Option C — strict layering
- Lazy seqs are the substrate primitive (`:wat::seq::*`)
- Threaded streams are a wrapper on top (`:wat::stream::*` rebuilt
  to host a seq inside a thread, OR mints `seq->thread` /
  `thread->seq` adapters)
- Today's `:wat::stream::*` HOFs become thin wrappers that lift
  seq HOFs into a threading discipline

Gaze likely says: **A is honest** (the two protocols are genuinely
different; merging them lies about what each does). B is convenient
but loses information. C may be the implementation strategy
underneath A's surface.

## `:wat::list::*` vs `:wat::seq::*` — justifiably different (settled 2026-05-01)

A natural follow-up question: arc 109 § H proposes `:wat::list::*`
for HOFs over Vec<T> (map, foldl, filter, sort-by, etc.). Arc 118
proposes `:wat::seq::*` for lazy HOFs over Seq<T>. **Are these
duplicates?**

**No — they're justifiably different.** The distinction is
**eager vs lazy**, which is a real runtime-cost / memory /
error-timing distinction visible at every call site.

| | `:wat::list::*` | `:wat::seq::*` |
|---|---|---|
| Operates on | `Vec<T>` (materialized) | `Seq<T>` (lazy thunks) |
| `(map f xs)` evaluates `f` | NOW, for every element | WHEN PULLED, lazily |
| Memory | proportional to N | proportional to consumed prefix |
| Error timing | up-front (eager) | per-element (deferred) |
| `(sort-by xs)` | natural — eager sort over a Vec | requires forcing first; "lazy sort" is meaningless |
| `(iterate f x)` | meaningless — iterate is infinite | natural — produces an infinite Seq |

### Why polymorphism (one `:wat::poly::map`) loses information

A reader sees `(:wat::poly::map f xs)`. To know if it's eager or
lazy, they have to find `xs`'s type. The eagerness signal is
hidden. Calling them both "map" and dispatching erases a real
semantic distinction. Through the four questions:

- **Obvious?** No — forces lookup of xs's type.
- **Simple?** Apparent simplicity (one name); actual complexity
  (the runtime cost depends on operand type).
- **Honest?** No — different operations dressed as one.
- **Good UX?** Worse — call sites become ambiguous.

### Op overlap and uniqueness

| Op | list | seq | Notes |
|---|---|---|---|
| `map` | ✓ | ✓ | both natural |
| `filter` | ✓ | ✓ | both natural |
| `take` / `drop` | ✓ | ✓ | both natural |
| `concat` | ✓ | ✓ (lazy-cat) | seq variant doesn't materialize |
| `fold` / `foldl` | ✓ | ✓ | seq variant forces while folding |
| `for-each` | ✓ | ✓ | terminal in both |
| `sort-by` | ✓ | ✗ | sort needs all elements; can't be lazy |
| `find-last-index` | ✓ | ✗ | requires materialized index |
| `last` / `reverse` | ✓ | ✗ | last needs to walk to end; reverse materializes |
| `iterate` | ✗ | ✓ | infinite generator; only meaningful lazily |
| `repeat` / `cycle` | ✗ | ✓ | same — infinite |
| `take-while` / `drop-while` | (?) | ✓ | could be eager too; arguable |
| `partition` / `interleave` | ✓ | ✓ | both natural |

### Conversion verbs

```
:wat::seq::from-vec  (Vec<T>) → Seq<T>          ;; lift; no eval
:wat::seq::collect   (Seq<T>) → Vec<T>          ;; force + materialize
```

User picks the world; conversion verbs join them at the boundary.

### Clojure precedent

Clojure already made this call: `clojure.core/mapv` (eager,
returns vec) vs `clojure.core/map` (lazy by default, returns
seq). Same lesson — eagerness deserves its own name. Wat's
naming is cleaner because the namespace itself signals the
eagerness, no `v` suffix needed.

User direction (2026-05-01):

> in the 118.. and 109.. we have :wat::list::* being declared..
> do we need a :wat::seq::* as well.. the two are justifyably
> different?.. (i think so.. but we need scrutiny..)

Scrutiny applied. Convergence: keep both.

## Open questions

> **⊘ RESOLVED 2026-06-27 (kept for the record; the questions taught). The build + this session's decisions closed
> all but one, and the remaining one is strike-survey, not a design fork:**
> - **Q1 (representation / memoized?)** → RESOLVED: `Seq = Empty | Cons{head, tail:Arc<Seq>} | Thunk(LazyCell{thunk})`,
>   **single-pass, NO memoization** (built, `74883c15`).
> - **Q2 (termination)** → RESOLVED: a sentinel `Seq::Empty` variant (`seq-empty` / `stream/empty`), NOT `Option`. Built.
> - **Q3 (error propagation)** → RESOLVED-by-build: a thunk body that errors propagates as the normal `EvalBreak`
>   through `realize` — no forced `Result<T,E>` element type; errors ride the evaluator's existing path.
> - **Q4 (channel interop)** → COLLAPSED under the functional model: a channel source is a **functional producer** —
>   a thunk that `recv`s one item on force, then `lazy`-tails; the consumer's pull drives the `recv`. **No producer
>   thread** unless the reader genuinely guards state (the arc-118 metric). The "which do I pick" fork dissolves.
> - **Q5 (holding the head)** → RESOLVED + EVAPORATED by single-pass: no cache to pin → holding the head pins nothing
>   → constant-memory streaming is unconditional.
> - **Q7 (naming)** → RESOLVED: the two-world split — `:wat::seq::*` (eager) / `:wat::stream::*` (lazy). Not
>   `:wat::lazy::*` / `:wat::iter::*`.
> - **Q6 (which existing `:wat::stream::*` consumers benefit)** → the ONLY remainder, and it is **strike-survey, not a
>   design decision** — enumerated DURING the `wat/stream.wat` annihilation (the HOF-family strike), not before.
>
> The "forced hand" — building **CEK-stable** — closed the last real architectural fork (the producer shape:
> functional-only, thread struck, imperative generator = CEK-additive). **No design questions remain open;** what is
> left is build-roster execution (the HOF family + the annihilation survey).

1. **What's a seq's runtime representation?** A struct holding a
   thunk + a force/realized state? An enum (Cons | Nil | Lazy)?
   Memoized? Garbage-collected? Wat-rs is immutable + Arc-based;
   memoization needs an interior `OnceLock` or similar.
2. **Termination signaling.** A seq's `rest` returns either another
   seq (more values) or `:None` (end). Does the substrate use
   `:Option<Seq<T>>` or a sentinel `Nil` variant?
3. **Error propagation.** Can a seq element fail? If yes, the seq's
   element type becomes `:Result<T, E>` and consumers handle errors
   per-element. The Clojure equivalent is exceptions; wat-rs
   doesn't have those.
4. **Interop with channels.** `seq/from-receiver` (pulls a value
   per realization; thread blocks on the receiver) vs
   `stream/from-receiver` (one thread reads continuously into a
   channel-fed stream). When does the user pick which?
5. **Memory.** Lazy seqs that are held during traversal can pin
   the entire prefix in memory. This is the "holding the head"
   bug Clojure programmers know. Does wat-rs's strict-evaluation
   make this worse or better?
6. **Existing `:wat::stream::*` consumers.** Which would benefit
   from a seq variant? The trading lab's pipelines? Telemetry's
   batch dispatch? The ddos lab's packet pipeline?
7. **Naming.** `:wat::seq::*` vs `:wat::lazy::*` vs `:wat::iter::*`.
   Each communicates differently. Gaze should weigh in once a
   concrete shape emerges.

## Why this is a NEW arc, not arc 109

Arc 109 is naming + filesystem cleanup. Renames + path moves +
walker-driven sweeps. Mechanical, doctrine-driven, mostly
sonnet-delegatable.

Arc 118 is **substrate design** — adding new primitives, working
out semantics (laziness, termination, error propagation), and
making interop decisions that ripple through every consumer that
uses streams. Different kind of work.

Mixing them muddies both:
- 109's clean cleanup gets dragged into design conversations
- 118's design gets fragmented across mid-slice micro-decisions

## Recommended sequencing

1. **Finish arc 109 cleanly.** K.telemetry → K.console → K.lru →
   K.holon-lru → K.thread-process → § J 10d-g → INSCRIPTION.
   Substrate ends in a clean naming + filesystem state.
2. **Then arc 118 from a clean baseline.** Substrate has honest
   names; gaze finding A/B is captured; § J's typeclass dispatch
   exists (which lazy-seqs may want to ride on).
3. **Possibly fold parts of arc 118 back into 109's INSCRIPTION** —
   e.g., a forward-pointer in § G's three-tier substrate
   organization noting `:wat::seq::*` as a future tier.

If the lazy-seq insight changes how K.console / K.lru consumers
are written — those consumers might prefer seqs over threaded
streams — that REFINES the K slices but doesn't change their
shape (the K slices are about Service-grouping flatten + channel-
naming patterns; lazy-seq is orthogonal).

## User direction (2026-05-01)

> i want lazy seqs and threaded streams...
> does streams need to be delegated to a thread?... that's the question
> we only use threads to guard mutable state — the metric
> if the producer isn't guarding mutable state they don't need to be in a thread
> i think i want this handled in 109... maybe .... maybe we pivot
> into this new arc before wrapping 109..

## Cross-references

- `wat/stream.wat` — the current threaded `:wat::stream::*` HOFs.
- `docs/SERVICE-PROGRAMS.md` — discusses thread-driven service
  patterns; relevant to "threads guard mutable state" framing.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K — the channel-
  naming patterns; arc 118's seq-vs-stream interop reuses
  Pattern A/B vocabulary for the threaded side.
- Clojure's `clojure.core/lazy-seq` + `seq` interface — the
  intellectual reference; arc 118 draws from there explicitly.
