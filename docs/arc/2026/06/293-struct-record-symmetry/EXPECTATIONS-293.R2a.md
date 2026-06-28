# EXPECTATIONS — 293.R2a: one `register_aggregate_methods` for accessor codegen

Independent scorecard, fixed BEFORE the strike. Scored by the orchestrator's own forced-clean re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the R2 parity probe flips GREEN (un-ignored) | `cargo nextest run --release -E 'test(aggregate_codegen_parity_generic_record_accessors)'` | PASS — `(:r2::probe)` = 60 |
| 2 | generic core-record accessor resolves | the probe's `:r2::CR/v` (no `UnresolvedReference`) | resolves |
| 3 | generic holon-record accessor resolves | the probe's `:r2::HR/v` | resolves |
| 4 | **policy a — struct ↛ comms (preserved)** | the existing struct-portability / wire-gate tests (`grep -rln 'is_portable\|portable\|struct.*wire\|channel_of' tests/` → the send'/recv' struct-rejection tests) | green (struct still rejected at comms) |
| 5 | **policy b — record edn-repr (preserved)** | the record EDN round-trip tests (`-E 'test(core_record_def)'` + the wire/cap handoff probes) | green |
| 6 | **policy c — holon ⊂ core (preserved)** | the probe's `(:r2::want-core (:r2::HR 20))` guard + `-E 'test(holder_substitution)'` | green |
| 7 | monomorphic struct/record accessors un-regressed | `-E 'test(core_record_def)' + test(defstruct) + binary(types)` | green |
| 8 | `register_record_methods` is GONE | `grep -n 'fn register_record_methods' src/runtime.rs` | no hit (folded into `register_aggregate_methods`) |
| 9 | whole workspace green, SET-diff ∅ | `cargo nextest run --release` (forced-clean) | floor 0 — `4098 passed / 0 failed / 92 skipped` (the R2 probe adds 1 pass, un-ignored; nothing else moves) |

## Independent prediction
- **Runtime:** 40–70 min. A real refactor (extract + fold + delete one fn + the call-site swap), but the shape is
  pinned and the struct accessor loop is the worked template. The subtle part is the index unification
  (struct position vs record inherited+own) and the bare-key fix.
- **Load-bearing rows:** #1 (the break is fixed) + #9 (no regression) + #4/#5/#6 (the three policies survive —
  the merge must not weaken the holder wall).

## Trap-door risks (named)
- **The `<T>`-in-name mangling (STOP-1).** If the accessor stays unresolved after the bare-key fix, the `<T>` is
  in `AggregateDef.name` itself — a parser/registration bug, not the accessor loop. Weigh #2/#3 against the disk.
- **Index contract mismatch (STOP-2).** `struct-field` and `Record/field-at` must agree on the absolute index
  the shared loop computes — a silent off-by-one would pass resolve and return the WRONG field. Verify #1 returns
  exactly **60** (10+20+30), not a scrambled sum.
- **Double accessor source (STOP-3).** If the `defrecord` macro also emits accessors, folding triggers
  `DuplicateDefine` — that surfaces the R2b macro-thinning question; surface it, don't silence it.
- **Policy regression.** The whole point is "holder is the ONLY variance" — if the merge accidentally makes a
  struct portable or breaks holon⊂core, #4/#5/#6 catch it. Do not let them be skipped.

## What "done" means
#1–#3 + #7–#9 green by the orchestrator's own forced-clean re-run; #4/#5/#6 confirm the three policies survive;
`(:r2::probe)` returns exactly 60 (right fields, not a scramble); `register_record_methods` is gone; the diff is
read end-to-end. Then commit on green; un-ignore stays.
