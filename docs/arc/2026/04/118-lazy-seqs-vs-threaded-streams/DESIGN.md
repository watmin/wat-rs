# Arc 118 — Lazy seqs vs threaded streams

**Status:** DESIGN settled 2026-05-01 — implementation deferred.

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
