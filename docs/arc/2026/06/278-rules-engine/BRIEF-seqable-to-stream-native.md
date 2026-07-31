# BRIEF — Strike 1: `seqable->stream` goes native (the one door)

Design: `DESIGN-STONE-seq-traversal-one-door.md`. Read it first — it carries the measurement, the
root cause at `file:line`, and the four-questions ruling you are implementing.

## The work, in one paragraph

Every lazy pipeline over an eager container in wat is O(n²), because the eager→lazy converter
`:wat::core::seqable->stream` (`wat/seq.wat:251-278`) steps its source by calling `rest` on it, and
`rest` on any eager container rebuilds the whole thing. Replace that wat implementation with a
native Rust one that steps its source **by position**, materialising nothing per element. The
Stream-only machinery downstream is already correct and does not change.

## Read in order (the rooms)

1. **`wat/seq.wat:243-278`** — `seqable->stream` today: the doc comment stating the intended
   architecture (`:248`), then four clause bodies that each recurse on `(rest coll)`. This is what
   you are replacing.
2. **`src/collection/eval.rs:1596-1660`** — `eval_rest`. Read the `PersistentVector`, `List`, and
   `Stream` arms together: the first two rebuild, the third is `Arc::clone(tail)`. This is *why*
   the current form is quadratic, and it also shows you the `StreamContainer::of_value` dispatch
   idiom you will reuse.
3. **`src/collection/transform.rs:440-470`** — `eval_vec_foldl`'s container match. **Copy this
   shape.** It is the working reference for iterating each eager container natively, and it is
   exactly the dispatch your new function needs.
4. **`src/collection/seq_container.rs`** — the registry (`StreamContainer::of_value`, `ordered()`,
   `mappable()`). Your dispatch goes through this; do not re-derive container classification.
5. **`src/stream/`** (the `Stream` enum: `Empty` / `Cons` / `Thunk` / `NativeThunk`) — how a Stream
   is built. `NativeThunk` is the shape that lets a Rust closure produce the next cell lazily.

## Implementation sketch

A native `eval_seqable_to_stream` registered for `:wat::core::seqable->stream`, dispatching on
`StreamContainer::of_value`:

- **`Stream`** — already a Stream; return it unchanged.
- **`Vector` / `PersistentVector`** — indexable. Produce a `Stream` whose thunk holds
  `(the container, index)` and yields `Cons(elem_at(index), thunk(index+1))`, `Empty` at the end.
  **Clone nothing per step** — the container is behind an `Arc`/persistent handle already.
- **`List`** — an `Arc<LinkedList>`, which has **no indexed access**. Snapshot it into an indexable
  form **once** (O(n)), then step that by index. One O(n) pass total, not per element. Do NOT index
  a LinkedList per step — that reintroduces the quadratic on this arm, and the design's four
  questions turn on exactly this point.

Then delete the wat `seqable->stream` clauses and register the verb natively (the registration
pattern is the same one `:wat::core::map` uses — see `runtime.rs:5168`).

The **six verbs that already delegate** through it (`keep`, `keep-indexed`, `take-nth`, `dedupe`,
`distinct`, `map-indexed`) must go linear with no edits of their own. That is the proof the door is
shared.

## Blast radius

`src/collection/` (the new function + its registration), `src/runtime.rs` (one dispatch arm), and
the removal of `seqable->stream`'s clauses from `wat/seq.wat`. **Do NOT touch** the seven
hand-rolling verbs — `filter`, `remove`, `take-while`, `drop-while`, `interpose`, `reductions` —
they are Strike 2 and are deliberately out of scope here.

## The RED gate — write it FIRST, prove it RED, then fix

Before changing any implementation, add a test asserting the **absence of the quadratic**, and run
it to confirm it fails. Paste the RED output in your report.

It is a wall, not a stopwatch: at n=4000 quadratic is ~12,000ms and linear is ~10ms, so assert
`(into [] (keep …))` — a verb that delegates through the normaliser and that you are NOT editing —
over 4000 elements completes in **under one second**. RED today by ~12×; GREEN after by ~100×. No
machine variance crosses a 100× margin.

Model it on `a8_node_share_fire_census` in `src/rete/kernel.rs` for the in-Rust wat-driving shape
(`startup_from_source` + `eval_in_frozen`).

## Your gates

Run everything in the FOREGROUND and wait. Never background a command and return.

1. `cargo build --release --all-targets` — exit 0, zero warnings.
2. Your new RED gate — **RED before the fix** (paste it), **GREEN after**.
3. `cargo test --release --lib -- seqable --nocapture` (or your gate's filter) — green.
4. Stdlib load order, since you touched `wat/`: a two-line `:user::main` printing
   `(:wat::deporder::verify-stdlib)` must print `[]`.

Do **not** run the full `cargo nextest run`. The orchestrator weighs the floor centrally.

## STOP triggers

Each means: ship nothing, report what you found, stop. None is permission to improvise.

- **STOP-1** — the `Stream` enum cannot express a thunk holding `(container, index)` without
  cloning the container per step. Report the exact shape that blocks it. Do not fall back to
  materialising the whole sequence eagerly — that changes laziness semantics and would break
  infinite/early-exit consumers.
- **STOP-2** — your RED gate does **not** fail before the fix. That means it is not measuring the
  defect and would pass vacuously afterwards. Report it; do not proceed on a gate you have not
  seen go red.
- **STOP-3** — any of the six delegating verbs changes RESULTS (not just timing), or the
  load-order gate prints anything but `[]`.
- **STOP-4** — you find yourself editing one of the seven hand-rolling verbs, or `eval_rest`, to
  make something pass. Both are out of scope by design; report instead.

## Report back

Each gate's result, the RED output and the GREEN output of your gate, the diff, and anything that
surprised you. Do not commit — the orchestrator weighs and commits.
