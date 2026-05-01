# Arc 118 — Lazy seqs vs threaded streams

**Status:** scoping (2026-05-01) — captured during arc 109 K.telemetry mid-slice.

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
