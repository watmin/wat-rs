# Arc 006 — Stream Stdlib Completions (Slice 1) — INSCRIPTION

**Status:** first slice shipped 2026-04-20. Arc remains OPEN for
the design-question + substrate-blocked items.
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

## What remains — pending prompts

The following await user-resolution on their design questions, per
the arc 006 BACKLOG:

- **`first`** — early-termination shutdown semantics. Current kernel
  `drop` is no-op on channels; let* can't force-release a binding.
  Need either an additive kernel primitive or a return-shape change.
- **`chunks-by`** — key-change vs key-end boundary.
- **`window`** — step / overlap / EOS partial-window behavior.
- **`from-receiver`** — ProgramHandle ownership when wrapping an
  external Receiver.

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

**Arc 006 first slice — complete.** Arc remains OPEN against its
pending prompts and substrate blocks. Next movement: user resolves
any of first / chunks-by / window / from-receiver; slice 2 ships.
