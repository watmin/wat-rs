# BRIEF — the dedupe map stops cloning itself

Change `Seen`'s `claimed` from `HashMap` to `PersistentMap` and move its call sites from
`:wat::hashmap::` to `:wat::map::`. `wat-scripts/fanout/circuit.wat` only.

A **fix**, not an instrument. Its gate is the drain curve.

## Read in order

1. **`DESIGN-the-dedupe-map-stops-cloning-itself.md`** — why: `hashmap::assoc` clones the whole map
   (`src/collection/eval.rs:367`), `claimed` grows to n×m, and `Seen` is one service for 12 workers.
2. **`circuit.wat:95`** — `:ephemeral [claimed <- (HashMap :- [String bool])]`. **The type.**
3. **`circuit.wat:98`** — the `:init` constructor for the empty map.
4. **`circuit.wat:163` and `:166`** — the mark fold's accumulator `Tuple`, which names the map type
   **twice** (parameter and `->` return).
5. **`circuit.wat:178`** — `(:wat::hashmap::assoc claimed key true)`. **The clone.**
6. **The `check` arm** — `(:wat::hashmap::get claimed key)`, and the mark fold's own `get`.
7. **`src/intrinsic/map.rs:60-183`** — `PersistentMap`'s API: `length`, `empty?`, `contains-key?`,
   `get`, `assoc`, `dissoc`, `keys`, `values`. Everything `Seen` uses is there.

## The work

**1. The type.** `claimed <- (:wat::core::PersistentMap :- [:wat::core::String :wat::core::bool])`
at `:95`, at `:98`'s constructor, and **both** spellings in the fold's `Tuple` at `:163`/`:166`.

**2. The verbs.** `:wat::hashmap::assoc` → `:wat::map::assoc`; `:wat::hashmap::get` →
`:wat::map::get`. Every site that touches `claimed`.

**3. Nothing else.** No other map in the file changes — see the rejected list in the DESIGN.

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**, and inside it **only the `:fanout::seen` service**. No
`sqs.wat`, no `wat/`, no `StatsResponse` — **no ripple.**

⚠ `circuit.wat:2075` (`distinct`), `:2082` (workers), `:2592`, `:2680`, `:2784`, `:2841` are other
`hashmap::assoc` sites in the same file. **Leave every one of them alone** — `:2075` has the same
defect and is deliberately a separate stone, so this one's gate stays readable.

## STOP triggers

- **STOP-1** — if `PersistentMap` cannot serve any operation `Seen` performs, **STOP and quote the
  checker.** `src/intrinsic/map.rs` says it can.
- **STOP-2** — if the no-args run differs in any field, **STOP.** `seen-recorded` and `seen-skipped`
  must be **identical**; this changes representation, not semantics.
- **STOP-3** — if `distinct` or `dup` changes at any depth, **STOP.** Dedupe correctness is the one
  thing this must not touch.
- **STOP-4** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-5** — **do not touch `:2075` or any other `hashmap::assoc` in this file**, and **do not
  touch `wat/`** — `wat/query/mem.wat` has the same defect and is the builder's call.
- **STOP-6** — **leave the tree parsing.**
- **STOP-7** — if the drain curve does **not** improve, **STOP and report it plainly.** The model is
  then dead and that is the finding; do not go looking for a second change to rescue it.

## What is deliberately NOT gated

⚠ `store-calls`, `store-ms`, `drain` all vary run to run — n=1000, same code, zero retries:
**4959 / 4982 / 4996 / 5002**. Five rows in this arc have policed noise. Report, do not band.
