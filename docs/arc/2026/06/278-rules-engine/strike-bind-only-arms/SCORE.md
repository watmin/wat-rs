# SCORE — C4, weighed against the orchestrator's own re-run

> Re-run at `53ae65822` + the rider's working tree. **Every number below is from my run, not the
> rider's report.** The cure is the one the strike drew; the sharpest finding is against my own
> scorecard.

## The scorecard

| # | pre-value | after, MY re-run |
|---|---|---|
| 1 | ★ table 1 production row **absent** | ✅ `Ap  activate, production bind_only  11.33 ms` |
| 2 | ★ table 3 production row **absent** | ✅ `Ap  activate, production bind_only  10.82 ms` |
| 3 | `A alpha_activate_fact` (both) | ✅ `A activate, skip_span forced off` + a six-line note naming the branch and stating `Ap` derives nothing |
| 4 | table 1 ladder nests | ✅ `M 11.93 → A 14.02`, `A−M push (A = skip_span off) 2.09` |
| 5 | table 3 ladder nests | ✅ `M 11.91 → H 12.05 → V 12.55 → D 13.16 → A 14.08`, `A−M 2.16` |
| 6 | ⚠ no NEW negative | ✅ none. `H−M` read `+0.14` here — see B |
| 7 | probe still green | ✅ 7/7 on `test(accum_alpha)` |
| 8 | radius | ✅ `accum_alpha_cost.rs` only, **+148 −26** |
| 9 | lints 196/196 | ✅ 196/196 |
| 10 | floor 5312 + probe | ✅ **`5313 tests run: 5313 passed, 21 skipped`**, exit=0 |
| 11 | clippy rc=0 | ✅ rc=0 |

## ⭐ A — BOTH `Ap` ROWS LAND BELOW `M`, WHICH IS THE WHOLE CLAIM

Table 1 `Ap 11.33` sits **below `M 11.93`**; table 3 `Ap 10.82` **below `M 11.91`**. The production
arm does *less* work than the arm that merely adds `exec_compiled`, because with the real map
`skip_span` fires and the exec never runs. That is the shape the probe predicted from `pool=0` vs
`pool=120,200`, now confirmed in time. **STOP-1 did not fire.**

**I drove mutation 2 myself** — the row I said I would read hardest, because it is the only check
that separates *a row that reads the map* from *a second arm that happens to differ*. Pointing the
map at `id + 1`:

```
A   activate, skip_span forced off   14.14 ms     Ap  ...  15.56 ms      (table 1)
A   activate, skip_span forced off   15.27 ms     Ap  ...  15.13 ms      (table 3)
```

`Ap` jumps from ~10.8–11.3 to ~15.1–15.6 — at or above `A`, straight back into the forced-off
regime. The row reads the map's **contents**. Restored and re-verified: `grep -c 'insert(id + 1'`
returns 0.

## ⛔ B — MY SCORECARD PINNED MILLISECONDS, AND THE RIDER CAUGHT IT

EXPECTATIONS recorded pre-values as absolute times: table 1 `M 12.09 → A 14.18 → A−M 2.09`. The
rider measured `12.80 → 16.28 → 3.48` **on the same box, before touching anything**. My own re-run
came back `11.93 → 14.02 → 2.09`. Three readings of one unchanged tree, spanning ~16% on `A`.

**This is the C8 lesson landing on my own instrument, one day after I wrote it down.** I had just
proven the grid cannot resolve <20% and promoted *the noise floor is measured, not assumed* — and
then wrote a scorecard whose rows are millisecond values. The invariants are what hold and what I
should have written: **`A > M`** (the ladder nests) and **`Ap < M`** (the branch fires). Both held in
all three readings; not one absolute value did.

Row 6 was stale the same way: it recorded `H−M` at `−0.00 ms`. It printed `−0.22` for the rider,
`+0.17` after, `−0.41` under mutation 1, `+0.14` for me. It is two nearly identical arms differenced
— **the sign is noise**, and treating "still ≥ −0.00" as a check would have gone red on nothing.

## ⛔ C — MY SKETCH WOULD HAVE MADE THE ARM TIME A LOOP PRODUCTION DOES NOT RUN

I wrote `bind_only_prod` as its own loop after `cond_key_ids`. `fire/delta.rs:340-347` builds **both
in one loop** — verified. The rider merged them, so the new arm's setup is production's setup rather
than an extra pass over `compiled_conds`. On a *timing* arm that is not cosmetic, and the rider
changed my sketch rather than following it.

## ⭐ D — THE READ-LIST HELD THIS TIME

All seven citations checked to the line (`delta.rs:70`, `:71-76`, `:339-346`; `accum_alpha_cost.rs`
`:231`, `:233`, `:269`, `:274`, `:1100`, `:1102`, `:1130`, `:1135`, `:1136`). After D4's `ArmLease`
citation pointed at a file with zero occurrences of it, I grepped every pairing while drawing this
one. No dead pointer.

## ⚠ E — TWO FINDINGS BANKED, NEITHER FIXED HERE

1. **`compiled:calls` is a designed union of both branches** — `delta.rs:78` and
   `compiled_cond.rs:928` both bump it, so it reads 80,200 either way. `accum_cost.rs:52` pins that
   number as a correctness assertion and cannot see which branch produced it. C3-adjacent; on the
   list.
2. **A rendering defect in table 1's in-fire block**, pre-existing and confirmed in my own output:
   `seed` / `delta` / `seed+delta` print two columns left of `FIRE` / `alpha`. Rust's `\`-newline
   continuation strips the *leading* whitespace of the next line, so the intended indent never
   reaches stdout. The rider flagged it and correctly did not fix it inside this radius.

## Per-arm status

| arm | status |
|---|---|
| table 1 `Ap` | **proven** — mutation 2 driven by me; both mutations discriminate |
| table 3 `Ap` | **proven** — same |
| old `A` rows | **proven**, untouched and still driven; now labelled with the branch they take |
| the C4 probe | **proven** — unchanged, still green, still the non-vacuity guard |
| `src/rete/kernel/fire/` | **not touched** — STOP-2 did not fire |
