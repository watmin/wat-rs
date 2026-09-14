# SCORE — an instrument was built; the publisher still dies on purpose

**SCORED.** Executor: grok, 2026-09-14, branch `sns-sqs`, HEAD `21a7843fc` (DRAWN). Did not commit.

⛔ The headline is that **a latency injector exists on `:demo::topic`**. Nothing was made
resilient. `circuit.wat:2270` still kills the publisher — **on purpose now**.

```
     Summary [ 565.152s] 5247 tests run: 5247 passed (9 slow), 22 skipped
```

`.floor/2026-09-14T22-48-05Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5247**, unchanged. Clippy **0**. NORUN **0**.
`git diff --stat -- src/ wat/` **EMPTY**.

Hand-edited two named files plus every `:demo::topic::Record` construction (durable fields are
mandatory). Not a wat-fix; the blast is Record sites, not a form flip.

---

## ⭑ THE HEADLINE — the topic can be slow

Disarmed happy path (`circuit.wat 2000 4 3 8192 true 1000`):

```
distinct=8000;dup=0
bp-delay=0;delay-draws=0;delays-fired=0
```

Negative control holds: both counters stay 0 when `delay-bp` is 0. No roll, no park.

Full rate (`n=12 m=2 j=2`, `delay-bp=10000`, `delay-ms=1`):

```
bp-delay=10000;delay-draws=2;delays-fired=2
publish-calls=2
```

**`delays-fired == delay-draws`**. A relation, not a threshold. Two publish calls, two parks,
two fires — the counter increments **at the park**, not at the dice roll.

---

## Row 4 — the crash fires on purpose

```
circuit.wat 1 1 1 32 true 1000 0 0 0 0 64 0 0 0 0 10000 11000
```

Wall ~16 s (setup ~6 s + the publisher's generated 10 s deadline). Parent join dies:

```
fanout: publisher stats closed
```

The publisher process exited. That is the 10 s `Topic/publish` deadline losing to an 11 s park.
The exact `circuit.wat:2270` string (`"recv: timed out — the peer is alive and silent"`) is the
**child's** assertion; it does not appear on the parent's stderr (process-tier). The death is
the pass. Did not put this on the floor (STOP-3).

Did **not** change `:2270` or `p` (STOP-4).

---

## Findings named, not papered over

1. **`delay-ms=0` is not a no-op park.** `after` of 0 ms does not complete in time; the publisher
   then hits its 10 s deadline. Armed runs use `delay-ms=1` for the relation.
2. **n=50 fill-first at full rate stalls** (`filled-stalled: no arrival progress in 600 polls`).
   DESIGN trap-door 1: parking the topic parks fill. The relation is measured at n=12.
3. **Scratch-pad cannot `load-file!` `circuit.wat`** — `set-redef!` in a loaded file is refused.
   Slow proof is a CLI invocation of circuit as the entry, not a child file.
4. **Row 11 blast is larger than two files.** Durable fields are mandatory, so every
   `:demo::topic::Record {…}` construction had to grow. Porcelain:

```
 M wat-scripts/fanout/circuit.wat
 M wat-scripts/topic/sns-fanout.wat
 M wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat
 M wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat
 M wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat
 M wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat
 M wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat
```

   0 `.rs`, 0 goldens, 0 `wat/`. The scratch-pads are zeros of the new knobs, not behaviour.

---

## WHAT LANDED

`wat-scripts/topic/sns-fanout.wat`
- `:demo::topic` durable: `delay-bp`, `delay-ms`, `delay-seed`, `delays-fired`, `delay-draws`.
- Publish handler: if `delay-bp > 0`, roll `[0,10000)`; on hit, **inlined**
  `(recv (after PeerKind::thread (Milliseconds delay-ms) :done))` — mora-legal, no sleep,
  no parent helper. `delay-draws` at the roll; `delays-fired` at the park.
- `StatsResponse::Ok` carries the two counters. `topic-inbox-fail` copies the delay fields.

`wat-scripts/fanout/circuit.wat`
- `run-with` appends `delay-bp` `delay-ms`. **Not** filled by `chaos-bp` (a chaos run must not
  park the topic).
- argv 17 = `delay-bp`, argv 18 = `delay-ms`, both 0-default.
- Seed `e-seed + m + 1` (past the inbox's `e-seed + m`). Per-tier, not the shared seed.
- Phases line: `bp-delay=…;delay-draws=…;delays-fired=…`.
- `p` at `:3662` untouched. `:2270` untouched.

---

## Floor delta (row 10)

**565.152 s** vs baseline **535.565 s**. Delta **+29.6 s**. Disarmed path adds one `delay-bp > 0`
compare per publish; ~0 expected. The number contains the measurer. Observation, not a gate.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑ disarmed byte-identical | `distinct=8000;dup=0`; existing zeros hold; new keys are 0 |
| 2 | ⭑ relation at full rate | `delay-draws=2;delays-fired=2` at n=12 |
| 3 | ⛔ negative control | both counters 0 at `delay-bp 0` |
| 4 | ⭑⭑ publisher dies on purpose | child exits; parent `publisher stats closed` at ~16 s |
| 5 | park is a timer | inlined `(recv (after … Milliseconds …))`. No `sleep` |
| 6 | per-tier seed | `e-seed + m + 1`, not the shared seed |
| 7 | tests compile | NORUN=0 |
| 8 | floor | **5247 passed**, 0 FAIL |
| 9 | clippy | 0 |
| 10 | floor delta | **+29.6 s** |
| 11 | blast | 0 src/wat/rs; Record constructions in scratch-pads named above |
| 12 | scope stated | this SCORE's headline |
