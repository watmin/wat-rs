# FINDING — the induced failure rate per component, measured — and the chaos gate was inert

Builder: *"so... in our chaos engineering... what is our induced failure rate per component?"* then
*"how about you prove it empirically... make all the things fail at some rate."*

**The short answer: it was 5 % on two reply paths of the subscriber queues and ZERO everywhere else,
and the one injector that was supposed to be aggressive fired ZERO times at its shipped rate.** Now it
is measured, per component, and armable from one switch.

---

## 1. The census, before anything was built

| injector | knob | reachable from CLI | counted its own fires |
|---|---|---|---|
| sub queue · receive reply | `drop-recv-bp` | YES (argv 8) | **NO** |
| sub queue · ack reply | `drop-ack-bp` | YES (argv 9) | **NO** |
| inbox queue · receive + ack | — | **NO — hardcoded 0** | **NO** |
| seen · check reply | `drop-check-bp` | **NO — pinned 0 in `main`** | **NO** |
| seen · mark reply | `drop-mark-bp` | **NO — pinned 0** | **NO** |
| worker + topic-worker · disrupt | `disrupt-rate-bp` | **NO — pinned 0** | partly (`disrupts`) |
| store (all m+1) | — | no knob exists | — |

**Two of five reachable. One of five counted, and that one counted the wrong thing (§4).** The drop
*decision* was computed at `sqs.wat:833` / `circuit.wat:132,176` and **discarded**, so the only
evidence a fault fired was downstream — `ack-retries`, `redeliveries`, `seen-skipped`. That is
inference, not measurement.

⛔ **Tier 1 had no fault injection at all.** `circuit.wat`'s inbox queue took literal zeros while the
sub queues one binding above took the parameters — so `ack-after-sends` and `ok = min over subs`, the
topic's two partial-failure properties, **had never been executed.** That is owed-ruling #8, confirmed.

## 2. What was built

- **Every injector counts its own fires**, at the site that performs the drop, never at the decision:
  `recv-drops` / `ack-drops` / `recv-replies` / `ack-calls` on `:queue::Counters`+`Stats`,
  `check-drops` / `mark-drops` / `check-calls` / `mark-calls` on `:fanout::seen::Record`,
  `disrupt-fires` on `:fanout::worker::Record`.
- **`chaos-bp` (argv 13)** — one rate in basis points arming every injector that has no explicit rate.
  Plus `drop-check-bp` / `drop-mark-bp` / `disrupt-bp` as argv 14/15/16, so each is armable alone.
- **The report prints the EFFECTIVE rate beside the observed one** (`bp-recv`, `bp-ack`, `bp-check`,
  `bp-mark`, `bp-disrupt`, `chaos-seed`), resolved once in `run-with` so the value that reaches a
  service and the value printed cannot diverge.
- `:fanout::SeenFinal` replaces a 3-wide Tuple that now needs seven fields.

## 3. The measurement, one injector at a time, bp = 500 (5 %)

`2000 4 3 8192 true 1000`, quiet box, through `scripts/capped.sh`, seed 20260910.

| armed | exit | observed | wall-ms | result |
|---|---|---|---|---|
| nothing | 0 | — | 20 721 | `distinct=8000 dup=0` |
| seen `check` | 0 | 43/843 = **5.10 %** | 21 275 | `8000/0` |
| seen `mark` | 0 | 43/843 = **5.10 %** | 21 711 | `8000/0` |
| queue `recv` | 0 | pooled 50/1052 = **4.75 %** | 54 890 | `8000/0` |
| queue `ack` | 0 | pooled 52/1044 = **4.98 %** | 54 600 | `8000/0` |
| recv **and** ack | 0 | 4.75 % / 4.98 % | **94 975** | `8000/0` |
| `disrupt` | **2** | — | DIES | — |
| all five | **2** | — | DIES | — |

★ **A 5 % reply drop on receive or ack costs 2.6× wall clock and stays perfectly correct**; both
together cost **4.6×**. The seen service's drops are nearly free. Correctness never wavered in any
surviving arm — `distinct=8000 dup=0`, every run.

## 4. ⛔ Two counters I built were wrong, and their own numbers exposed both

**(a) `recv-drops` covered one of two paths that drop a reply.** `receive` has THREE exits: take,
immediate-empty, and park. The first two both suppress a reply under `hit?`; the third registers a
`Waiter` and has no reply yet. My counter incremented only on *take*, while the denominator counted
all three — so the inbox read **1.87 %** against a 5 % setting while every sub read 5.0 %. The inbox
was the only tier that showed it **because it is the one that parks.** Fixed by counting the fire
wherever a reply is actually suppressed and dividing by `recv-replies`, not `receive-calls`. The
corrected inbox reads 8/205 = **3.90 %**, 0.7σ from 5 % at n=205.

**(b) `disrupts` counts TEARS, not FIRES.** `disrupt-hits` increments only when `tore?` — when the
poisoned call came back `lost`/`closed`. So `disrupts=0` was indistinguishable between *never
injected* and *injected repeatedly and the fault did nothing*. `disrupt-fires` now separates them.

★ Both are the same class: **a counter that covers one of the paths that can do the thing is not a
slow counter, it is a wrong one.**

## 5. ⛔⛔ THE BIG ONE — the chaos gate was green because it never injected

At the shipped gate's own rate (`:user::chaos` → `run-chaos* … 200 42`):

```
disrupt-bp=200   disrupt-draws=257   disrupt-fires=0   disrupts=0   exit 0
disrupt-bp=100   disrupt-draws=254   disrupt-fires=0   disrupts=0   exit 0
```

257 rolls, **zero fires**. The intrinsic is not at fault —
`wat-scripts/scratch-pad/probe-disrupt-draw-is-uniform.wat` replays the exact loop shape and reports
uniform draws, range [2, 9994], and `rate=5000 → 133/257` against an expected 128.5.

★★★ **The cause is a shared seed.** Every worker and topic-worker received the same `e-wseed`, so all
m·j of them **replayed one identical sequence**. Each only draws ~21 times per run, so the fleet's
effective sample was **21 draws, not 257** — and the probe shows that 21-draw prefix contains **zero**
hits at 200 bp (`PREFIX n=21 -> hits=0`, `n=43 -> hits=0`). P(no injection at all) = 0.98²¹ ≈ **0.65**:
the gate injected nothing in roughly **two runs out of three**, deterministically per seed.

**Fixed:** per-worker seed `e-wseed + 97·qi + wi`, per-topic-worker `e-wseed + 9001 + twi`, per-queue
`e-seed + i` and inbox `e-seed + m`.

⛔ **I introduced half of this defect and left the other half in place for an hour.** I decorrelated the
queues after measuring four subs reporting `recv-drops=11` and `recv-replies=212` identical to the
digit — and did not ask the same question of the workers until their own number forced it. That is
`[[feedback_fix_the_sibling_or_gate_the_class]]`, reproduced inside the session that wrote it down.

### And with the fault actually firing, the system dies at 1 %

```
BEFORE the seed fix:  disrupt-bp=200 → exit 0, fires=0
AFTER  the seed fix:  disrupt-bp=100 → exit 2
                      disrupt-bp=200 → exit 2
```

**Two distinct fatal modes**, both captured whole and neither re-run beyond the deterministic replay:

1. **bp=100 — a SUBSTRATE bug.** `:fanout::worker/stop` (`circuit.wat:2551`):
   `:wat::core::match: no arm matched scrutinee of type wat::core::Enum; exhaustiveness should be
   caught at type-check time`. A match the type-checker accepted as exhaustive, failing at runtime.
   Reachable **only** when the disruptor fires, which it never did before today.
2. **bp=200 — `"fanout: publisher stats lost"`** (`circuit.wat:2276`, via `publishers-all-done?`).
   The publisher died and **left no message of its own**; the entire log is three lines. Only the
   harness's dial reports it, as `Lost`, with no cause.

⚠ **The mechanism of (2) is NOT established** and I am not naming one. The disruptor poisons the
worker↔seen link; why a *publisher* dies is unexplained, and the publisher's silence is itself the
obstacle.

## 6. Two independent reasons the gate was toothless, stacked

The floor is **green at 5237/5237, 22 skipped, 0 FAIL** (`.floor/2026-09-11T01-51-38Z/`) *after* the
seed fix — and the count is unchanged. So the chaos tests that would now inject are among the **22
skipped**. The gate was inert *twice over*: **ignored in nextest**, and **unable to fire even if run.**
A green floor was never evidence about chaos.

## What this rules

- The induced rate is now **measured per component and armable from one switch**, with observed
  printed beside effective on every run.
- **4 of 5 injectors at 5 % are survivable and correct**; reply drops cost 2.6–4.6× wall clock.
- **The disruptor at ≥1 % is fatal**, in two different ways, one of them a checker/runtime
  disagreement in `src/`.
- ⛔ **Every chaos result this harness has ever published for the disruptor is void** — not wrong, void:
  the fault did not fire. Prior reply-drop results stand; they were reachable and are confirmed here.

⚠ **Scope:** one configuration, `2000 4 3 8192 true 1000`, 1–3 runs per arm, one seed. The 5 % figures
are pooled across 5 tiers (n≈1050) and solid; the disruptor threshold between 0 and 100 bp is **not**
bisected, and the 2.6× cost is 1 run per arm.
