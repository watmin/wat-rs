# EXPECTATIONS — the store can fail

**Written BEFORE the strike**, from `730552b5f`.

## What this stone is graded on

Not a green floor — the proxy is new userland, so green mostly proves it type-checks. It is graded on
**three arms firing that have never fired**, and on the fault being the *honest* one.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ `TimedOut` fires on a store call | run the harness with `store-drop-reply-bp` live | the `sqs.wat:845` or `:1279` arm's raise text, **captured verbatim** |
| 2 | ⭑⭑ `Lost` **or** `Closed` fires | `store-die-bp` live | the `:780`/`:814` or `:1215`/`:1248` arm's raise text |
| 3 | ⭑ the write LANDED before the reply died | the proxy's own counters + a store read | the row is **present** in the store although the caller got no reply |
| 4 | forward-then-suppress, not skip | read the diff | the `Store/<op>` call happens **before** the roll's effect; no code path returns `None` without forwarding |
| 5 | counted at the suppression site | read the diff | `drops-fired` incremented where the `None` is produced, **not** at the dice roll |
| 6 | all six features forwarded | grep the proxy | `ensure-schema put delete count-index scan scan-index` — six, none stubbed |
| 7 | pass-through is byte-identical | rates at 0 | the real store's reply returned unchanged; no proxy-held state but counters + RNG |
| 8 | no stdlib, no Rust | `git diff --stat -- src/ wat/` | **EMPTY** |
| 9 | unfaulted happy path unchanged | `circuit.wat 2000 4 3 8192 true 1000`, **proxy not inserted** | `distinct=8000;dup=0` |
| 10 | floor | `./scripts/floor.sh` → **Summary line** | **5237 passed, 0 FAIL**, no `ARM.txt` |
| 11 | clippy | `-D warnings` | exit **0** |
| 12 | the 10 s cost is respected | the SCORE's own commands | run sized for it; no command that looks like a hang |

## ⚠ The result I EXPECT and will accept as success

**The tier dies.** Those six arms `assertion-failed!` today, so a fired fault kills the queue tier and the
run ends non-zero. **That is the correct outcome of this stone** — it converts "six arms nobody has ever
executed" into "six arms with captured evidence of what they do." A strike that makes the tier *survive*
has done the **next** stone and violated STOP-4.

## ⛔ What I will reject

- **A fault that skips the store call.** Row 4. It inverts §2d: the write definitely did not land, so the
  arm fired on a state it was never written for. This is the single most likely way to produce a
  green-looking result that proves nothing.
- **A counter at the dice roll.** Row 5. `disrupt-hits` is still broken at `ec3ea95c8` for exactly this,
  and the earlier chaos-rate stone was drawn to fix the same class. A third instance is not acceptable.
- **A stubbed feature.** Row 6.
- **Any `rt-store` number measured through the proxy.** It adds a hop; the number would be a lie.
- **`wat/` or `src/` touched.** Row 8 / STOP-3.

## Runtime prediction

**45–70 minutes.** Six forwards is the bulk; the fault logic is small. Two harness runs at ~10 s per
injected fault, plus floor ~490–520 s.

## Trap-doors, ranked

1. ⛔ **Skip-instead-of-suppress** (row 4) — silently wrong, looks right.
2. ⛔ **Counter at the roll** (row 5) — third instance of a known class.
3. **10 s per fault** — misread as a deadlock by the next reader if the run is sized wrong.
4. **A stubbed sixth feature** — passes a shallow probe, corrupts a real run.
5. **Measuring perf through the proxy** — +2 crossings per op.
