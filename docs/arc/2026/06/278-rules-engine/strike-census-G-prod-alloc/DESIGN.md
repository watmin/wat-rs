# DESIGN — census G: two counters that measure nothing and are read by nobody

Census audit section G (`../vigilia-2026-09-05/recon/census-name-audit.md:97-105`), re-grounded at
HEAD 2026-09-06 — and it is wider than the audit rowed it.

## The finding

`src/rete/eval_insert.rs`, both on the same straight-line path in one function:

```rust
:188   census_count_n("prod:vec-alloc",    2);   // value_asts + fields
:207   census_count_n("prod:record-alloc", 2);   // AggregateValue + the fields Arc
```

**A hardcoded `2`.** Not a measurement — a constant multiple of the call count, wearing an
allocation name. And the constant is provably wrong in three directions:

- the kwargs branch of `rete_kwargs_value_asts` allocates **three** vecs (`:55`'s `placed`, the
  result vec, and `fields`), the positional branch two;
- `Vec::with_capacity(value_asts.len())` at `:189` allocates **none** when `value_asts` is empty;
- and between the two bumps the function can return `Err` on a malformed RHS, so even against each
  other they are attempts vs successes.

## ★ Nobody reads either one — and two siblings share that fate

Grepped at HEAD across `src/` and `tests/`:

| counter | emit sites | read sites |
|---|---|---|
| `prod:derivations` | 1 | **4** — real, and gated |
| `prod:vec-alloc` | 1 | **0** |
| `prod:record-alloc` | 1 | **0** |
| `prod:class-alloc` | 1 | **0** |

Neither appears in any exact-equality NAMES list. They are emitted into a census that nothing
inspects.

**And that is a structural gap, not an accident.** `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs`
enforces *read ⇒ emitted*. **The reverse — emitted ⇒ ever read — is enforced by nothing**, which is
exactly how three counters reached HEAD unobserved. Naming that is part of this strike's record; a
gate for it is not (see out-of-scope).

## THE ONE CONTRACT DECISION

**Delete both. Do not rename them, and do not make them count real allocations.**

- **Renaming is pointless**: a constant times the call count carries no information a call count
  would not, and there is no reader to serve.
- **Counting real allocations is rejected**, on census F's precedent and for the same reason:
  nothing wants the number, and manufacturing one invites a future consumer to trust a figure no
  test constrains.
- **Deleting removes nothing observable.** This exact function is already bracketed by three phase
  marks — `prod:shape` (`:190`), `prod:resolve` (`:203`), `prod:construct` (`:209`) — and its output
  is counted by `prod:derivations`, which four tests read. Path liveness and volume both survive.

## Why delete here and keep the never-firing guard in census E

The line is **falsehood, not unreadness.** Census E's `None` branch was kept because it is a
*guard* that correctly handles a malformed `cond` — true, useful, and merely never exercised.
These are *instruments* that report a number nobody measured. An unread truth is fine; an unread
falsehood is a trap with nobody standing near it.

By that line **`prod:class-alloc` stays**: it counts one real `String` allocation per call and its
name is accurate. Unread, but true. Named here so it is not swept up.

## Out of scope = REJECTED

- A lint for *emitted ⇒ read*. It is the right idea and it is a separate strike: it would need a
  survey of every emitted name first, and it would land findings well outside census G.
- Touching `prod:derivations` or `prod:class-alloc`.
- Census H–M. One section per strike.

## Mutation proof — none is available, and that is the point

Deleting a counter that nothing reads cannot red anything **by construction**. Saying so is the
honest report, as in census D and E; manufacturing a red would mean inventing a reader.

The evidence is instead:
1. the grep above, re-run after the change (zero readers, zero emit sites);
2. **the floor staying green**, which is itself the demonstration that nothing was watching;
3. any exact-equality NAMES list that changes — none is expected, and a change would mean the
   counters did fire in a gated world and the DESIGN's reader count is wrong.

## STOP triggers

1. Any NAMES list or test needs editing to accommodate the deletion → STOP and report. It would
   mean something *was* observing them and the "zero readers" measurement is wrong.
2. Any measured value changes → STOP.
3. You are tempted to replace them with a real allocation count or a renamed call count → STOP.
   Both rejected above.
