# EXPECTATIONS — `vis` is swept, not chosen

Written **before** the strike. The parameter is the stone; the sweep is its output.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ `vis-ms` is a parameter | `grep -n 'vis-ms' circuit.wat` | on `run-with` and parsed in `:user::main` |
| 2 | ★ `0` preserves today's behaviour | `… circuit.wat` (no args) | every field **identical** to before |
| 3 | wrappers unchanged | `grep -n 'run-with n m j' circuit.wat` | every existing wrapper passes the default |
| 4 | the knob turns | `… circuit.wat 500 4 3 8192 true 5000` | runs; `distinct=2000; dup=0` |
| 5 | ★ **the sweep is reported** | six values at n=2000 | each: completed-or-`drained-never`, `drain`, `distinct`, `dup`, `ack-retries`, `ack-exhausted` |
| 6 | correctness at every completing point | the sweep | `distinct` = n×m and `dup=0` wherever a run finishes |
| 7 | `limit-ms` still derives from `vis` | read `:522` | unchanged — STOP-4 |
| 8 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 9 | scripts load | `every_wat_scripts_file_loads` | green |
| 10 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **Row 5 is the stone.** Not "n=2000 completes" — **the sweep, reported in full.** A sweep in which
nothing completes is a result: it means `limit-ms = vis/1e6` is the wrong coupling, which no choice
of constant could have shown.

⚠ **Row 2 is what keeps this safe.** A new knob that changes the default is a behaviour change
wearing a parameter's clothes.

⚠ **Sweep on a quiet box.** `drain` at n=500 measured 549 ms quiet and **1122 ms under a running
floor**. Check `ps` first; the breadcrumb's freshness probe says so and I ignored it once today.

## Runtime prediction

**30–45 minutes** for the parameter. The sweep is six runs at n=2000: ~1 minute each when they
complete, ~4 when they strand — so **10–25 minutes** depending on how many complete. The floor is
the long pole.

## Trap-doors named in advance

- **`vis` is nanoseconds, the parameter is milliseconds.** `1000000000000` ns = 1000 s = `1000000`
  ms. An off-by-1000 gives a 1-second visibility timeout that looks plausible and reclaims live work.
- **`0` must reach the *existing* conditional**, including the drop-run branch — a drop run with
  `vis-ms 0` must still get 200 ms, not 1000 s.
- **Do not thread a real value through the wrappers.** They pass `0`; that is what keeps row 3 and
  the floor unchanged.
- **The argv position is after `fill-first?`.** A missing sixth argument must default to `0`, not
  raise.

## What this stone does NOT claim

⚠ **It ships no constant.** STOP-5: the sweep reports, the builder rules.

⚠ **It does not explain the slope.** It unblocks the n=2000 point that the `PersistentMap` win is
still unmeasured at — confirmed −18 % at n=1000, unconfirmed at n=2000.

⚠ **It does not decouple the retry bound from `vis`.** That is the fork's second branch, and the
sweep is what decides whether it is needed.
