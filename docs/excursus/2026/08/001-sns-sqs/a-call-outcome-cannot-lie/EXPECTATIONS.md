# EXPECTATIONS — a call outcome cannot lie

Written **before** the strike. Every "before" is a run of mine on `51211338f`.

| # | what | command | expected |
|---|---|---|---|
| 1 | **★★ the site that ignored the code can no longer** | read `circuit.wat:397-401` | it `match`es three arms; **no `first`/`second` on a call result anywhere** |
| 2 | ★★ rate-0 identical | circuit ×5 | `total=8000; distinct=8000; dup=0; seen-recorded=8000` |
| 3 | ★★ tiny identical | tiny ×6 | **6/6 terminate**, `total=100; distinct=100; dup=0; seen-recorded=100` |
| 4 | **the floor** | `./scripts/floor.sh` — **the Summary line** | `5214 passed`, 19 skipped |
| 5 | the enum is the only new stdlib form | `git diff wat/service.wat` | one `defenum` + the helper's arms; **nothing above `:896` moves** |
| 6 | goldens undisturbed | floor | no `peers_bijection` failure |
| 7 | blast radius | `git diff --stat` | `service.wat` + `circuit.wat` only — **no `.rs`** |
| 8 | timings unchanged | circuit ×5 | publish in `45547–46716` |

### Before-state, recorded verbatim

```
row 2  total=8000; distinct=8000; dup=0 ×5; seen-recorded=8000; seen-skipped 2 6 9 14 5
row 3  6/6; total=100; distinct=100; dup=0; seen-recorded=100; seen-skipped 14 13 18 16 16 16
row 4  Summary [ 359.199s] 5214 passed, 19 skipped   .floor/2026-09-05T07-55-54Z/
row 8  publish 45547 45923 46074 46100 46716
```

## ⛔ ROW 1 IS THE ONLY ROW THAT IS NOT "NOTHING CHANGED"

Rows 2–8 all assert **sameness** — this is a refactor, and a moved number means the arms are not
equivalent to the codes (STOP-2). **Row 1 is the whole point**: the shape must make the ignored
discriminator unignorable.

★ The check is structural, not behavioural: after this, `(None, 0)` and `(Some x, 2)` have no
form. If the executor can still write either, the enum has not replaced the pair — it has been
added beside it.

## RUNTIME PREDICTION

25–40 min. The `defenum` and the helper's four returns are small; the four call sites are
mechanical. Budget a rebuild after the `wat/` edit.

## TRAP DOORS, NAMED

1. **The stdlib is frozen at build time.** No rebuild → every row reports the *old* helper.
2. **A refactor that "improves" behaviour is a failed refactor.** If a call site's redial policy
   changes because the arms made a better one obvious, that is STOP-2 — report it as a finding
   for the next stone, do not take it here.
3. **`:wat::enum::Pure`.** The probe's enum carries it; an enum declared without it may not
   behave the same across the wire. Copy the probe exactly.
4. **A green floor proves nothing about row 1.** Row 1 is read off the diff, not run.
