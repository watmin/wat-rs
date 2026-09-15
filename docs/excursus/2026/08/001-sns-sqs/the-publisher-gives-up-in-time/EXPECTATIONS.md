# EXPECTATIONS — the publisher gives up in time, not in tries

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`ecc4fc682`): floor **5251**/5251, 0 FAIL, **537.427 s** (orchestrator's own, quiet box).
clippy 0. Happy path `distinct=8000;dup=0`.
Slow-topic scenario today: `exit=2`, `"publisher stats closed" ×3`.

⚠ **The executor's floor wall clock is not comparable to this baseline** — five stones running:
executor +29.2/+29.6/+18.0/+23.2/+27.2 s where the orchestrator measured +7.5/+0.96/+0.24/+0.025/+1.6 s
on the same trees. Report the number; the orchestrator's run grades row 5.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **the slow topic no longer kills the publisher** | the slow-topic record invocation | no `assertion-failed!` from the publisher. Today `exit=2` + `publisher stats closed` ×3 |
| 2 | ⭑⭑ **the give-up is NAMED and reaches the report** (STOP-2) | same run | an exhaustion carrying **elapsed-ms and ceiling-ms**, visible in the output. A silent stop is a fail |
| 3 | ⛔ **the payload is TIME, not tries** | read the enum | no attempt count in the exhaustion variant. Both struck rulings are satisfied only by this |
| 4 | ⛔ **negative control: a healthy run never exhausts** | injector disarmed | `Exhausted` not produced. A ladder giving up on a healthy topic is measuring the clock wrong |
| 5 | floor | `./scripts/floor.sh` → **Summary** | `5251 passed` or higher, 0 FAIL. A SHRINK is a finding |
| 6 | ⭑ **happy path byte-identical** | the record invocation | `distinct=8000;dup=0` and every existing counter unchanged |
| 7 | clippy | `cargo clippy --all-targets -D warnings` | 0 |
| 8 | tests compile | `cargo nextest run --release --no-run` | clean |
| 9 | ⭐ **the sibling was CHECKED, not skipped** (STOP-4) | the SCORE | says `circuit.wat:747` was read and already returns `SeenRetry::Exhausted`, and was deliberately left alone |
| 10 | ⛔ `Verdict` and `require!` untouched (STOP-3) | `git diff` | `:1389` and `:1398` unchanged. Different ruling, different population |
| 11 | ⚠ the ceiling's default, stated | the SCORE | where it defaulted and **why that value** — not a number chosen silently |
| 12 | blast radius | `git status --porcelain` | `circuit.wat` only. 0 `src/`, 0 `wat/`, 0 goldens |
| 13 | scope stated | the SCORE's headline | the publisher gives up in time. `recv-by-deadline` still unmeasured; the worker's ladder unchanged |

## Runtime prediction

**90–120 minutes.** The shape is in the same file (the ack ladder); the cost is the six arms, the
`:fanout::Input` field rippling through every construction site (fields are mandatory — the injector hit
7 files on the same class of change), and getting the exhaustion through `publisher-stats` to the report.

## Trap-door risks, ranked

1. ⭑⭑ **Row 2 silent.** The likely wrong "pass" is a publisher that stops crashing and stops
   publishing, with `distinct=8000` failing for no stated reason. A crash at least says something.
2. **Row 3 spelled as a count.** `Exhausted[tries]` would satisfy one ruling and violate the other's
   whole point. The payload is the decision.
3. **Row 6 drifting.** The ceiling must not change a healthy run. Every counter in this harness is
   somebody's later baseline.
4. **Row 1 passing via STOP-1's hole** — a ceiling under 10 s gives up before the first attempt
   completes, which looks like a fix and publishes nothing.
