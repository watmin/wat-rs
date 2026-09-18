# EXPECTATIONS — diagnostics in source order

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,408 plus every arm you drive.**

## The scorecard — pre-values driven at HEAD `645f219c4`

| # | what | state AT HEAD (driven, 16 runs each) | required after |
|---|---|---|---|
| 1 | ★ the two fixtures are stable | ⛔ `c2_mixed_macro_swap` **5/11**, `w2a_kwargs` **8/8** | **one output over 24 runs each** |
| 2 | ★ the order is MEANINGFUL, not merely stable | errors arrive in `HashMap` order | **source order** — file, line, col |
| 3 | ★ the sort is TOTAL | — | mutation 2: a partial key re-randomises a same-span pair, **or an honest "no such pair exists"** |
| 4 | removing the sort re-breaks it | — | mutation 1 over **24 runs**, not 2 |
| 5 | the set is unchanged | 4 errors per fixture | **same 4**, same spans, same messages — only order moves |
| 6 | `SymbolTable` untouched | `functions: HashMap` | **unchanged** — hot lookup path |
| 7 | the quarantine moves on evidence | `QUARANTINE_LEN = 2` | 0 **only if** the gate proves it; otherwise stated why not |
| 8 | floor / lints / clippy | **`5408 tests run: 5408 passed (1 slow), 21 skipped`**, 0 FAIL, lints **258**, clippy rc=0 | ≥ 5408 + arms, 0 FAIL, lints ≥ 258, rc=0 |

## Runtime prediction

**40–70 minutes.** The sort is a few lines; the 24-run stability runs and the tie-break proof are the
work. **Budget three release rebuilds.**

## Trap doors named in advance

- **⛔ A PARTIAL SORT KEY LEAVES HASH ORDER IN THE TIES.** Row 3. Sorting on `(line, col)` and calling
  it deterministic is the defect wearing a sort.
- **24 runs, not 2.** At p≈0.5 two runs miss a flip half the time. C19's sweep needed 24/file, and its
  own first run caught a file a 2-run scan had missed.
- **The floor's `(1 slow)` on `reachability_shard_0_of_6` is NOT new** — it appears across floors at
  many HEADs this session, count oscillating 0–6. A timing annotation on a PASS.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.

## What would make this strike a failure even if every test passes

**A stable but arbitrary order.** De-randomising into hash-stable order would satisfy the gate and
serve no reader. The point is that a person reading four errors gets them in the order they occur in
their file; anything less is the defect with the symptom suppressed.

**And a sort whose ties are unproven.** If no same-span pair is ever constructed, the tie-break is an
untested branch that the next same-span program will exercise for the first time in production.
Row 3 accepts an honest "no such pair exists" — it does not accept silence.
