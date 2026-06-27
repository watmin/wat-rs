# BRIEF — 294.a: direct-EDN measurement

**The work, in one paragraph.** The holon measurement surface (`:wat::holon::cosine` / `dot` / `coincident?` /
`presence?` / `coincident-explain` / `coincident-floor?` / `simhash`) today accepts only
`:wat::holon::HolonAST | :wat::Record | :wat::holon::Vector`, so a plain EDN value must be hand-lifted via
`(:wat::holon::to-holon …)` before it can be measured. Widen the surface to accept **any `EdnRepresentable`
value**, lifting internally via `to_holon_inner` — so `(:wat::holon::cosine {:a 1 :b 2} {:a 1 :b 3})` Just Works
(and `[1 2 3]`, scalars, AND base records). **The Holder wall holds:** a `Struct` (non-portable, not EDN-repr)
still cannot measure — it fails honestly via `to_holon_inner`'s own error. This is the 294 thesis at the smallest
surface: measurement rides on the EDN data, not on a derived type the caller must name.

## THE ONE CONTRACT DECISION (pinned)
**The measurement surface accepts any `EdnRepresentable` value — plain collections, scalars, AND base records —
lifting internally via `to_holon_inner`; only genuinely non-EDN values (`Struct`, resources, closures) reject.**
ONE rule, no special cases. The current base-record rejection (*"base record has no holon flavor; construct a
holonic record"*, `runtime.rs:16314`) **is the inversion 294 annihilates** — it dies into the uniform lift.

## Read in order (the rooms — grounded this session)
1. **`src/check.rs:12854` — `infer_polymorphic_holon_pair_to_f64`** (the cosine/dot arg type-check). Both args are
   gated by `is_holon_or_vector(&resolved, env.types())` at `:12881` and `:12892`; failure → `TypeMismatch` with
   `expected: ":wat::holon::HolonAST, :wat::Record, or :wat::holon::Vector"`. **Widen the gate to also accept
   EDN-repr types**, and update the `expected` string.
2. **`src/check.rs:4909`** — the dispatch (`:wat::holon::cosine | :wat::holon::dot` → the handler above). **Find the
   sibling type-check handlers** for `coincident?` / `presence?` / `coincident-explain` / `coincident-floor?` /
   `simhash` (grep the holon verb names in `check.rs`) and widen each the same way. Prefer widening the **shared
   predicate** if they all route through one — but see STOP-2.
3. **`is_holon_or_vector`** (grep its def in `check.rs`) + **`is_portable_type`** (`check.rs` ~`:13001`, the EDN-repr
   gate that keys `TypeDef::Record => true`, `TypeDef::Struct => false`). The widened predicate is
   `is_holon_or_vector(t) || is_portable_type(t)`.
4. **`src/runtime.rs:16296` — `pair_values_to_vectors`** (the value→vector converter). The `let a = match a {…}` /
   `let b = match b {…}` normalization blocks (`:16308`–`:16335`) reject base records (`:16310`/`:16324`); the final
   `match (a, b)` has a catch-all reject at `:16367`. **Replace both rejects with a `to_holon_inner` lift.**
5. **`src/runtime.rs:14396` — `to_holon_inner(value, span) -> Result<Value, EvalBreak>`** (Ok = `Value::holon__HolonAST`).
   The polymorphic lift of any value → HolonAST; errors honestly on non-EDN (`Struct`, resources).

## Implementation sketch (fill it; do not invent the shape)
- **check side** (both arg gates, in every holon-pair handler): `if !is_holon_or_vector(&resolved, env.types()) &&
  !is_portable_type(&resolved, env.types()) { …TypeMismatch… }`. Update `expected` → e.g.
  `":wat::holon::HolonAST | :wat::holon::Vector | any EDN-representable value"`.
- **runtime side** (the `let a = match a {…}` block, and identically for `b`): keep the holon-record → HolonAST arm;
  keep `Value::Vector` and `Value::holon__HolonAST` as passthrough; **replace the base-record reject AND the
  `other => other` passthrough** with a single lifting catch-all:
  `other => to_holon_inner(other, list_span)?` (yields `Value::holon__HolonAST`, which the existing `match (a, b)`
  arms already encode). The final-match catch-all reject at `:16367` then becomes unreachable for EDN values and
  stays only as the belt for anything `to_holon_inner` somehow returns non-AST (keep it).

## Blast radius (bounded)
`src/check.rs` (the holon-pair arg handlers + their `expected` strings) and `src/runtime.rs`
(`pair_values_to_vectors`) **only**. No new types, no new verbs, no signature/arity changes. The RED gate
`tests/types/probe_arc294a_edn_measures_directly.rs` flips GREEN — **un-ignore both tests**. A holon test that
asserts the old *"construct a holonic record"* error must be updated (that error is the inversion being removed) —
surface it, don't silently delete.

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1:** if `to_holon_inner` does **not** already lift a base record (`Value::wat__Record`) cleanly — i.e.
  lifting one errors — STOP and report. The contract requires base records to measure; if the lifter can't, that's
  a real gap to surface, not to hack around.
- **STOP-2:** if the holon measurement handlers do **not** share one predicate, or widening a shared predicate
  trips unrelated non-measurement holon ops (e.g. `Bind`) — STOP; widen per-measurement-handler instead, and report
  which ops share the predicate.
- **STOP-3:** if `is_portable_type` is not the right EDN-repr type-predicate (wrong signature, wrong home) — STOP
  and report what the correct EDN-repr type-check is.

## EXPECTATIONS (scorecard — fixed before the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | plain EDN map measures | `nextest … -E 'test(/edn_map_measures/)' --run-ignored all` (un-ignored) | GREEN |
| 2 | plain EDN vec measures | `… test(/edn_vec_measures/)` | GREEN |
| 3 | base record measures | run `(:wat::holon::cosine (:T 1 2) (:T 1 3))` via the wat binary | a cosine in [-1,1]; NO "construct a holonic record" |
| 4 | **Struct still rejects (Holder wall)** | `(:wat::holon::cosine (struct-val) (struct-val))` | rejects honestly (Struct not EDN-repr) |
| 5 | nothing else breaks | `cargo nextest run --release -p wat` | floor 0; SET-diff ∅ vs HEAD |

**Runtime prediction:** 15–25 min. **Trap-door:** an existing holon test may assert the old base-record rejection
text → update it to the new measure-directly behavior (the rejection was the inversion). Content-integrity: read
the diff end-to-end; nothing outside `check.rs`/`runtime.rs` should move.

**You are a LEAF. Do NOT spawn subagents. If the work exceeds this brief, STOP and report.**
