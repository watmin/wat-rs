# BRIEF — `insert-all` the primitive, `insert` varargs over it

## The work

`:wat::rete::insert` takes exactly one fact and rebuilds a 7-field `Session` to add it — **measured at
~1.03 µs per fact above a bare `PersistentVector/conj`**, all of it that reconstruction. Seeding 40,000
facts pays it 40,000 times (~41 ms) when one rebuild would do.

Clara's primitive is the *batch* form and its single-fact call is sugar over it (`rules.cljc:11,17` —
both delegate to `(eng/insert session facts)`). We shipped only the degenerate case. Add the door:
`insert-all` as the real primitive, plus a variadic clause on `insert`.

A RED gate fails today with `UnknownFunction: :wat::rete::insert-all`.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-insert-all.md`** — the measurement, the contract
   decision, the affirmative cuts.

2. **`wat/rete.wat:833-870` — the `insert` trio today.** `insert-spec` (oracle), `insert` (delegate to
   `insert'`). THE EXEMPLAR — your `insert-all` trio mirrors it exactly.

3. **`wat/core.wat:68-78` — `:wat::core::+`.** The proof that multi-arity `defn` with a typed
   rest-param exists in this language: a 2-ary clause and a `[x y & rest <- :Vector<i64>]` clause on
   one `defn`. Your `insert` takes that shape. Copy it; do not invent syntax.

4. **`src/runtime.rs:4712` — the `insert'` dispatch arm.** Add the `insert-all'` sibling beside it,
   with its own `// rune:lint(retired-name)` marker in the same style.

5. **`src/rete/kernel.rs` — `eval_insert_native`.** Its arg-checking, span handling and **by-name
   `facts` resolution** are what `eval_insert_all_native` mirrors. Read the by-name lookup especially;
   see STOP-2.

## ★ THE ONE CONTRACT DECISION

**`insert`'s existing 2-ary clause stays UNCHANGED and must NOT route through `insert-all`.**

Clara sends even its single-fact call through the sequence form. We deliberately do not: the chaos
engine takes facts **one at a time off a wire**, so the 2-ary path is the streaming hot path, and
routing it through `insert-all` would force a one-element `PersistentVector` allocation onto the case
that matters most — buying nothing and taxing the target.

```clojure
(:wat::core::defn :wat::rete::insert
  ([session <- … fact <- …] -> :wat::rete::Session
    (:wat::rete::insert' session fact))                          ;; UNCHANGED, byte for byte
  ([session <- … fact <- … & rest <- :wat::core::Vector<…>] -> :wat::rete::Session
    …delegate to insert-all…))                                   ;; NEW
```

## The three new forms — mirror the `insert` trio

```clojure
(:wat::core::defn :wat::rete::insert-all-spec [session <- … facts <- …] -> :wat::rete::Session …)
(:wat::core::defn :wat::rete::insert-all      [session <- … facts <- …] -> :wat::rete::Session
  (:wat::rete::insert-all' session facts))
```
plus `eval_insert_all_native` in `src/rete/` and one dispatch arm in `runtime.rs`.

The native side extends the `facts` PersistentVector **once** and rebuilds the `Session` **once** —
that single rebuild is the entire point of the stone.

## Why this is semantically free (and why the gate is about shape, not semantics)

`insert` performs **zero activation** (`wat/rete.wat:828-830` — working memory stays open until
`fire-rules`). Batch insert is "extend `facts` by N" instead of "extend by 1, N times". No ordering
question, no activation, no truth-maintenance surface.

## The RED gate — `probe_arc278_insert_all_differential.{wat,rs}`

Copy the shape of `tests/rete/probe_arc278_native_insert_differential.{wat,rs}`.

| # | assertion | why it is in the gate |
|---|---|---|
| 1 | `insert-all(s,[f1..fN])` == N chained `insert` calls (same derived count AND same fact count) | the correctness claim — **load-bearing** |
| 2 | `insert-all-spec` == `insert-all'` | the dual-impl; the oracle is never skipped |
| 3 | N > 1 and resulting `facts` length == N exactly | **non-vacuity** — a no-op `insert-all` returning the session unchanged passes 1 and 2 against an empty vector |
| 4 | a single 2-ary `insert` still yields the same result as before | the hot path was not re-routed |

Put the "what would turn this red" reasoning in the `.rs` header, as
`probe_arc278_native_insert_differential.rs:18` does.

## Blast radius

`wat/rete.wat` (three new forms + one added clause), `src/rete/` (the native fn), `src/runtime.rs`
(one arm). **No call-site churn** — `insert`'s 2-ary signature is preserved, so every existing caller
compiles untouched. No corpus migration, no codemod.

## STOP triggers (each is a rejection: ship nothing, report the gap)

1. **STOP-1** — if the 2-ary `insert` ends up routing through `insert-all`, STOP. That is the contract
   decision and the whole reason this brief exists.
2. **STOP-2** — if `facts` cannot be resolved **by name** through `RecordDef.field_names`, STOP and
   report what the lookup returned. Do not fall back to a positional index; a future field reorder
   would then write the wrong slot silently.
3. **STOP-3** — if multi-arity `defn` or the typed rest-param does not work as `wat/core.wat:68-78`
   shows, STOP and report the checker's exact diagnostic. Do not reach for a macro instead.
4. **STOP-4** — if any existing rete test goes red, STOP and report the name and assertion. `insert`'s
   2-ary signature is preserved, so nothing should move.

## Definition of done

- `cargo nextest run --release -E 'test(/insert_all/)'` — all four assertions pass.
- `cargo nextest run --release -E 'binary_id(wat::rete)'` — all pass.
- `cargo nextest run --release` — the whole floor.
- `cargo clippy --all-targets --release` — silent.
- **The measurement:** seed 40,000 facts via one `insert-all` vs the existing `foldl` + `insert`, and
  report both. Expect roughly a 41 ms drop; report what you actually got.
- `git diff --stat`.

Leave the tree dirty and uncommitted. Do not commit, push, or stash.

## A prior result to copy for shape

`d9eadfe3` (native `insert`) is the direct predecessor: oracle in wat, prime in Rust, public delegate,
one dispatch arm, by-name field resolution. You are adding the second member of that family.
