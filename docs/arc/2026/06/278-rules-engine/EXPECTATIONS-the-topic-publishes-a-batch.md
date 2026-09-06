# EXPECTATIONS — the topic publishes a batch

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE: an invariant this stone controls. ▪ = REPORT: an observation, recorded not gated.

★ The stone's PURPOSE is to cut `publish`, but a millisecond count is an observation. What the
stone **controls** is the number of publish calls — 200 instead of 2000 — and that is countable
and deterministic. Gating the time would gate a consequence, which has fired wrongly three times
in this arc.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the driver makes 200 calls, not 2000** | a counter in the probe, or the topic's `stats` ticks | 200 publish calls for 2000 messages |
| 2 | ★★ **delivery is still exact** | `circuit.wat` ×5 | `total=8000; distinct=8000` every run |
| 3 | ★★ **the cap rejects, nothing enqueued** | scratch probe via `:demo::Topic/publish` | 11 msgs → `RequestTooManyEntries(11,10)`, inbox depth unchanged |
| 4 | ★★ **10 still passes** | same probe | `Ok`, inbox depth `+10 × nsubs` |
| 5 | ⛔ **the cap is a readable def** | evaluate `:demo::Topic::PUBLISH-MAX-ENTRIES` | `10` |
| 6 | ⛔ **stamps stay per message** | read `circuit.wat`; the e2e histogram's shape | each message carries its own `t0`; `e2e` buckets still span a range, not collapse toward one value |
| 7 | ⛔ **`cap` is unchanged** | `grep -n ":cap " wat-scripts/topic/sns-fanout.wat` | `64` everywhere it was 64; `2` where it was 2 |
| 8 | ⛔ **both publish loops changed** | read `circuit.wat:1439` and `:1763` | both chunked |
| 9 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 10 | **blast radius** | `git status --porcelain` | `sns-fanout.wat`, `circuit.wat`, scratch probes, the SCORE. **No `wat/`, no `src/`** |

## REPORTS — recorded, not gated

| ▪ | what | why it is not a gate |
|---|---|---|
| a | ★ `publish + drain` median ×5 | the point of the stone, and still an observation. Prior: publish 23716, publish+drain ~23907 |
| b | ★★ **`Full` retry count** | the number that decides the cap trade. A 40-body batch against `cap 64` may bounce often — **that is a finding to report, not a reason to raise the cap** |
| c | the new dominant term, with numbers | rule 4: every perf stone names what leads next |
| d | `dup` per run | `distinct` is the invariant |
| e | `setup` / `stop` | untouched by this stone; tracked because they are the next target |

## RUNTIME

45–75 min. The surface and impl changes are small; the driver chunking plus keeping per-message
stamps honest is the real content, and there are two loops.

## TRAP DOORS

- ⚠⚠ **The cap is the likeliest source of a disappointing result, and the likeliest thing to be
  quietly tuned.** 10 × 4 = 40 bodies against `cap 64`. If `publish` does not improve, STOP-1 says
  report it with the retry count. Row 7 exists because raising `cap` would make row 2 and report
  (a) both look better while measuring a different system.
- ⚠ **Per-batch stamping is the easy mistake** — one `now()` per request reads naturally and
  destroys the e2e histogram. Row 6.
- ⚠ **Two publish loops.** `:1439` is the main circuit; `:1763` is a second entry point. Changing
  one leaves a mixed corpus that will confuse the next measurement.
- ⚠ **2000 ÷ 10 = 200 exactly**, so a chunker bug at the tail will not show here. If the count is
  ever not a multiple of the batch size, the last partial batch is the untested path — say so if
  the implementation cannot express it.
- ⚠ **`total=8000` is arithmetic, not proof.** `distinct=8000` is the invariant that catches a
  chunker duplicating or dropping a message; row 2 gates both together.
