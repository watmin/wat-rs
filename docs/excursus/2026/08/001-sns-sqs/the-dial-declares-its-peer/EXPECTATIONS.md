# EXPECTATIONS — the dial declares its peer

**Written BEFORE the strike**, from `899530a3e`.

## What this stone is graded on

That the runtime death becomes a **compile-time refusal**, and that nothing the corpus legitimately does is
refused. Both halves; the second is where a check like this usually goes wrong.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ the refusal FIRES | probe: handler dials an `Address` field, no `:peers` | a **`macro-error`** naming `:peers` and the surface, **at compile time** |
| 2 | ⭑⭑ it fires where it used to DIE | the FINDING's variant A | **never runs** — previously `Disconnected []` / `RuntimeError ["unknown function…"]` |
| 3 | ⭑ the declared redial still compiles | probe: same shape **+** `:peers` **+** `:ephemeral` peer | compiles and runs |
| 4 | ⭑⭑ **the strongest corpus control** | `sqs.wat`'s queue **handler-dials the store SIX times** (`:830 :866 :900 :1296 :1329 :1360` — this session's own §2d redials), `:peers [:wat::query::Store]` | **all six accepted.** Plus `circuit.wat:370`'s worker redialing `seen` |
| 5 | ⭑ holding ≠ dialing | `sns-fanout.wat` holds `Address<demo::Topic>` and `Address<demo::TopicWorker>` with `:peers [:queue::Queue]` only | **not refused** |
| 6 | ⛔ NOT the strict rule | read the diff | the check consults the `:impls` connect sites, not just field declarations |
| 7 | ⛔ `:init` untouched | read the diff | the walk is over `:impls` only |
| 7b | ⛔ top-level `defn`s untouched | `sqs.wat:1945`/`:2034` — helpers taking an `Address` **parameter** and `connect`ing it | **not refused** — they are not in any `:impls` |
| 8 | message reads as a sibling | compare to `:896`/`:913` | same family — names the surface and the fix |
| 9 | ⛔ no `src/` | `git diff --stat -- src/` | **EMPTY** |
| 10 | ⛔ `LociDiedError` untouched | `git diff` | unchanged |
| 11 | floor | `./scripts/floor.sh` → **Summary line** | green at **5239**, 0 FAIL, no `ARM.txt` |
| 12 | clippy | `-D warnings` | exit 0 |
| 13 | happy / chaos | the two commands | `distinct=8000;dup=0` · exit 0 |
| 14 | ⚠ corpus-wide effect reported | the SCORE | how many `defservice`s the check ran over, and that **none** tripped (or which did) |

## ⭑ Rows 4 and 5 are the ones I expect a careless check to fail

Row 4 is a **handler dial that must be allowed** (surface declared). Row 5 is a **held address that must not be
treated as a dial**. A check that gets row 1 right and either of these wrong has replaced a runtime crash with
a compile-time false refusal, which is worse — it blocks correct code.

## ⚠ What I will reject

- **The strict rule** (row 6). Measured to falsely refuse three corpus files.
- **Any `:init` involvement** (row 7 / STOP-5).
- **`src/` or `LociDiedError` touched** (rows 9–10 / STOP-3, STOP-4).
- **Row 2 claimed without running the old reproducer.** The point is that a specific thing that *used* to die
  now cannot compile.
- **An existing `defservice` silently made to pass** by weakening the rule (STOP-2). If the corpus trips it,
  that is a finding to report, not a rule to soften.

## Runtime prediction

**40–70 minutes.** The `:impls` walk is the new part; the two sibling checks give the structure. Floor
~500–560 s, and the check runs at every `defservice` macroexpansion so the whole corpus exercises it for free.

## Trap-doors, ranked

1. ⛔ **Strict rule** (row 6) — falsely refuses correct code in three files.
2. ⛔ **Refusing the worker's declared redial** (row 4).
3. ⛔ **Treating a held address as a dial** (row 5).
4. **Walking `:init`** (row 7).
5. **A message that reads like an unrelated third rule** (row 8).
