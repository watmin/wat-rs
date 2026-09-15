# EXPECTATIONS — a dead runner names its orphan

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`2ea49a6bd`): floor **5251**/5251, 0 FAIL, **537.959 s** (orchestrator's quiet box), clippy 0.
Probe today: `clean=[0 2 4 6 8 10 12 14]`, then `"bracket collect-loop: runner 0 crashed: …"` — **no item
named**.

⚠ **The executor's floor wall clock is not comparable** — seven stones running: executor
+29.2/+29.6/+18.0/+23.2/+27.2/+22.0/+18.2 s against the orchestrator's +7.5/+0.96/+0.24/+0.025/+1.6/+1.0/+0.5 s.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **the orphan is named** | the probe | the failure identifies which item the dead runner held |
| 2 | ⭑⭑ **and it is the RIGHT item** | the probe kills item **3** | the message says **3**. A wrong index is worse than none |
| 3 | ⛔ **the arms still RAISE** | the probe | still a failure. No re-dispatch, no hang, no short answer |
| 4 | ⭑ **a clean map is unchanged** | the probe's control | `clean=[0 2 4 6 8 10 12 14]`, byte-identical |
| 5 | ⛔ **the existing brackets tests pass unchanged** | `cargo nextest run --release -E 'test(bracket)'` | green, and **not edited to accommodate** |
| 6 | ⚠ **an idle runner's death reads honestly** | the SCORE | says how the `-1` sentinel prints. Must not claim item 0 |
| 7 | ⛔ **`peer-pos` stability confirmed** (STOP-1) | the SCORE | states that `peers` is never compacted, or what the ledger is keyed on instead |
| 8 | ⛔ **no nested-Tuple fourth slot** (STOP-2) | `git diff` | a named aggregate or a parallel vector |
| 9 | floor | `./scripts/floor.sh` → **Summary** | `5251 passed` or higher, 0 FAIL. A SHRINK is a finding |
| 10 | clippy | `cargo clippy --all-targets -D warnings` | 0 |
| 11 | blast radius | `git status --porcelain` | `wat/bracket.wat` only |
| 12 | scope stated | the SCORE's headline | the orphan is named. `brackets/map` still fails on a dead runner |

## Runtime prediction

**60–90 minutes.** One binding and three messages; the cost is the ledger's shape (trap-door 2) and the
stdlib freeze.

## Trap-door risks, ranked

1. ⭑⭑ **Row 2 wrong.** A message that confidently names the wrong item is worse than today's honest
   anonymity — it would send someone to re-run work that completed fine.
2. **Row 3 drifting into a fix.** Re-dispatch is a contract change the builder has not ruled; the probe
   measured that removing the raise yields a hang or a silent short answer.
3. **Row 6 unanswered.** A runner dying while idle is the common case at pool teardown; if `-1` prints
   as item 0 the message lies routinely.
4. **Navigating by the failure's reported line.** It says `:624` for a raise at `:633`. Trap-door 1.
