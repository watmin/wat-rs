# EXPECTATIONS — STONE: the three special-form tables. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the buckets re-derive | the BRIEF's `comm` instrument | 32 registered / 3 not | orchestrator's slurp count, hand-checked |
| 2 | bucket-1 deletions change nothing | `signature-of-defn` on `let`/`match`/`fn` | identical before and after | live probe: all three already render `@syntax` |
| 3 | the nine render identically | `signature-of-defn` on each of the 9 | identical before and after | STOP-2 |
| 4 | `apply` still rejects all 11 | the STOP-8 path for each name incl. `defn` | rejected, same diagnostic | room 3 |
| 5 | `wat` unit binary | `-E 'binary_id(wat)'` | green | — |
| 6 | reflection / wat_lang / function | the three scoped runs | green | 51 files touch this surface |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement

**None, and that is derived.** This stone registers no new row and retires no name: it deletes
duplicate rows and moves sketch text to `@syntax`. `@syntax` is not one of the five axes, so no
property grade changes.

```
GAP_A 49 · GAP_B 42 · DEBT 121 · TYPES_UNCHECKED 10 · registry 552 · corpus 37   ← ALL UNCHANGED
```

⚠ A ledger that DOES move means the stone touched a registration, which is out of scope — treat it
as a finding, not a bonus.

## Runtime

**30-45 min.** Nine `@syntax` additions with a before/after probe each is the bulk; the deletions
and the membership query are small.

## Trap doors, named in advance

1. **The multi-line `insert(` calls.** A single-line regex misses five of thirty-five. The
   orchestrator published 30 before catching it. The BRIEF hands over the `-0777` instrument.
2. **Arm 247 may not become fully unreachable.** It is guarded on `lookup_special_form(&n)`, and
   three rows survive — but those three (`defstruct`, `unquote`, `unquote-splicing`) are
   **unregistered**, so they can never match `Binding::Registered` in the first place. That makes
   247 look dead by construction; the rider must **demonstrate** it rather than argue it, and if it
   cannot, the arm stays.
3. **A bucket-2 `@syntax` that changes rendering.** The nine currently render via arm 227 (`args`)
   or arm 247 (the sketch); adding `@syntax` moves them to arm 201. If `args` was producing a
   different string than the sketch, the "identical before and after" bar catches it — which is
   why row 3 exists per-name rather than as one aggregate.
4. **`defn` silently folded in.** The tempting simplification, and it would make `apply` stop
   rejecting a macro. STOP-4.
