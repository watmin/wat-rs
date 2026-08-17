# BUILD ORDER — 118.5 · `Seqable`. Everything below is measured; nothing is left to derive.

**Builder, 2026-08-17: *"we build tomorrow."*** The contract is ruled
(`DESIGN-118.4-…`), the mechanism landed (`118.3-B`, `a15f4ea9`), the ground is measured. This file
is the order, so the next self starts building instead of re-deciding.

## The invariant everything follows from

> **`Seqable` is what you can walk. An operation is INTERMEDIATE (returns a Stream, stays lazy) or
> TERMINAL (consumes, returns a value). On a read-once value, consumed means gone.**

`into` is the materializer (**both Stream clauses already exist** — verified). A Stream refuses cheap
structural queries; you write `into` to say *"I am consuming this"*; the materialized value answers
everything. **The refusal is a forcing function, not a hole.**

## ⛔ Sequence is load-bearing: TABLES first, SURFACE second

Minting `Seqable` before the two sets converge **bakes the split in** — you would be naming a concept
whose members disagree about what they support.

---

### Stone 1 — converge the capability sets

`src/collection/seq_container.rs`. Flip `Stream => true` in **four** tables, and **only** four:

| table | verbs it gates | why |
|---|---|---|
| `mappable()` | `foldl` · `foldr` | terminal; `foldl` is naturally single-pass |
| `ordered()` | `reverse` · `concat` | `concat` intermediate; `reverse` terminal, materializes |
| `searchable()` | `contains?` | terminal, short-circuits |
| `has_append()` | `conj` | intermediate |

⛔ **DO NOT TOUCH `measurable()` OR `gettable()`.** `Stream => false` there is **the contract**, not a
gap — `length`/`empty?`/`get` must keep refusing. Their `○ gap` comments are **wrong** and should be
corrected to say so in the same stone. *(Ruby refuses `Enumerator#length` and `#empty?` for exactly
this reason — demonstrated live by the builder.)*

★ `indexable()` and `has_tail()` are **already `true`** for Stream — `first`/`second`/`third`/`rest`
work today. **Clojure's ISeq primitives are already under this.**

⚠ `map`/`filter`/`take`/`drop` **bypass these tables entirely** via `extract_lazyable_elem`
(`infer.rs:628`). So `mappable()`'s doc comment — *"`map`/`filter`/`foldl`/`foldr`"* — is **already
partly stale**; fix it while you are there.

### Stone 2 — mint `count`

The terminal walker. Any `Seqable` including Stream → `i64`.

★ **`:wat::core::count` is FREE and ALREADY BLESSED PURE-TOTAL** — it occurs exactly once in the
tree, in `macros/eval.rs:563`'s `is_pure_total` allowlist, with **no `TypeScheme` and no dispatch
arm**. Someone intended it and never built it. **No allowlist change needed.**

`counted?` (Clojure's name for Ruby's `size → nil`: is the length known without walking?) is optional
and also free — rule it separately, do not bundle.

### Stone 3 — mint the surface

`wat/seq.wat` — stdlib position **67**, immediately after `core.wat` (40) and before `Record.wat`
(131) / `bracket.wat` (169) / `string.wat` (278). Declared there, `Seqable` is visible to nearly the
whole stdlib.

```wat
(:wat::core::defsurface :wat::core::Seqable<T> :nature :wat::core::Struct
  :features [(seq [self <- :wat::core::Seqable<T>] -> :wat::stream::Stream<T>)])
```

`:wat::core::seq` is free (0 hits). Extend all four containers. **Exemplars that already work:**
`wat-scripts/scratch-pad/probe-seqable-parametric-all-four.wat` (declarations, `--check` 0) and
`probe-seqable-is-spellable-today.wat` (runs, `"3,4"`).

⚠ **`extend-type` methods register GLOBALLY as `<ConcreteType>/<method>`, not per-surface.** Two
surfaces implementing `Vector` with the same method name collide with `DuplicateDefine` — a rider hit
this. `Vector/seq` · `List/seq` · `PersistentVector/seq` · `Stream/seq` are all currently **free**.

### Stone 4 — the payoff: collapse the seven `-stream` twins

`wat/seq.wat`: `dedupe-stream` · `distinct-stream` · `interpose-stream` · `keep-stream` ·
`keep-indexed-stream` · `map-indexed-stream` · `reduce-stream`. Each exists **only because this type
did not.** Seven verbs become one polymorphic one — which is also strictly easier for the bytecode
compiler to consume.

**Its own stone. Do not bundle it into 1–3.**

---

## Known limits to carry, not rediscover

- **The `Var`-gate excludes CONCRETE surface instantiations.** `[s <- :Seqable<T>]` works;
  `[s <- :Seqable<i64>]` does not. **Deliberate** — it is what keeps 118.3-B away from
  `Dialable`/`TypedCapability`/`Handle`, whose args are fully concrete. Costs `Seqable` nothing
  (`join`/`map`/`filter` are all generic). `NOTE-the-Var-gate-excludes-concrete-surface-instantiations.md`
- **A surface with >1 type param is untested** — the positional-order assumption (declared param *i*
  ↔ actual arg *i*) has no coverage; every container and tenant here is arity-1.
- **`into` lacks `(Vector<T>, List)`** — sibling of task #45's shipped `(PersistentVector, Vector)`.
  Independent, small, real.
- **`join` shipped over `Vector<T>`, not `Seqable<T>`** — narrower than the chain doc specified.
  Pointing it at `Seqable` needs a question nobody has probed: **can a Rust intrinsic's `TypeScheme`
  name a wat-defined surface?** Measured: **zero** existing schemes name any wat type, so there is no
  precedent. `join` must stay an intrinsic (bootstrap cycle).
- **Dispatch cost ~795 ns, upper bound, once per collection** —
  `bench-surface-dispatch-cost.wat`. ⛔ **Do not cite it against surfaces**: it measures the condemned
  interpreter (its own *direct* arm costs ~1.05 µs for a `length` call). The surface is the
  compiler's input.

## The gate shape for every stone here

Row 0 non-vacuity **first**, before touching `src/` — capture the RED verbatim. Then floor via
`scripts/floor.sh` (Summary line, never a piped exit code), clippy 0, `#[ignore]` 13. Baseline at
`f5c570f2`: **4703 / 0 / 0**.

⚠ Goldens under `tests/diagnostics/` that fail on a **shifted `src/*.rs` line number**: updating them
**is** the work. Anything else red follows the no-known-flakes protocol.
