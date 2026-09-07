# EXPECTATIONS — clamp the request, not the handler

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE. ▪ = REPORT.

★ `publish` is the point and is still a REPORT — the stone controls the *emitted shape*, not the
wall. But STOP-2 makes the non-recovery case a hard stop, which is where the real information is.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the handler is spliced ONCE** | read the emitted `page-clamped`; count `~outcome-match` | exactly **one** occurrence |
| 2 | ★★ **behaviour is identical** | `probe-a-read-declares-its-page.wat` ×2 | byte-identical to today: `page-n=64;some-cur=yes;all-n=250;ordered=yes;first-n=1;pages=4;recv-n=64;recv-max=64` |
| 3 | ⛔ **the clamp still clamps** | same probe | `receive :limit 1000` still returns 64. A `let` that rebinds nothing would pass row 1 and silently drop the bound |
| 4 | ⛔ **delivery exact** | `circuit.wat` ×3 | `total=8000; distinct=8000; dup=0` |
| 5 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 6 | ⛔ **blast radius** | `git status --porcelain` | `wat/service.wat` + probes + SCORE. **Nothing else** |

## REPORTS

| ▪ | what | target |
|---|---|---|
| a | ★★★ `publish` median ×3 at 25 ms pinned | **18466** (was 19912 with the duplication) |
| b | `receives` | ~4700–4780, unchanged either way |
| c | `setup` / `stop` | drifted to 12.5 / 6.9 this session — the next target after the m=8 buckets |

## RUNTIME

15–30 min. One expression moves; the work is the measurement.

## TRAP DOORS

- ⚠⚠ **Row 3 is the silent failure.** Binding `~req-binder` to something that is not the clamped
  request — or shadowing it in a way the handler does not see — passes row 1, looks like a win on
  `publish`, and quietly removes the response bound we just built. Check the probe, not the timing.
- ⚠⚠ **STOP-2 is the valuable outcome if it fires.** If `publish` does not recover, the
  duplication was not the cost, and I will have been wrong about a mechanism for the sixth time
  today. Report it rather than looking for a second explanation inside this stone.
- ⚠ **Pin BOTH delay sites or neither.** Patching only `circuit.wat`'s parent silently measures
  adaptive; that produced a false result for me earlier today.
- ⚠ **Do not touch the write-side guard.** It wraps `send-recv-form`, not a handler — verify that
  claim if you look, but fixing it is STOP-3, not this stone.
