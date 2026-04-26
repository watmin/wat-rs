# wat-rs arc 059 — `:wat::core::concat` — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~30 minutes
of focused work — even smaller than DESIGN's 1-hour estimate.

Builder direction (2026-04-26, mid-experiment 008):

> "what do you want concat to do .... you reached for it... we have
> ::string::concat but not a generic concat - you reaching for
> something that's not there is indicative that we should have it"

Pattern noted: when the substrate's developer reaches for an op
that doesn't exist, the gap is the design signal. Same as arc 058's
opening direction ("if it's missing it shouldn't be").

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/check.rs` — 1 dispatch arm + `infer_concat`. `src/runtime.rs` — 1 dispatch arm + `eval_concat` (pre-evaluates all args; sizes the output Vec exactly; copies elements once with `extend`). `docs/USER-GUIDE.md` — 1 surface-table row. | ~95 Rust + 1 doc | 7 new (two-arg basic, n-arg variadic, empty Vec args, single-arg clone, left-to-right order, non-Vec rejection, zero-arity rejection) | shipped |

**wat-rs unit-test count: 630 → 637. +7. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release` (workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### Mechanical mirror of `string::concat`

`infer_concat` lifts `infer_string_concat`'s shape exactly: loop
over args, unify each against the expected element type, push
TypeMismatch on failure. The only difference: `string_ty` is fixed
to `:String` for `string::concat`; for `concat`, the element type
is a fresh type variable that all args unify on. Same loop, one
substitution.

`eval_concat` pre-collects each arg's `Arc<Vec<Value>>` so the
output's `Vec::with_capacity` gets the exact total length (one
allocation, no resize). Then a single pass with `extend` copies
elements through. Per-arg type rejection happens in the first loop;
if any arg isn't a Vec, return TypeMismatch before allocating.

### Variadic with `≥1` floor

Zero-arg `(concat)` is ambiguous on T (no element type to infer)
— same shape as `(:wat::core::vec)` which also rejects zero-arg.
The check rejects with ArityMismatch (`expected: 1, got: 0` —
"1" reads as "at least 1"). Single-arg `(concat v)` returns a
clone of v — uniform with the multi-arg behavior; no special
case in the code path.

### Why a fresh substrate primitive instead of a wat-side `foldl`

A consumer could implement concat themselves:

```scheme
(:wat::core::foldl xss (:wat::core::vec :T)
  (:wat::core::lambda ((acc :Vec<T>) (xs :Vec<T>) -> :Vec<T>)
    ...))
```

But the inner step has no append-many primitive — it'd have to
foldl AGAIN over `xs` calling `conj` per element. That's `O(N×M)`
allocations for N input vecs of M elements each. The substrate
implementation is one allocation + N memcpy — `O(N+M)` total.

When the wat-side workaround is asymptotically slower than what
the substrate can give in 25 lines of Rust, the substrate provides
the primitive. Same rule arc 035 applied to `length` (substrate-
provided beat hand-rolled foldl-counting).

### Inputs unchanged (values-up)

Each piece is borrowed via `Arc::clone` from the source Vec, then
`extend` copies the Values into the new buffer. Original Arcs stay
live; their backing vectors are not modified. Standard substrate
discipline.

---

## What this unblocks

- **Lab experiment 008 (Treasury service `loop-entry`).** The
  triggering call site:
  ```scheme
  (:wat::core::concat
    (:wat::core::vec :EventRx tick-rx)
    broker-rxs)
  ```
  Direct construction of the select-loop's Vec<Receiver<Event>>
  from a single tick-rx + N broker-rxs. Replaces the workaround
  foldl that would have been needed.
- **Future programs combining Vec inputs** — service setup with
  mixed-source receivers, pipeline stages combining vec-chunked
  outputs, regime observers building chain.regime_facts from
  per-source vecs.

---

## What this arc deliberately did NOT add

Reproduced from DESIGN's "What this arc does NOT add":

- **HashSet `union` / HashMap `merge`.** Different ops; collision-
  policy questions. Future arcs.
- **Vec `splice` / `insert-at`.** Different mutation shape. Future
  arc when consumer surfaces.
- **Lazy concat (chain iterator).** Out of scope; build it when a
  consumer needs lazy ops over Vec sequences.
- **Variadic `cons` / prepend.** Out — `(concat [x] vs)` covers it.

---

## The thread

- **2026-04-26 (mid-experiment 008)** — Treasury layout reaches for
  `:wat::core::concat`; it's not there.
- **2026-04-26 (DESIGN)** — proofs lane drafts the arc; one decision
  pass (Q1–Q5) settles variadic shape, ordering, values-up,
  inference, scope.
- **2026-04-26 (this session)** — slice 1 ships in one commit:
  `infer_concat` + `eval_concat` + 7 inline tests + USER-GUIDE row
  + this INSCRIPTION.
- **Next** — Treasury layout consumes the new primitive directly.

PERSEVERARE.
