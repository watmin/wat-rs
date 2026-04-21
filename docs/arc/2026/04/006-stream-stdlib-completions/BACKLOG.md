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

## Resolved — ship in this arc

- **`:wat::std::stream::take`** (replacing the originally-named
  `first`, slice 2 of arc 006). See the "What wat deliberately does
  NOT have" section below for the substrate reasoning that forced
  the rename.

## Prompt on design question — ship after user resolves
- **`:wat::std::stream::chunks-by`** — RESOLVED 2026-04-20 but
  PAUSED. Decision: ship `:wat::std::stream::with-state` as the
  substrate primitive (Mealy machine as a stream stage); let
  `chunks`, `chunks-by`, `chunks-while`, `window`, `dedupe`,
  `sessionize`, etc. all land as library compositions in wat on
  top of it. Signature:
  ```
  (with-state stream init step flush) -> Stream<U>
    step:  (Acc, T) -> (Acc, Vec<U>)
    flush: (Acc)    -> Vec<U>
  ```
  Decomposition validated by convergence with prior art —
  Elixir's `Stream.transform/3`, Rust's scan-with-emit,
  Haskell's `mapAccumL`. Finding the same shape the greats found
  is the signal the abstraction is real. Next slice: ship
  with-state, rewrite `chunks` on top (surface reduction proof),
  ship `chunks-by` as library code, then reassess window / dedupe.
  **Paused for a detour the builder is about to show — see
  `../007-detour-tbd/NOTES.md`.**
- **`:wat::std::stream::window`** — RESOLVED 2026-04-20.
  Ships as library code on `with-state`; step/overlap/EOS-behavior
  become caller lambda parameters rather than stream-primitive
  design questions. No primitive-level design remains.
- **`:wat::std::stream::from-receiver`** — SHIPPED 2026-04-20
  (arc 006 slice 3). `(from-receiver rx handle) -> Stream<T>` —
  trivial tuple-wrap, no worker spawned. Caller provides both
  arguments; if they don't have a handle they don't have a stream,
  they have a bare Receiver whose producer will never be joined.
  Don't hide that.
- ~~Original prompt text:~~ wrap an existing
  `Receiver<T>` as a Stream. Design question: ownership of
  ProgramHandle. Stream's typealias is `(Receiver<T>,
  ProgramHandle<()>)`; if the receiver came from outside, there's
  no producer handle to attach. Options: synthesize a no-op
  handle (dishonest), pass the handle as a second argument (what
  did spawn the receiver?), or change `Stream<T>` to make the
  handle optional (breaks symmetry with spawn-producer). None
  obviously right.

## What wat deliberately does NOT have — and why take is a stage

Recording the substrate gap we hit while designing `take`, so the
next contributor doesn't re-derive it.

**The gap:** wat has no way to force-release a let* binding while
it's in scope. `:wat::kernel::drop` on a channel endpoint is a
runtime no-op (see comment in `src/runtime.rs::eval_kernel_drop`:
"Close happens when the caller's enclosing scope releases its own
binding"). There is no `std::mem::drop` equivalent that invalidates
a binding mid-function.

**Why that matters:** a TERMINAL combinator with early termination
(`first(stream, n) -> Vec<T>` read as "take n, then stop and return
the Vec") deadlocks against an infinite producer. The caller still
holds `stream`; stream's tuple holds the `Receiver` Arc; the Arc
refcount never reaches zero; the producer never sees `:None` on
send; we can't join its handle; the caller's function never returns.

**Why that's intentional, not missing:** the scope discipline IS
the shutdown discipline. If you could force-release one binding
while others live, you'd invent a new class of bugs — "was this
resource still alive when I expected it?" — that Rust's ownership
model explicitly rules out. wat borrows that discipline. The
absence of a force-drop is what absence-is-signal flags: the
language is telling us the terminal shape is wrong for this
problem.

**The solve:** make `take` a STAGE, not a terminal. It returns
`Stream<T>`, spawns a worker that counts down from `n`, forwards
until exhausted. When the worker exits, its `Sender` and
`Receiver` drop naturally (spawn-closure scope exit). The drop
cascade fires the same way it does for every other stage (map,
filter, chunks). No kernel change required.

The pattern we'd have needed for `first(stream, n) -> Vec<T>` — an
early-terminating terminal — is the one wat says "no" to. The
pattern we landed on — `take(stream, n) -> Stream<T>`, then
`collect`/`for-each` as the terminal — is the one wat says "yes"
to. Different primitives, same user intent.

This is the second concrete instance of **absence-is-signal**:

1. **arc 004 `reduce`**: absence = real gap. Close it (one
   canonical normalization pass).
2. **arc 006 `first` terminal**: absence = feature that shouldn't
   exist. Don't close it; reframe the combinator.

Both instances captured in `CONVENTIONS.md`'s "When to add a
primitive" section.

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
| flat-map | **shipped** | slice 1 |
| inspect | **shipped** | slice 1 |
| take (ex-first) | **shipped** | slice 2 |
| from-receiver | **shipped** | slice 3 |
| chunks-by | **resolved** via with-state | blocked on with-state |
| window | **resolved** via with-state | blocked on with-state |
| chunks-by | prompt-pending | — |
| window | prompt-pending | — |
| from-receiver | prompt-pending | — |
| time-window | blocked | — |
| from-iterator | blocked | — |
| Level 2 iterator | blocked | — |
