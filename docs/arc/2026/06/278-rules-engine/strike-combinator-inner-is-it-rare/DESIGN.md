# DESIGN — is the combinator-inner path actually rare?

A **measurement strike**. It ships no hoist. Its deliverable is two numbers and a verdict on a
recorded exemption.

## The board row, and what grounding it found

`RETE-BOARD.md` carries *"`ensure_gather` re-deriving its own cache key per token"* as an open
`temperare` row, and the breadcrumb lists *"the combinator-inner `ensure_gather`"* as still open.

Grounded at HEAD, the mechanism §5 built is intact and the split is exact — `fire/mod.rs:2137-2140`:

```rust
let gathered = match join_keys {
    Some(keys) => ensure_indexed(cache, wm, alpha_id, Arc::clone(keys)),  // hoisted
    None       => ensure_gather(cache, wm, alpha_id, seed),               // re-derives
};
```

Two callers of `any_seeded_keyed`:

- **`:483`** — the Leaf arm of the hoisted path, passes the hoisted `join_keys`. §5's cure. ✓
- **`:559`** — the inner Leaf under `CondDriver::Exists`, passes **`None`**. The remaining site.

## ★ But that `None` is a RECORDED DECISION, not an oversight

Directly above the hoisted caller, `fire/mod.rs:494-496`:

> ```rust
> // rune:temperare(simplicity-win) — combinator :not/:exists still PMap::from_pairs;
> // leaf already uses BindView. n tokens with combinator inners is the rare path.
> ```

Someone met this path, judged it rare, and wrote the judgement down. **The board row is therefore
not "an un-hoisted site" — it is an exemption whose premise nobody has re-weighed.**

An exemption is a claim about present truth (`excusare`). This one says *"n tokens with combinator
inners is the rare path"*, and the honest next act is to measure that sentence, not to hoist past it.

## Why measuring first, specifically here

`temperare` taught this session its sharpest method lesson, and it is recorded in the breadcrumb:
**two of five readings of a hot path were wrong in OPPOSITE directions** — §1 overstated by 3×, §4
was dismissed as vacuous when it was 1.00 per element. Both were settled by a count, not an argument.

Hoisting on the strength of "there is a `None` there" would be a third reading.

## THE ONE CONTRACT DECISION

**Instrument both callers with a file-append tripwire and run the full floor.** Report
`:483` (hoisted) and `:559` (re-deriving) call counts.

The file-append is the instrument, not a census counter: `census_count` is only visible inside a
`with_count_census` window, and nextest runs a process per test, so a counter cannot aggregate across
the suite. This technique was **validated in this session** — placed unconditionally it produced
10,000 lines, so an empty file is a real absence rather than a broken probe (see
`../strike-census-E-reachability/`).

⚠ **Validate it again here**, cheaply: the two counters must sum to something non-zero, and
`:483`'s count is the built-in positive control. A zero at BOTH sites means the probe failed.

## The verdict this buys

- **`:559` rare relative to `:483`** (say, well under 10%) → the rune's premise **holds**. Close the
  board row by citing the measurement at the rune, dated. **Ship no hoist.** An exemption that has
  been re-weighed and survived is worth more than a hoist nobody needed.
- **`:559` comparable to or above `:483`** → the premise has **rotted**. The hoist is then warranted,
  the stability precondition is already gated (below), and it becomes its own strike.

## The precondition is already in place either way

`rank_and_instrument.rs:1096-1104` walks four axes — `accum`, `exists-of-leaf`, `not-of-leaf` and
**`exists-of-and`** — and asserts `distinct == 1` per node, with a non-vacuity guard
(*"ensure_gather recorded ZERO rows — the instrument did not fire"*). **The combinator path's
key-set stability is already gated.** §5 scoped its hoist to Leaf but built the gate wider. So if
the measurement says hoist, the safety argument is not new work.

## Out of scope = REJECTED

- **Hoisting in this strike.** The point is to decide whether it is wanted.
- Touching the `PMap::from_pairs` half of the rune. That is a separate cost from the key derivation.
- Removing or editing the rune before the measurement returns.

## STOP triggers

1. Both counts are zero → STOP; the probe did not fire and the measurement is void.
2. `:483` is zero while `:559` is not → STOP and report; the hoisted path is not being exercised
   and the comparison has no baseline.
3. You find yourself hoisting → STOP. This strike measures.
