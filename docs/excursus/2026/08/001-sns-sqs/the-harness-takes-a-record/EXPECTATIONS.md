# EXPECTATIONS — the harness takes a record

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`29a90655a`, +the slow stone if landed): floor **5247**/5247, 0 FAIL, **535.565 s**
(orchestrator's own, quiet box). clippy 0. Happy path `distinct=8000;dup=0`.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **equivalence** — the record reproduces today exactly | happy path via the record | `distinct=8000;dup=0` and **every reported counter identical** to the positional baseline. Both-green is NOT the gate |
| 2 | ⭑ **the defect is fixed** | `:vis-ms (Some 0)` | reaches the service as **0**, not the default. Unreachable today; this is why the stone exists |
| 3 | ⭑ **`None` still means default** | omit `:vis-ms` | the same value today's omitted slot produces |
| 4 | ⛔ **negative control** | a record missing `:n` | **refused**, not silently defaulted. A fallback here means the type is not working |
| 5 | the four overloaded zeros, each read | the SCORE | names all four (`:2920`, `:2929`, `:3666`, `:3699`) and what zero meant at each. A blanket conversion is a fail |
| 6 | `Eof` / `Stopped` decided | the SCORE | says what was chosen and why — not left to fall out |
| 7 | callers converted | `git status` | the 4 test files; `capped.sh` only if it is not arg-agnostic |
| 8 | ⛔ docs untouched | `git status -- docs/` | **empty**. Invocation strings in SCOREs are evidence, not documentation |
| 9 | tests compile | `cargo nextest run --release --no-run` | clean |
| 10 | floor | `./scripts/floor.sh` → **Summary** | `5247 passed` (or higher), 0 FAIL. A SHRINK is a finding |
| 11 | clippy | `cargo clippy --all-targets -D warnings` | 0 |
| 12 | ⚠ floor delta — REPORT, do not gate | vs 535.565 s | parsing one record vs 15 strings; ~0 expected. Say the number |

## Runtime prediction

**90–120 minutes.** The harness is copyable and the blast radius is small; the cost is the **four
overloaded zeros**, each of which needs its comment read and its intent decided, and the four callers.

## Trap-door risks, ranked

1. ⭑⭑ **Row 1 passing while measuring something else.** Every number in this harness is a baseline
   someone compares against later. Green-but-different is the failure mode, and it is silent.
2. **A blanket `0 → None`.** Four sites, four intents, one of which (`:3666`) explicitly distinguishes
   "default" from "no redelivery".
3. **`readln` blocking a test that forgets stdin** — a hang, not a failure, against a 30 s wall.
4. **Rewriting the SCOREs to match.** Tempting, wrong, and it destroys the record of what was run.
