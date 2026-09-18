# DESIGN — measure `col_field_of` before hoisting it; §4 may be §1 again

> Drawn 2026-09-06 at HEAD `4f52da56b`. Source: vigilia 2026-09-05 `temperare` §4.
> **This DESIGN does not ask for a hoist. It asks for a number, and REFUTING THE ROW IS A SUCCESS.**

## Why this is a measurement and not a cure

`temperare` §1 said `join_extend` did three lookups per emitted pair. Driven, it was **one** —
`span_from_row` is skipped whenever the element already carries a BindSpan. I had read the callee's
prologue without checking whether the call site reaches it.

**§4 has the same shape, and I nearly wrote the same brief.** `key_of_el` (`fire/mod.rs:1719`) opens:

```rust
if el.binds.len > 0 {
    let el_b = element_fact_bindings(el, intern.bind_keys, intern.vals, intern.pool);
    return key_of(&el_b, join_keys, intern.val_ids);   // ← col_field_of NEVER runs
}
```

`col_field_of` is reached only on the **`binds.len == 0`** path. On the indexed path — the one the
D2/A1 work showed dominates the driven axes — it is not called at all.

## What is verified, and what is not

**Verified on disk:** `col_field_of` (`fire/mod.rs:1662`) is a pure function of
`(intern.alpha_id, join_key)` — no element input. Its body is two `HashMap<i64,_>` lookups plus
**two linear `position` scans**, one over `intern.bind_keys` with `Value` equality. It is called from
`key_of_el`'s unary arm and once per join key in the n-ary arm. Seven `key_of_el` call sites exist;
three (`hash_join.rs:187`, `:320`, `:441`) construct `GatherIntern::from_wm` inside the loop.

**NOT verified — and these are the strike's questions:**
1. **How often does `col_field_of` actually run** on the driven axes? If the `binds.len > 0` early
   return dominates, §4's cost is near zero and the row should be closed as refuted.
2. **Is `from_wm` a cost at all?** `temperare` called it "8 borrows"; reading it, it is eight
   reference copies and an `i64` — a struct literal that should inline away. I do not believe this
   one, and the counter can say so.

## The one contract decision, pinned

**Instrument, drive, report. Hoist ONLY if the number earns it.**

- a `#[cfg(test)]` counter on `col_field_of` entries, and one on the `binds.len == 0` branch of
  `key_of_el`, so the two questions are separable;
- driven on the axes that already exist (the fanout and accum cells, and the `hash_join` axes the
  three in-loop sites sit on);
- **then** a decision, stated in the SCORE:
  - **large** → hoist it, gate the predicted count, exactly as §1 and §3 did;
  - **near zero** → **REFUTE THE ROW.** Say so, record the numbers, and close `temperare` §4 as
    measured-and-not-a-cost. That is a completed strike, not a failed one.

⛔ **NO WALL-CLOCK CLAIM**, in either outcome. Counts only.

## Why refuting is worth the strike

Four `temperare` rows remain on the board, each rowed as an L1 on a reading. §1 was overstated 3×.
If §4 is near-zero, the board is carrying cost claims that no measurement supports, and every future
reader inherits them. **A rowed L1 that turns out not to be a defect is worth exactly as much as one
that is — and only a count can tell them apart.**

## Scope

**IN:** the two counters, the readings, the decision, and — only if the number earns it — the hoist
and its predicted-count gate. Floor GREEN either way.

**OUT, affirmatively cut:** `temperare` §2 and §5 (their own strikes); the FxHashMap question; the
`from_wm` construction unless the counter shows it matters; census B, D–M; A4, D2p, F2.
