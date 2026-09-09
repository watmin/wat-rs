# EXPECTATIONS — a claim remembers its owner

Written **before** the strike. Every "before" is a run of mine on `23eec1f10`.

| # | what | command | expected |
|---|---|---|---|
| 1 | **★★ the stranding closes** | `cargo nextest run --release --run-ignored only -E 'test(r2_drop_after_tiny)' --no-capture`, ×6 | `total = `**`100`** on every completing run (before: `{89,90,90,91,89}`) |
| 2 | **⛔ no double-emission** | same runs | `dup=0` **and** `distinct == total` on every run |
| 3 | **★★ the rate-0 baseline returns** | `./target/release/wat wat-scripts/fanout/circuit.wat` ×5 | `seen-dups=`**`0`** ×5 (before: `7 7 10 7 7`) |
| 4 | rate-0 invariant | same ×5 | `total=8000;distinct=8000;dup=0` ×5 |
| 5 | the ledger still sees each message once | row 1 runs | `seen-firsts=100` every run (unchanged) |
| 6 | **the floor** | `./scripts/floor.sh` — **the Summary line** | `5214 passed`, 19 skipped |
| 7 | the mechanism probe still holds | `./target/release/wat wat-scripts/scratch-pad/probe-a-claim-remembers-its-owner.wat` | `discriminates=yes` |
| 8 | `claim deadline exhausted` | row 1 runs | **report the count.** Before: 1/6. Not this stone's job to change |

### Before-state, recorded verbatim

```
row 1/5  total ∈ {89,90,90,91,89} of 100; seen-firsts=100 ×6; seen-dups ∈ {13,15,15,16,16}
row 3    seen-dups = 7 7 10 7 7      (rate 0, five runs)
row 4    total=8000;distinct=8000;dup=0 ×5
row 6    Summary [ 360.461s] 5214 passed, 19 skipped   .floor/2026-09-05T05-08-25Z/
row 8    1 of 6 died: claim deadline exhausted;depth=3;attempts=3;elapsed=601
```

## ⛔ ROW 1 IS THE REFUTATION ROW

`total = 100` is the whole claim. **If `total` stays below 100, the DESIGN's mechanism is wrong**
— the stranded messages are not taking the `DupSelf` path — and the honest outcome is
**refute**, not a patch. Say so and hand it back.

## ⛔ ROW 2 IS THE GUARD ON ROW 1

Row 1 alone is satisfiable by cheating: emit on every `Dup` and `total` hits 100 while genuine
duplicates get emitted twice. **Row 2 is what makes row 1 mean something.** `dup=0` and
`distinct == total` must both hold, or row 1 is not a pass.

## RUNTIME PREDICTION

30–50 min. The surface and service changes are small; the care is in the worker's emit arm and
the `_` catch-all at `:479` that currently asserts on any response that is not First-or-Dup.

## TRAP DOORS, NAMED

1. **The `_` arm at `circuit.wat:479`** — `"claim not First/Dup"`. It will fire on the *correct*
   new path if it is not updated. A green-looking crash with a misleading message.
2. **`dups'` in the service.** `DupSelf` must not increment it (DESIGN, consequence 2), and
   `seen-dups` is a printed summary field — row 3 is where a mistake here shows.
3. **Idempotency of `DupSelf`.** Two retries must both answer `DupSelf`. If the second flips to
   `First` the ledger was rewritten and the count is wrong.
4. **A green floor proves nothing about rows 1–3.** The floor runs rate 0 and the drop cells are
   `#[ignore]`d. Rows 1, 2 and 3 are the only evidence this stone works.
