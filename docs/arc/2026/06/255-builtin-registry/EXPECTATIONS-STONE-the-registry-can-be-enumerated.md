# EXPECTATIONS — STONE: the registry can be enumerated. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the verb exists and enumerates | the wat probe | 552 rows | measured: registry census + anchored grep AGREE at 552 |
| 2 | kind split sums to the total | the wat probe | SpecialForm + Intrinsic == 552 | a row has exactly one kind |
| 3 | cross-check against the Rust census | `-E 'test(probe_can_doc_types…)' --no-capture` | `total registry rows` == the probe's total | same `all_entries()` |
| 4 | the totality work list is countable | the wat probe | a number, first time ever from wat | `runtime-meta.wat:241` |
| 5 | the loader gate | `-E 'test(every_wat_scripts_file_loads)'` | pass | the probe is a `wat-scripts/` file |
| 6 | `wat` unit binary + reflection | the two scoped runs | green | new verb + new record + new scheme |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement — one moves, and it is EXPECTED

```
registry rows   552 → 553      ⬅ `:wat::intrinsic::rows` is itself a new registered row
GAP_A 49 · GAP_B 42 · TYPES_UNCHECKED 10     unchanged
DEBT  121 → 121 or 122         the new row gets a checker scheme (room 3 registers one), so it
                               should NOT land on DEBT. If DEBT rises, the scheme is missing.
```

⚠ **Row 1's "552" is the count BEFORE this stone.** The probe will see 553 including itself. Both
numbers are correct; the rider must say which it is reporting. This is the kind of off-by-one that
becomes a wrong number in the SEAM three sessions later.

## Runtime

**35-50 min.** Four sites plus a probe, against a template that shipped once. The enum fields
crossing the wat boundary is the only part without a line-by-line precedent — `Example` carries
`bool`/`keyword`/`WatAST`/`Option`, not the five axis enums.

## Trap doors, named in advance

1. **The axis enums may not cross cleanly.** `Example`'s fields are primitives; `Row`'s include
   five `:wat::runtime::*` enums. They are `wat_enum_from!`-derived and `metadata-of` already
   returns them inside a map — so the machinery exists — but *inside a defrecord field* is a
   position `Example` does not exercise. **This is the most likely STOP-4.**
2. **Load order.** The record must load before the verb's scheme resolves. `stdlib.rs:295` records
   that constraint for `Example`; the same applies and the failure mode is a startup error, not a
   compile error.
3. **The off-by-one on the total.** See the ledger note.
4. **Scope creep into the ledgers.** The moment enumeration exists, deriving the ledgers looks
   obvious and free. It would destroy them. STOP-2, and it is the single most important line in
   this brief.
