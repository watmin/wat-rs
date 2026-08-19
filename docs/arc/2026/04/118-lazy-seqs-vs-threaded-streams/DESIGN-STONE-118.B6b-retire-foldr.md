# DESIGN STONE — 118.B6b · RETIRE `foldr`. It is `reverse` + `foldl` wearing a borrowed name.

**Builder's ruling, 2026-08-18:** *"delete foldr"* — after asking the question that settled it:
*"is our foldr wrong?"*

**No. It is correct, and that is not the defect.**

## What it actually is, on disk

`src/collection/transform.rs:646` — the entire implementation:

```rust
for x in xs.iter().rev() {
    acc = apply_function(func.clone(), vec![x.clone(), acc], sym, call_span.clone())?;
}
```

Reverse-iterate, accumulate. Verified correct by run: `(foldr - 0 [1 2 3])` → **2** = `1-(2-(3-0))`,
where a left fold would give −6. It computes exactly what Haskell's `foldr` computes on finite input.

**It is `reverse` then `foldl`. That is the whole verb.**

## ⛔ WHY IT GOES — the name promises a property this language cannot deliver

**Haskell's `foldr` is the LAZY one**, and that is its entire reason to exist as a distinct verb:

```haskell
foldr f z (x:xs) = f x (foldr f z xs)
```

The recursive call is an **argument** to `f`, so it is forced only if `f` forces it.
`foldr (||) False (repeat True)` returns immediately. It emits before it consumes. There, `foldl` is
the one that cannot handle infinite input — the polarity is the inverse of ours.

**wat is strict.** `apply_function` evaluates both arguments before the call. So the property that
makes `foldr` a distinct verb in Haskell cannot exist here, and what remains is `reverse` + `foldl`.

**The two other strict languages that faced this both declined to name it.** Clojure has no `foldr` —
a right fold is `(reduce f init (reverse coll))`. Ruby has no right fold — `reverse.inject`.

⚠ **The Haskell / Clojure / Ruby statements are from knowledge, not fetched.** The wat-side facts are
all measured. If this stone's argument is ever challenged, ground the three languages first — a
`clojure.org` fetch corrected me on `rest`/`next` earlier the same day.

## ★ THE DIRECTION IS WHY IT COULD NEVER TAKE A STREAM

Measured — `foldl` over a Stream, one cell at a time:

```
FORCE 0 · APPLY 0 · FORCE 1 · APPLY 1 · FORCE 2 · APPLY 2 · FORCE 3 · APPLY 3 · FORCE 4 → 6
```

`foldl` never holds more than one cell: O(1) space over a lazy source. `foldr` cannot do this by
construction — `f(a, «f(b,f(c,z))»)` needs its second argument evaluated first, which needs the LAST
element, so under strict evaluation nothing can be applied until the whole sequence is walked and
held.

**This is not an implementation gap.** The direction that makes `foldr` distinct is the same direction
that cannot stream. Retiring it dissolves the Stream question rather than answering it.

## The replacement — already on disk, no new verb

```wat
(:wat::core::reduce f init (:wat::core::reverse coll))
```

`reduce` is already `foldl`: `wat/seq.wat:308`, a two-arm defclause delegating straight to the native
(118.B7). ★ **So the alias the builder remembered already exists** — nothing is renamed *to* `reduce`;
`foldr` simply stops being a word, and the operation is spelled from parts that were always there.
And the drain becomes **visible**: you wrote `reverse`, so of course it materializes.

This moves the surface *toward* Clojure, which is the crusade's direction.

## The four questions

- **Obvious? YES.** The call site says what happens. `foldr` said something Haskell means and wat cannot.
- **Simple? YES.** One fewer verb; the composition *is* the definition, from two verbs that already exist.
- **Honest? YES.** It stops a name advertising laziness the substrate has no way to provide.
- **Good UX? YES.** One spelling, no per-verb rule about which fold tolerates a lazy input.

## ⚠ THE TRAPS

**This is a DELETION, and deletion needs more grounding than addition.** The count (5 call sites: 4
tests, 1 string literal in a codemod's rename table) is **not** the argument — zero consumers is not
evidence of deadness, and `insert-all` would have measured zero the day it landed. **The argument is
the ruling on what the thing IS.** If that ruling is ever revisited, revisit it on the semantics, not
the count. `[[feedback_no_consumers_does_not_mean_dead]]`,
`[[feedback_an_instruction_to_delete_needs_more_grounding_than_one_to_add]]`

**`foldr` is a DECLARED RETE VERB** — `src/rete/vocabulary.rs:919`, a real `ReteOp`,
`class: OpClass::Redispatch`, ruled pure/deterministic/total. Deleting core's `foldr` **requires**
deleting that row. Expect a vocabulary completeness gate to fire the way the purity gate did in
B4-0; check for one before assuming the row is free to remove.

**Three separate ledgers name it**, and they do not know about each other — the same three-gate
problem `255/NOTE-promotion-is-not-relocation-…` recorded today: `is_pure_total`
(`src/macros/eval.rs`), `intrinsic_meta` (`src/rete/purity.rs`, 3 hits), and the rete vocabulary
(4 hits). Removing from one does nothing for the others.

## Blast radius — measured

```
src/collection/transform.rs   12    src/runtime.rs               7
src/check.rs                   6    src/collection/infer.rs      6
src/rete/vocabulary.rs         4    src/rete/purity.rs           3
src/collection/seq_container.rs 3   src/macros/eval.rs           1
src/collection/mod.rs          1
```

43 sites, 9 files. Plus the corpus: 4 test call sites and 1 rename-table string literal
(`wat-scripts/fixes/rete-where-per-type-spelling.wat:83` — a *pair* in a migration table, not a call).

⚠ **`seq_container.rs`'s 3 hits are `mappable()`'s doc comments**, which describe `mappable()` as
gating `foldl`/`foldr`/`reverse`/`concat`. Those sentences go stale the moment `foldr` does.

★ **AND ITS NEIGHBOUR IS ALREADY STALE — measured 2026-08-18, fix it in the same motion.**
`ordered()`'s header (`seq_container.rs:277`) says it gates *"`reverse`/`take`/`drop`/`concat`"*.
It has exactly **two** live consumers: `concat` (`collection/eval.rs:763`) and `reverse`
(`collection/transform.rs:51`). **`take` and `drop` do not consult it** — 118.2a moved them to
`extract_lazyable_elem`'s fixed set, and `collection/infer.rs:1070` records the move in as many
words: *"classification no longer routes through `ordered()`"*.

This is NOT a capability being routed around (the `mappable()`/`foldl` shape). It is a comment that
outlived its subject, in the same file and the same class as the `mappable()` sentences above — so
it is IN SCOPE here rather than tracked elsewhere. Two capability headers, one motion, both made to
say what they gate.

## ACCEPTANCE

| | assertion | instrument |
|---|---|---|
| 1 | `(foldr …)` is refused, and the message names the replacement | a `.bad` fixture |
| 2 | ★ `(reduce f init (reverse coll))` gives the same answer `foldr` did | a test asserting **2** for `1-(2-(3-0))` |
| 3 | the 4 test call sites are **rewritten**, not deleted — they measured a right fold and still should | read the diff |
| 4 | all three ledgers are clean | build + the rete gates |

Plus: floor ≥4772/0, clippy 0, ignores 13.

## Out of scope — affirmative cuts

- **Retiring `foldl` in favour of `reduce` alone.** Clojure has only `reduce`, and `foldl` is currently
  the native kernel `reduce` delegates to — that is the oracle/native shape, legitimate as it stands.
  A separate question, not this stone's.
- **`reverse` over a Stream.** `ordered()` says `false` for Stream; whether it should is a separate
  ruling, not this stone's.
