# Arc 006 — Stream Stdlib Completions — Backlog

**Opened:** 2026-04-20.
**Motivation:** arc 004 INSCRIPTION's "Not shipped" list called out a
deferred combinator set under stdlib-as-blueprint discipline. A
second look at that list splits it into three shapes:
pattern-completions with no design questions, combinators with
real design questions, and substrate-blocked items. This arc
closes the first shape and catalogs the other two.

---

## Ship this slice — trivial pattern completions

Symmetric with the shipped set. No design questions. Shipping
them closes obvious gaps a reader would ask about.

- **`:wat::std::stream::flat-map`** — 1:N expansion. For each
  upstream item, apply `f` to get a `Vec<U>`; emit each element
  downstream. Empty vec emits nothing. Symmetric with `chunks`
  (N:1).
- **`:wat::std::stream::inspect`** — 1:1 side-effect pass-through.
  Apply `f` for its effect; forward the original value unchanged.
  Debugging ergonomics.

## Prompt on design question — ship after user resolves

- **`:wat::std::stream::first`** — take N, then stop. Early-terminates
  the upstream. Current wat has no way to force-release a let*
  binding while it's in scope, and `:wat::kernel::drop` on channels
  is a runtime no-op (comment in `runtime.rs::eval_kernel_drop`
  confirms: "Close happens when the caller's enclosing scope
  releases its own binding"). So the naive shape — drain N then
  join — deadlocks against an infinite producer. Need shutdown
  semantics: either an additive kernel primitive to force-drop, a
  drain-and-discard pattern (only works for finite producers), or
  a return shape that passes the handle back to the caller for
  them to join after their own drop cascade.
- **`:wat::std::stream::chunks-by`** — N:1 with key-fn boundary.
  Design question: emit on key-change (new key arrives) or
  key-end (upstream disconnects / last key ended)? Rust's
  `itertools::group_by` emits on key-change; Elixir's
  `Stream.chunk_by` emits on key-change too. Probable answer is
  key-change; confirm before shipping.
- **`:wat::std::stream::window`** — sliding window. Design
  questions: (a) step size (1 = every-item windows; N = stepped;
  what's the default?); (b) overlap semantics (N-step-N is
  chunks; 1-step-N is true sliding); (c) EOS behavior — emit
  partial windows at end, or drop them, or only if consumer opts
  in? `:wat::std::list::window` exists for in-process — match
  its shape? (Need to re-check its semantics.)
- **`:wat::std::stream::from-receiver`** — wrap an existing
  `Receiver<T>` as a Stream. Design question: ownership of
  ProgramHandle. Stream's typealias is `(Receiver<T>,
  ProgramHandle<()>)`; if the receiver came from outside, there's
  no producer handle to attach. Options: synthesize a no-op
  handle (dishonest), pass the handle as a second argument (what
  did spawn the receiver?), or change `Stream<T>` to make the
  handle optional (breaks symmetry with spawn-producer). None
  obviously right.

## Substrate-blocked — not this arc

- **`:wat::std::stream::time-window`** — N:1 with time-bucket
  boundary. Needs a clock primitive wat doesn't have. Defer until
  a clock substrate arc.
- **`:wat::std::stream::from-iterator`** — needs
  `:rust::std::iter::Iterator<T>` surfaced via `#[wat_dispatch]`.
  Its own arc.
- **Level 2 iterator surfacing (`:rust::std::iter::Iterator<T>`)**
  — the in-process lazy flavor. Own arc, different substrate
  work (iterator trait surface, adapter-chain methods, interop
  with channel `Receiver::into_iter`).

---

## Tracking

| Item | Status | Commit |
|---|---|---|
| flat-map | **shipped** | this slice |
| inspect | **shipped** | this slice |
| first | prompt-pending | — |
| chunks-by | prompt-pending | — |
| window | prompt-pending | — |
| from-receiver | prompt-pending | — |
| time-window | blocked | — |
| from-iterator | blocked | — |
| Level 2 iterator | blocked | — |
