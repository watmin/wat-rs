# EXPECTATIONS — a ledger is a receipt, not a lock

Written **before** the strike. Every "before" is a run of mine on `42b3dd610`.

| # | what | command | expected |
|---|---|---|---|
| 1 | **★★ the stranding stays closed** | tiny drop-after ×6 | completing runs `total = 100`, `distinct = 100` |
| 2 | **★★ the rate-0 baseline is restored** | circuit ×5 | `total=8000; distinct=8000;` **`dup=0`** ×5 |
| 3 | the mechanism probe holds | `./target/release/wat wat-scripts/scratch-pad/probe-a-ledger-is-a-receipt-not-a-lock.wat` | `s1 …=0 (LOST); s2 …=1 (no-loss); s3 …=2 (duplicate-not-loss)` |
| 4 | **the floor** | `./scripts/floor.sh` — **the Summary line** | `5214 passed`, 19 skipped |
| 5 | the receipt count is the message count | tiny ×6 | `seen-recorded = 100` on completing runs |
| 6 | ★ timings, reported not gated | circuit ×5 | record `setup/publish/drain/stop`. Before: publish `45356–46520`, drain `197–214`, stop `5643–6555` |
| 7 | `claim deadline exhausted` | tiny ×6 | **report the count.** Before: 3/6. Not this stone's job |
| 8 | the rename is contained | `git diff --stat` | `circuit.wat` + `probe_arc278_sane_circuit.rs` only |

### Before-state, recorded verbatim

```
row 1   total ∈ {89,90,90,91,89} of 100 (claim-before, pre-DupSelf); seen-firsts=100 ×6
row 2   total=8000; distinct=8000; dup=0 ×5; seen-dups = 7 7 10 7 7
row 4   Summary [ 357.607s] 5214 passed, 19 skipped   .floor/2026-09-05T05-59-47Z/
row 6   publish 45356 45401 45550 46033 46520 | drain 197–214 | stop 5643–6555
row 7   3 of 6 died: claim deadline exhausted;depth=3;attempts=3;elapsed=601
```

## ⛔ ROW 2 IS THE ROW THE LAST STONE DIED ON

`dup=0` at rate 0, five runs. The previous attempt produced `dup ∈ {2,0,2,0,4}` — a worker
re-emitting something it had already emitted. **Record-after must not reintroduce that**: a
`Recorded` check means someone reported it, and the emit is skipped.

⚠ **`dup=0` is asserted HERE and only here**, and for a stated reason: no worker dies in a
completing rate-0 run, so nothing is irreducible. It is **not** a general invariant — the
general one is `distinct=N; dup >= 0`, and STOP-3 of the previous brief was wrong to assert
otherwise.

## ⛔ WHAT THIS STONE CANNOT SHOW, AND WILL NOT CLAIM

**The dead-owner loss is not measurable at circuit scale.** A worker that dies aborts the run
(row 7), so the loss it would cause is never counted. **The structural claim rests on the probe
(row 3), not on the circuit.**

★ Say so in the SCORE. It becomes circuit-observable only after the `claim deadline exhausted`
crash is repaired, and that is a different stone. **Do not report row 1 or row 2 as evidence
that the dead-owner hole is closed** — they are evidence that nothing regressed.

## RUNTIME PREDICTION

40–60 min. The surface split and the two impls are small; the care is in the worker — the emit
must precede the mark, and the T1 deadline `select` now wraps `check` instead of `claim`.

## TRAP DOORS, NAMED

1. **`mark` before `emit`.** The whole stone is the ordering. A `mark` moved earlier to simplify
   the fold is claim-before under a new name, and every row here would still pass.
2. **`check` with a side effect.** A read that writes is a lock. STOP-2.
3. **The `_` arm at `circuit.wat:479`** — it asserts on anything that is not First/Dup and will
   fire on the correct new path.
4. **Two seen round-trips per message now.** Row 6 exists to see it. Report it; do not fix it.
5. **A green floor proves nothing about rows 1–3.** The floor runs rate 0 with the drop cells
   ignored; row 2 is the only floor-adjacent evidence, and rows 1 and 3 are outside it entirely.
