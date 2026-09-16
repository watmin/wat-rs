# EXPECTATIONS — both ends bound the same handshake

Written **before** the strike. Graded on the orchestrator's **own** re-runs.
Executor: a spawned **Opus** subagent (named per examinare; grok's credits exhausted).

Baseline (`07e481f82`): floor **5251**/5251, 0 FAIL, clippy 0, happy path `distinct=8000;dup=0`.
Classification baseline: **BOUND = 4**.

⚠ Floor delta as a **BAND**, never a number — this box's noise floor is ≥ ±16 s.
⛔ **`timeout -k`** on anything that may block; SIGTERM does not stop a blocked `wat` (125 s measured).

| # | what | expected |
|---|---|---|
| 1 | ⭑⭑ **the BOUND list goes to zero** | all four sites call `recv-by-deadline`; re-running the classification's own grep shows no bare `recv` at `spawn.wat:531`/`:588`, `test.wat:329`/`:438` |
| 2 | ⭑⭑ **ONE constant, in `wat/spawn.wat`** | named by all four sites **and** by `child-main`. `grep -c` for the new name ≥ 5 |
| 3 | ⛔ **the old constant is GONE** | `:wat::service::CHILD-MAIN-STARTUP-DEADLINE-MS` returns **0** matches. Two constants with one value is the drift this stone exists to prevent |
| 4 | ⛔ **no arm changes** | the four `TimedOut` arms already exist and are unedited. If one had to change, it is reported |
| 5 | ⭑⭑ **every service and every test still starts** | `./scripts/floor.sh` → `5251 passed` or higher, 0 FAIL. `launch` is on the path of **every** spawn, and `test.wat` is on the path of every spawned test program — the floor **is** this test |
| 6 | ⭑ **the timeout is reachable, driven** | a child that neither crashes nor announces readiness must make the launcher report the `TimedOut` arm instead of hanging. ⚠ If it cannot be induced, **say so** — an honest absence, not a fabricated fixture |
| 7 | ⭑ happy path | `distinct=8000;dup=0`, completeness counters identical |
| 8 | clippy | `--all-targets -D warnings` → 0 |
| 9 | tests compile | `cargo nextest run --release --no-run` clean |
| 10 | ⚠ **test.wat sharing the number is reported** | if a hermetic test program needs longer than 30 s, that is a **finding**, not a second constant |
| 11 | blast radius | `spawn.wat`, `test.wat`, `service.wat` only |
| 12 | scope stated | both ends of the spawn handshake are bounded. Supervision still deferred |

## Runtime prediction

**60–120 minutes.** Four one-line swaps plus a constant move; the cost is `spawn.wat` being manifest
position 171 (a stdlib rebuild, and everything loads after it) and driving row 6.

## Trap-door risks, ranked

1. ⭑⭑ **Row 5.** `spawn.wat` is upstream of nearly the whole stdlib. If this breaks, it breaks everything —
   loudly, on the floor, which is why the floor is the real gate rather than a formality.
2. **Row 3 half-done.** Leaving both constants alive recreates exactly the drift this stone removes, while
   *looking* symmetric.
3. **Row 6 faked.** A fixture that "proves" the timeout by asserting the constant exists proves nothing.
   An honest "could not induce" is worth more.
4. **Row 4.** An arm edit would mean `recv-by-deadline` does not in fact return the same `RecvOutcome` —
   which contradicts a measurement, and should stop the stone rather than be worked around.
