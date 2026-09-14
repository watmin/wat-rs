# EXPECTATIONS — the topic can be slow

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline, this tree (`29a90655a`): floor **5247**/5247, 0 FAIL, **535.565 s** (orchestrator's own,
quiet box). clippy 0. Happy path `distinct=8000;dup=0`.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑ **disarmed is byte-identical** (STOP-1) | happy path at today's argv | `distinct=8000;dup=0` and **every** reported counter unchanged. Any drift ⇒ the injector is in the wrong place |
| 2 | ⭑ **the relation holds at full rate** | `delay-bp 10000`, small n | `delays-fired == delay-draws`. A relation, not a threshold — deliberately |
| 3 | ⛔ **negative control** | `delay-bp 0` | `delays-fired=0` **and** `delay-draws=0`. A counter that ticks while disarmed measures the wrong thing |
| 4 | ⭑⭑ **THE DELIVERABLE — the crash fires on purpose** | scratch-pad run, `delay-ms` > 10 s | the publisher dies at `circuit.wat:2270`, *"recv: timed out — the peer is alive and silent"*. ⭑ **The death IS the pass** |
| 5 | the park is a timer, not a sleep | read the diff | `(recv (after … Milliseconds …))`, inlined into the service form. No `sleep`. `mora` holds |
| 6 | the seed is per-tier | read the diff | not the shared seed — `circuit.wat:2950` records what sharing cost |
| 7 | tests compile | `cargo nextest run --release --no-run` | clean |
| 8 | floor | `./scripts/floor.sh` → **Summary** | `5247 passed`, 0 FAIL. A count that GREW is fine and must be said; a SHRINK is a finding |
| 9 | clippy | `cargo clippy --all-targets -D warnings` | 0 |
| 10 | ⚠ **floor delta — REPORT, do not gate** | vs 535.565 s | disarmed adds one comparison per reply; ~0 expected. **Say the number either way** |
| 11 | blast radius held | `git status --porcelain` | `sns-fanout.wat` + `circuit.wat` only. 0 `.rs`, 0 goldens, 0 `wat/` |
| 12 | scope stated | the SCORE's headline | it says an instrument was built, NOT that anything was made resilient. `circuit.wat:2270` still kills the publisher — **on purpose now** |

## Runtime prediction

**60–90 minutes.** The mechanism is proven and copyable; the cost is the wiring (two files, one argv
slot) and the slow proof's wall-clock.

⛔ **The floor's TIMEOUT WALL is 30 s** and a test timed out at 40 s under contention this week. The
slow proof is ~12 s by construction and belongs in a scratch-pad run, not near the floor (STOP-3).

## Trap-door risks, ranked

1. ⭑⭑ **Row 1 drifting.** The one way this stone does harm is by changing behaviour while disarmed.
   Every existing number in this harness is a baseline someone will compare against later.
2. **Parking the topic parks EVERYTHING** — it is a serializing actor. At a high rate the circuit will
   not fill, exactly as `disrupt-bp` at 100 % already stalls it. Expected, and the reason for a rate.
3. **Counter at the dice roll instead of the park.** That is the `disrupt-hits` defect verbatim: a
   counter that could never increment, whose zero took three independent causes to explain.
4. **Row 4 not reaching the arm.** If a 10 s+ park does not fire the publisher's `TimedOut`, the
   deadline is not where the DESIGN thinks it is — report that, it is a finding either way.
