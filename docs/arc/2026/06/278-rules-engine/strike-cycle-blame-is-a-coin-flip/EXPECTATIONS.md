# EXPECTATIONS — the blame stops being a coin flip

> ⚠ **This strike cures ONE of C20's three quarantined files.** The other two are a driven-different
> root (check-phase error ordering). A report claiming C20 closed is wrong.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,402 plus every arm you drive.**

## The scorecard — every pre-value driven at HEAD `04abe37fc`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the blamed function is stable | **12 runs → 6 `:probe::b` @ line 5, 6 `:probe::a` @ line 8** | **12+ runs, ONE outcome** |
| 2 | ★ the order is unrepresentable, not sorted | `HashSet<String>` at ~6 sites | `BTreeSet<String>` at **every** site; zero `HashSet` left for this value |
| 3 | ★ the regression test reads IDENTITY | — | mutation 2 (`.rev()`) REDs — proves it is not just "an error appeared" |
| 4 | reverting the type REDs | — | mutation 1, **with the run count justified** (⚠ 2 runs miss a p≈0.5 flip half the time) |
| 5 | the quarantine can tell cured from broken | `QUARANTINE_LEN = 3` | **2**, `probe_arc278_rete_defn_recurse_mutual` row removed; mutation 3 shows a stale row REDs |
| 6 | the other two stay quarantined WITH evidence | 2 rows, captured evidence in the header | unchanged, evidence intact |
| 7 | no behaviour change | — | **no golden moves.** If one does, STOP-1 fired and it is reported, not absorbed |
| 8 | floor / lints / clippy | **`5402 tests run: 5402 passed (2 slow), 21 skipped`** (439.5 s, 0 FAIL rows), lints **254**, clippy rc=0 | ≥ 5402 + arms, 0 FAIL, lints ≥ 254, rc=0 |

## Runtime prediction

**45–70 minutes.** The type change is mechanical; the regression test's run count and mutation 2 are
the work.

## Trap doors named in advance

- **⛔ SAMPLE SIZE IS THE TRAP.** At p≈0.5, "observe both outcomes in N runs" is a false green at
  `0.5^(N−1)` — **50% at N=2**. (⛔ this row first read `2·0.5^(N−1)`, which is **1.0** at N=2; the
  headline was right and the formula was not. Corrected by the rider, 2026-09-04.) C19's own sweep needed **24 runs/file over 280 files**, and its
  first run caught a third file a 2-run scan had missed on a coin flip. A regression test that runs
  the fixture twice proves almost nothing.
- **One surviving `HashSet` re-opens the hole** and makes the type change a lie. Row 2 is
  "every site", not "the loop".
- **Determinism is not a licence to change WHICH error is reported.** Row 7: no golden moves.
- **`git checkout <sha> -- <path>` STAGES.** Verify restores by hash.

## What would make this strike a failure even if every test passes

**A regression test with too few runs.** It would go green on a bug that survived, and it would do so
*most* of the time — which is worse than no test, because the next hand reads green as proof. Row 1
says 12+ and row 4 requires the count to be justified, not asserted.

**And claiming C20 closed.** Two of its three files are a different root, driven, and unfixed. The
row shrinks; it does not close.
