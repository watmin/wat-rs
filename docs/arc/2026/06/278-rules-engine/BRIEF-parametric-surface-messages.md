# BRIEF — a surface's `:messages` completeness check must compare BASE names (parametric messages)

> Third in the series: `7336464e` (defservice takes type params) → `10107da9` (depth-aware type-arg split)
> → this. **The last blocker for cache Stone 2's parametric protocol** (builder-ruled option (a),
> four-questions: 4×YES). Builder's read, which the grounding confirmed: *"we've had issues with generics
> being wiped from symbols before — i expect this is another string parser thing."*

## The defect — generics wiped from ONE side of a string comparison

`src/types/surface.rs`. The declaration side stores the message keyword **verbatim**, params included (`:648`):

```rust
if let Some(WatAST::Keyword(mn, _)) = mi.get(1) {
    message_names.push(mn.clone());          // → ":probe::Cache::GetRequest<K>"
}
```

The reference side walks a `TypeExpr` and, for a parametric, pushes **the head only** (`collect_user_type_paths`):

```rust
TypeExpr::Parametric { head, args } => {
    out.push(format!(":{}", head));          // → ":probe::Cache::GetRequest"   ← params GONE
    for a in args { collect_user_type_paths(a, out); }
}
```

Then both walls compare by raw string equality:

```rust
if !message_names.iter().any(|mn| mn == &r) { …error… }
```

`":…::GetRequest<K>" != ":…::GetRequest"` → a correctly-declared parametric message is reported undeclared.
A **concrete** message has no params on either side, so the spellings coincide — which is why every existing
surface passes and only a parametric one trips.

**Observed, verbatim** (orchestrator probe, the real cache-protocol shape):

```
malformed :wat::core::defsurface declaration: surface :probe::Cache feature `get` references protocol
type :probe::Cache::GetRequest which is not declared in this surface's :messages …
```

Note the reported name has **no `<K>`** while `:messages` declares `GetRequest<K>`.

## The fix

Normalize the **declared** name to its base before it enters `message_names` — the reference side is already
base-only and needs no change. Keep the full spelling anywhere the params are genuinely needed (registration,
the shipped-forms payload); this normalization is for the membership comparison.

**BOTH comparison sites need it — a fix at one is a half-fix:**
- `src/types/surface.rs:827` — the direct wall (a feature's `req <-` / `-> ret` types).
- `src/types/surface.rs:670` — *"WALL 2 — TRANSITIVE completeness"*, walking each message TypeDef's own
  referenced paths.

Prefer one shared helper over two call-site edits, so the twins cannot drift (the arc-170 lesson that shaped
the previous strike: reuse the tracker, don't mint a second).

## ⚠ SAFETY PROPERTY

A message with **no** type params already has identical spellings on both sides, so base-normalization is the
identity for it. **Every existing surface must be bit-for-bit unaffected.** Verify it, don't argue it — the
prior strike's standard was: run the HEAD-built binary and the patched one over the whole `.wat` corpus with
`--check --check-output edn`, diff, and dispose of every difference. Match that bar.

## The gate — and it must reach the WIRE, not just the declaration

Declaring the surface is necessary but proves little on its own. The builder's option-(a) ruling passed
*Honest* **conditionally**: *the decode must actually enforce `K` at the boundary — if a `Vector<K>` decodes
without checking `K`, the parameter is decoration.* So the gate must carry a parametric payload across.

Land a `deftest` sibling of `wat-tests/service-parametric-two-params.wat`:

- A surface with **parametric messages** — e.g. `GetRequest<K> [probes <- Vector<K>]` and a response carrying
  `Vector<Option<V>>` — plus the two mandatory bits (`:max-request-bytes` on the op, `:RequestTooLarge` on the
  response enum).
- A `<K,V>` service satisfying it, stood up on the thread locus, `connect'`ed via `Handle/addr`.
- **K and V pinned to DIFFERENT concrete types** (`K=String`, `V=i64`) and a real round-trip: send actual
  `String` probes, get actual `i64` results back, assert on the values. A gate that only proves the surface
  *declares* would not distinguish this fix from a broken wire.

The existing single-param and two-param gates stay green; all nine concrete defservices unchanged.

## STOP triggers — halt and report, do NOT paper over

1. **If the wire cannot carry the parametric payload** — the EDN codec, the child-lineage forms, or the decode
   failing to enforce `K` — **STOP and report**. That is the deeper ruling the four-questions flagged as
   *Honest*-conditional, and it belongs to the builder. Do not weaken the gate to concrete messages to get green.
2. If base-normalization breaks a concrete surface — STOP; the safety property is not negotiable.
3. If the blast radius exceeds `src/types/surface.rs` + the new gate — STOP and report.

## Blast radius

`src/types/surface.rs` (the normalization + both call sites) and the new gate. Nothing else.

## Gate

- The parametric-message service declares, stands up, and round-trips typed values as a `deftest`.
- `cargo build --release` clean.
- `cargo nextest run --release` — the **Summary line VERBATIM**. Current floor: **4171 passed, 314 skipped**.
- FOREGROUND only. **Do NOT commit** — the orchestrator weighs by their own re-run and commits.

## Your report

Diff shape; how you verified the concrete path is unchanged; the probe's before/after; whether the WIRE
genuinely carries and enforces `K` (with evidence, not inference); the verbatim Summary line; any STOP.
