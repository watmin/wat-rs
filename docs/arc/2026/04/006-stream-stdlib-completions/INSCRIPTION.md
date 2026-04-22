# Arc 006 — Stream Stdlib Completions — INSCRIPTION

**Status:** slices 1 + 2 + 3 shipped 2026-04-20; slice 4 (with-state
substrate + chunks-on-with-state surface-reduction proof) shipped
2026-04-21 alongside arc 009 (names are values), which closed the
fn-by-name gap that with-state's ergonomics depended on. Slice 5
(chunks-by + window as library code on with-state) shipped
2026-04-21. **Arc 006 closes.** Remaining items — time-window,
from-iterator, Level 2 iterator surfacing — are substrate-blocked
on primitives that don't exist yet (clock, iterator-trait surface)
and earn their own arcs when callers demand.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — full classification of
arc 004's deferred set.
**This file:** completion marker for the trivial pattern-completion
slice.

Same inscription discipline as arcs 003, 004, 005: DESIGN is pre-ship
intent; INSCRIPTION is the shipped contract. This arc has no DESIGN
file — the items are pattern-completions of combinators established
by arc 004, so the BACKLOG is sufficient scaffolding.

---

## What shipped

Two 1:N / 1:1 stream combinators added to `wat/std/stream.wat`,
following the same shape as map/filter/chunks (worker pair: outer
wrapper sets up the bounded(1) queue and spawns the worker; worker
is a tail-recursive program parameterized by upstream Receiver,
downstream Sender, transform, and any carried state).

### `:wat::std::stream::inspect<T>`

Signature: `:Stream<T> × :fn(T)->() -> :Stream<T>`.

1:1 side-effect pass-through. The worker pulls each item, calls `f`
for its effect (return type `:()`), and forwards the original value
unchanged downstream. Same worker shape as map except the send uses
`v` instead of `(f v)`.

Use cases: logging, tracing, metrics counters, debug breakpoints in
a pipeline — any "observe without modifying" pattern.

### `:wat::std::stream::flat-map<T,U>`

Signature: `:Stream<T> × :fn(T)->Vec<U> -> :Stream<U>`.

1:N expansion — symmetric with `chunks` (N:1). For each upstream
item, apply `f` to get a `Vec<U>`; emit each element downstream.
Empty vec from `f` emits nothing for that input (0:1 sub-case).

The worker carries a `pending` parameter — items produced by the
most recent expansion that haven't been sent yet. State machine:

- If `pending` empty: pull next upstream item; if Some, recurse
  with `pending = (f v)`; if None, exit.
- If `pending` non-empty: send its first item; if send succeeded,
  recurse with `pending = (rest pending)`; if None (consumer
  dropped), exit.

One function, state threaded through the parameter — same pattern
arc 004's chunks uses for its buffer. Mutual recursion with a
helper was considered and rejected; the single-function state
machine is clearer.

## Tests

5 new tests in `tests/wat_stream.rs`:

- `inspect_passes_values_through_unchanged` — no-op inspect,
  collect equals source.
- `inspect_composes_between_map_and_collect` — four-stage
  pipeline (map → inspect → map → collect); inspect in the middle
  must be transparent.
- `flat_map_expands_each_input_to_two_outputs` — 3 inputs produce
  6 outputs (n, n*10).
- `flat_map_empty_expansion_emits_nothing` — 0:1 sub-case: every
  expansion returns empty, collect returns empty.
- `flat_map_mixed_expansion_sizes` — variable expansion (3, 0, 2)
  → 5 outputs in input order.

All 16 stream tests pass; full suite (490+ tests) passes.

## Why this slice existed

Arc 004 INSCRIPTION deferred a combinator set under stdlib-as-blueprint
discipline — ship when a real caller demands, not speculatively. A
second look at the deferred list split it into three shapes:

1. **Trivial pattern completions** — no design questions; symmetric
   with shipped combinators. Deferring these is just leaving obvious
   gaps; shipping them is completing a pattern.
2. **Real design questions** — shutdown semantics, boundary
   behavior, handle ownership. Shipping without a caller means
   designing in the dark; legitimate defer.
3. **Substrate-blocked** — depend on primitives not yet shipped
   (clock, iterator surfacing). Genuinely separate arcs.

This slice ships shape 1 only. Shapes 2 and 3 stay in the BACKLOG
with their blocking conditions named.

## Slice 2 — `:wat::std::stream::take` (ex-`first`, reframed as a stage)

**The question:** `first(stream, n) -> Vec<T>` as a terminal
deadlocks against an infinite producer — the caller still holds
`stream`, the Receiver Arc never drops, the producer's send never
returns `:None`, the join never completes. Current kernel `drop`
on channels is a runtime no-op, and wat has no `std::mem::drop`
equivalent to force-release a binding mid-function.

**The resolution:** make it a stage, not a terminal. `take(stream,
n) -> Stream<T>` spawns a worker that counts down from `n` and
forwards each item. When the worker exits (either because `n`
items passed through, or because upstream ended early), its `Sender`
and `Receiver` drop naturally via spawn-closure scope exit. The
drop cascade fires the same way it does for map, filter, chunks.
No kernel change required. The pattern Rust's `iter.take(n).collect()`
already has.

**What this taught.** wat's absence of a force-drop isn't a gap
to patch — it's a discipline. The scope discipline IS the shutdown
discipline. A combinator that "needs" to invalidate a binding
mid-function is probably the wrong shape; reframe it.

Documented in the arc 006 BACKLOG's "What wat deliberately does
NOT have" section. Captured as cross-session memory
`feedback_scope_is_shutdown`. Second concrete instance of
**absence-is-signal** — paired with arc 004's `reduce` (absence =
real gap, close it) as the other direction (absence = feature
that shouldn't exist, reframe the combinator).

### Tests

4 new in `tests/wat_stream.rs`:

- `take_cuts_off_at_n_with_producer_that_would_send_more` — the
  core drop-cascade test. Producer would send 10; take 3;
  collect returns exactly 3. The bounded(1) queue + take's exit
  conspire to stop the producer via `:None` on send.
- `take_returns_all_when_n_exceeds_available` — upstream-ends-
  early case. Producer sends 2; take 5; collect returns the 2.
- `take_zero_emits_nothing` — `n == 0` edge case.
- `take_composes_with_map` — drop cascade propagates through a
  middle stage (source → map → take → collect).

All 20 stream tests pass; full suite passes.

## Slice 3 — `:wat::std::stream::from-receiver`

Trivial tuple-wrap. `(from-receiver rx handle) -> Stream<T>`. No
worker spawned; just packages a caller-provided Receiver and the
caller-provided ProgramHandle into the Stream<T> tuple alias.

**The design decision:** both arguments are required. Stream<T>'s
typealias includes the handle because downstream terminators
(`for-each`, `collect`, `fold`) join it. If the caller doesn't
have a handle, they don't have a stream — they have a bare
Receiver whose producer will never be joined. That's a broken
shutdown story, and wat won't let the typealias paper over it.

**What this re-taught.** The first test shape deadlocked: main's
`let*` bound `tx` and `pair` before calling `collect` on the
constructed stream; those bindings kept Senders alive through
`collect`, which meant the channel never closed and `recv` never
saw `:None`. The fix was to move the queue + spawn + `from-receiver`
call into a helper `define` whose return is only the Stream<T>
tuple; the helper's local bindings drop on return. Same scope-IS-
shutdown discipline that forced `take` to be a stage. The
discipline applies to tests too — you can't verify a stream
combinator while holding Sender refs the combinator is waiting
to see dropped. Test shape written up inside the test file's
comments for future readers.

## Resolved at substrate — ship on with-state

Two more items from the original prompt list closed without
shipping their own primitives:

- **`:wat::std::stream::chunks-by`** — N:1 with key-fn boundary.
  Decomposes to `with-state` with `init = (None, [])`, `step` that
  accumulates on key-match and emits on key-change, `flush` that
  emits the final partial. Library code. No primitive-level
  design question remains.
- **`:wat::std::stream::window`** — N:1 sliding. Decomposes to
  `with-state` with `init = []`, `step` that appends and trims to
  size, `flush` that decides EOS policy at the call site. Step
  size, overlap, and partial-window behavior are caller lambda
  parameters rather than stream-primitive design choices.

Both ship in the slice that lands `with-state` — arc 007 or a
slice 4 of arc 006. The primitive list grew by one (`with-state`)
and the specialization list stays the same (chunks, chunks-by,
window, dedupe, etc., all written once as wat functions on top).

## Slice 5 — chunks-by + window on with-state (shipped 2026-04-21)

The library-code thread that slice 4 unblocked. Both shipped as
pure wat stdlib composing over with-state — no new primitives.

**`:wat::std::stream::chunks-by<T,K>(stream, key-fn) -> Stream<Vec<T>>`** —
N:1 with key-fn boundary. `init = (None, [])`; step threads the
last-key-seen as `Option<K>` through the accumulator, appending
on key-match and emitting on key-change; flush emits the final
run. Clojure `partition-by` shape. K-equality via polymorphic
`:wat::core::=`.

**`:wat::std::stream::window<T>(stream, size) -> Stream<Vec<T>>`** —
sliding, step=1. `init = []`; step appends, trims to size, emits
full window on full buffer; flush emits partial IFF buffer was
never emitted (stream was shorter than size — the Ruby-example
discipline chapter 20 named: *don't silently drop data at EOS*).

Step semantics captured via `:wat::core::cond` (three-way dispatch
on `len(new-buf) > size` / `== size` / `< size`). Surface-reduction
continuation of the factoring arc 012's slice-3 landed at language
core.

7 wat-level tests in `wat-tests/std/stream.wat`:
- chunks-by: runs-on-identity, all-distinct, empty-stream
- window: full-windows, short-stream-flushes-partial,
  exactly-size-no-flush, empty-stream

## What remains — substrate-blocked

- **`time-window`** — clock primitive.
- **`from-iterator`** — iterator surfacing.
- **Level 2 `:rust::std::iter::Iterator<T>` surfacing** — own arc.

## Lesson captured

**Classify before deferring.** The arc 004 INSCRIPTION's "Not
shipped" list was a single bucket ("stdlib-as-blueprint: ship when
demanded"). That's correct as a discipline but insufficient as
tracking — it conflates three very different reasons for a path
being absent. Pattern-completions (ship), design-open
(prompt-to-resolve), and substrate-blocked (separate arc) want
different treatment. The arc 006 BACKLOG adds this classification;
future deferrals should land in the same three-bucket shape.

---

**Arc 006 slices 1-3 — complete.** chunks-by and window closed at
the substrate level (library code on with-state). Arc remains OPEN
only against substrate-blocked items: shipping with-state itself,
time-window (needs clock), from-iterator + Level 2 iterator
surfacing (own arc).

Next movement: with-state implementation — the Mealy-machine stream
stage primitive that decomposes every stateful combinator into a
caller-supplied (init, step, flush) triple. Convergence with Elixir's
Stream.transform/3, Rust's scan-with-emit, Haskell's mapAccumL,
George Mealy 1955. Shipping it proves the decomposition by rewriting
`chunks` on top and reducing the primitive surface.

---

## Slice 4 — `with-state` + chunks on top (2026-04-21)

The paused slice resumed. The Mealy-machine stream stage landed
alongside two helpers and a `chunks` rewrite that proves the
decomposition.

### What shipped

- `:wat::std::stream::with-state<T,U,Acc>` — the substrate primitive.
  Signature: `:Stream<T> × :Acc × :fn(Acc,T)->(Acc,Vec<U>) × :fn(Acc)->Vec<U> -> :Stream<U>`.
  Worker threads `Acc` through upstream items, draining each step's
  `Vec<U>` emissions downstream; at EOS, flushes the final state and
  drains; exits. Tail-recursive, standard spawn-with-bounded(1) shape
  mirroring map / filter / chunks.
- `:wat::std::stream::drain-items<U>` — tail-recursive helper that
  sends every item in a `Vec<U>` downstream, stopping early on
  `:None` (consumer dropped). Returns `:Option<()>` so the worker can
  decide whether to continue or exit.
- `:wat::std::stream::with-state-worker<T,U,Acc>` — the spawn target.
- `:wat::std::stream::chunks-step<T>` + `:wat::std::stream::chunks-flush<T>`
  — the chunks reducer triple in explicit form. `chunks-step` is
  `(buf, item, size) -> (new-buf, emits-if-any)`; `chunks-flush` is
  `(buf) -> [buf] | []`.
- `:wat::std::stream::chunks<T>` — rewritten as a `with-state` call.
  The former standalone chunks-worker retired. Same behavior (22/22
  existing stream tests pass unchanged); cleaner factoring.
- `wat-tests/std/stream.wat` — six deftests covering chunks
  (exact-multiple, partial-flush, empty-upstream), with-state directly
  (dedupe-adjacent, buffer-all-at-eos), and a names-are-values sanity
  check.

### Convergence held

Every named stateful-stage pattern reduces to a with-state triple:
chunks (buffer + size-threshold emit), chunks-by (buffer + key-fn
boundary emit), window (buffer + stride-threshold + partial EOS
policy), dedupe (last-seen + equality suppress), distinct-until-
changed (same shape as dedupe, different equality), rate-limit
(counter + clock-threshold), running-stats (aggregator that never
emits per-step). All land as library code on `with-state` when a
caller demands them.

### Dependency that landed beside it — arc 009

`with-state`'s ergonomics required passing named defines to its
`step` and `flush` parameters. Shipping without arc 009 (names-are-
values) would have forced every caller to wrap each named function
in a pass-through lambda — honest-but-wasteful ceremony the verbose-
is-honest ward would have flagged. Arc 009 closed the gap at the
substrate (eval + check both lift registered keywords to function
values); `chunks`'s rewrite on with-state passes `chunks-flush` by
bare name; wat-tests'es three with-state direct tests do the same.
Two arcs shipped as a pair.

### What this slice does NOT add

- **`chunks-by`, `window`, `dedupe`, `sessionize`, etc.** The substrate
  carries them. Each lands as library wat on top of with-state when
  a concrete caller cites use. Arc 006 still OPEN for those.
- **Terminal variants of with-state.** `fold-with-state`, for example,
  would run the Mealy machine to completion and return the final
  accumulator without a Stream output. Not shipped; caller-demanded.

### What arc 006 still holds open

- `chunks-by`, `window`, `dedupe`, `distinct-until-changed`,
  `sessionize` — library combinators on with-state.
- `time-window` — needs a clock source; own arc when a caller
  surfaces.
- `from-iterator` + Level 2 iterator surfacing — stream-from-`:rust::
  std::iter::Iterator` bridge; design question still open.
