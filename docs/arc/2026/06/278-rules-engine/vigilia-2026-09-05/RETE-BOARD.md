# RETE-ONLY BOARD — vigilia 2026-09-05, re-scoped

> **Builder's ruling, 2026-09-05:** *"we should only be working on rete here… main is doing a lot of
> cleanup across the code base… this session is dedicated to making the rete code an exemplar."*
>
> `WORK-LIST.md` holds all 81 L1 / 97 L2 across the whole substrate. **This file is the subset that
> is rete.** Everything not listed here belongs to main. Every row below was grounded at HEAD
> `5924f664b` when this file was written.

## Out of scope — do not work these here

CLASS Ω (CLI / `distribution/`), CLASS B (doc-comment enumerations, mostly `runtime.rs`/`check.rs`),
CLASS C (the `:wat::` vocabulary, control chars, signing), CLASS E (unowned trees, `examples/`,
`.wat.bad`), all of `partire`'s splits, `conformare`'s error types, `secare`'s io_uring, `purgare`'s
`ChildHandle`, `intueri`'s doc comments, `exigere`, `excusare`. **Ω4 is halted — see
`../strike-config-silent-setters/HALTED.md`.**

Already landed this session and outside scope, kept because reverting green work is worse:
`strike-mode-parity` (`src/distribution/`).

---

## DRIVEN — reproduced by execution, not read

| id | site | what | status |
|---|---|---|---|
| **F1 ★** | `wat/rete/oracle/explain.wat:10-49`, doc at `:53` | **`fire-rules-explain$oracle` is NONDETERMINISTIC.** 8 samples, 8 producing rules: native stable `vex::aaa` 8/8; oracle returns **four** distinct rules, agreeing 2/8. Single-producer control stable on both. `harvest-support` folds `(:wat::core::PersistentMap/keys network)` — HAMT order, **no sort** — while its own doc claims *"First-producer-wins, matching the native index."* The sibling `wat/rete/oracle/fire.wat` has four **sorted-ids** walkers and states the law. **The referee for explain is not a function**, and the differential compares only `PersistentMap/length`. | OPEN |
| **F2** | `wat/rete/oracle/insert.wat:100` | `retract` removes **every** equal fact; `insert` stages one. Driven: 2 identical inserts → 3 facts; 1 retract → 1; the derived consequence vanishes. Documented as "by value equality", so an asymmetry the code states — but the multiplicity loss is real. | OPEN |
| **D3** | `fire/pass/mod.rs:152`; `fire/mod.rs:2090`, `:2100`, `:2124` | **Four `beta_written` sites still bypass `record_token`/`record_tokens`**, and `pass/mod.rs:24-28` claims *"a future site cannot push without counting."* **Mutation-proved invisible**: dropping the census at a bypass → 100 tests, 100 passed; dropping it inside the door → RED. No census world contains a `:where`. A1 did not touch these. | OPEN |
| **D2p** | `src/rete/reachability.rs:1659-1665` | A discrimination row that has **never executed** — the rewrite targets the miss face of a constant, so `replacen` no-ops and the `if` is never entered. Driven: swapping in the `assert_ne!` its two siblings use goes RED on the first iteration. Note `src/rete/mod.rs:86` wraps the file in `#[cfg(test)]`. | OPEN |
| **D1** | `src/rete/kernel/tests/right_index_counter_invariant.rs` | The D2 acceptance test is a **tautology** — `indexed_n[J] == Σ\|buckets[J]\|` holds by construction; verified by grep that nothing mutates those fields outside `session.rs`. One possible outcome. **A1's cure gives the left side the same shape, so this now under-guards both.** | OPEN |

## CLASS A remnants — the D2 shape, still in rete

A1 is CURED (`0ee56325f`) and proved the pattern: **at least some CLASS A rows are unfinished halves
of cures we already shipped.** Check each against that lens before treating it as fresh.

| id | site | pair | consequence |
|---|---|---|---|
| A8 | `fire/pass/alpha.rs:85`, `:139-145`, `:217` | `class_ids` `(Vec<u32>, bool)` + `any_mixed`, two disjoint `&mut` arms | **fact loss**, not duplication. Header says *"THE CURE IS THE `bool` BELOW"* — a bool cure maintained by convention |
| A3 | `rete/compiled_cond.rs:219-234`; `export.rs:1415` | `slot_keys` / `output_slots` parallel arrays, hand-checked at one of two writers | a rule silently stops matching; the guard returns `None`, indistinguishable from "did not match" |
| A4 | `value/value.rs:848-857`, `:1031-1047`; `fire/delta.rs:188-196` | `identity` memo vs a walk computing a **different function** on miss; `seen_ids`/`seen_rest` are two halves of one set | fixpoint dedup breaks if any cross-variant `eq` arm is added |

## Cost — `temperare`, all five in the fire path

No measurement is claimed; each row names its own trip count from a test the ward read.
**Gate the ratio, never the millisecond** — and `temperare`'s own L3: every counter measures
occurrences of a named operation, none measures lookups performed, so all five are invisible to the
gates by construction.

`join_extend` 3 SipHash lookups per emitted pair (`fire/mod.rs:686-724`) · `root_join_delta` 5 map
ops per element on keys fixed by the enclosing loops (`pass/root_join.rs:59-79`) · `production_delta`
an `entry` per derived fact with the identical hoist documented 100 lines away
(`pass/production.rs:124-127`) · `key_of_el`'s `col_field_of` hoisted in two places and in none of
the three `hash_join.rs` per-element loops · `ensure_gather` re-deriving its own cache key per token.

## Instruments — the census names its own quantities wrong

The `recon/census-name-audit.md` sweep returned **13 sections (A–M)** of *the counter's name says X,
the quantity is Y*, all in the rete census: `filter:test-pass` counting passes ∪ elided-passes ·
`match:calls` counting calls that had a pattern · `prod:vec-alloc` carrying a hardcoded ×2 ·
`dbeta:alloc` a 0/1 flag under an allocation name · `seed:mixed-class-activate` class-shaped, per-FACT.
**Every performance claim in this arc rests on these.** Unrowed until now.

## Spec-vs-code, rete only

`conferre` L2-1..3: a leading accumulate re-seeds every round into a cumulative beta while leading
`:not`/`:exists` does not · `insert-all`'s hardcoded `OP` defeats the stated reason its checker is
parameterised · stratify's `+1` for `:exists` / accumulate-`:from` over a derived type diverges
between native and oracle.

## Carried from A1, deliberately cut there

`sequi` L2-a — the catch-up's right-index walk pushes the **whole** alpha memory rather than
`right[already..]`, unlike `keyed_join_persistent`. D2's protection is now `is_keyed` on the left
type rather than a detached cache, so this is safe to revisit on its own.

`key_and_index` uses `keys.entry().or_insert()`: a second call keeps the original key list while
indexing with the caller's `key_of_tok`. Idempotent across today's two callers; a third caller
computing a different list would be hidden rather than surfaced. A `debug_assert` closes it.

---

## Recommended order

**F1 first.** It is driven, it is the *referee* for explain being nondeterministic, and the fix is
one `sort` at the check rung with a shared `topological-node-ids` verb at the shape rung — the law
is already written down in the sibling file. Then **D3** (the false structural claim, mutation-proved
invisible), then **A8** (fact loss), then the census names, which everything else's numbers depend on.
