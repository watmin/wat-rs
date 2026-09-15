# EXPECTATIONS — every waiter can bound its wait (the census)

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`7d1d6ad84`): floor **5251**/5251, 0 FAIL, clippy 0.

⚠ **Report the floor delta as a BAND, never a number** — this box's noise floor is ≥ ±16 s, and the
executor's box runs +18…30 s. Precision quoted on earlier stones was inside the noise.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **every primitive has BOTH controls** (STOP-1) | the SCORE | a PRESENT and an ABSENT named **per primitive**. Missing either ⇒ that primitive's number is unvalidated |
| 2 | ⛔ **controls are pinned to forms/names, not lines** (STOP-2) | the census source | no bare `file:NNN` control. A struck control in this corpus passed vacuously for weeks on exactly that |
| 3 | ⭑⭑ **the `select` test discriminates** | `call-by-deadline`'s select vs a timerless one | `bounded=yes` / `bounded=no`. If both land the same the test is inert and the census is worthless |
| 4 | ⭑ **`child-main`'s bare recv is reported unbounded** | the report | PRESENT, `primitive=recv bounded=no` — the headline instance |
| 5 | ⭑ **`bounded=UNKNOWN` is its own count** | the report | reported separately with the shape that defeated the walker. Never folded into `no` |
| 6 | ⭑ **exemplars per primitive** | the SCORE | ≥1 real `file:line` row each, plus the `home=`/`head=` split |
| 7 | ⛔ **both struck censuses still pass** | run them unchanged | their own controls fire; their numbers reported |
| 8 | ⛔ **nothing was bounded** (STOP-3) | `git status --porcelain` | one census under `wat-scripts/`. 0 `src/`, 0 `wat/`, 0 service scripts |
| 9 | `readln`'s real path found (STOP-4) | the SCORE | its registered path, or a stated reason it is not a `:wat::kernel::` primitive |
| 10 | floor | `./scripts/floor.sh` → **Summary** | `5251 passed` or higher, 0 FAIL. A SHRINK is a finding |
| 11 | clippy | `--all-targets -D warnings` | 0 |
| 12 | tests compile | `cargo nextest run --release --no-run` | clean |
| 13 | scope stated | the SCORE's headline | a census was taken. **Nothing was bounded** |

## Runtime prediction

**90–150 minutes.** The walk is a copy; the cost is the **structural `select` test** (row 3) and seven
control pairs.

## Trap-door risks, ranked

1. ⭑⭑ **Row 3 inert.** If `select`-has-a-timer cannot discriminate, every `select` lands in one bucket
   and the census reports a number that means nothing. This is the whole technical content of the stone.
2. **Row 1 partially done.** Controls for `recv` and `select` come free; `poll`, `accept`, `send`,
   `readln` must be found. A primitive with one control is a primitive whose count is a guess.
3. **Row 5 folded.** An `UNKNOWN` counted as `no` manufactures unbounded waiters that are actually fine
   — and would send the next stone after phantoms.
4. **Row 2.** The cautionary example is inside this very corpus and cost weeks of false green.
