# SCORE — find the 934 milliseconds

**STRUCK.** Executor: grok, 2026-09-06. Tree safe, uncommitted.
`wat-scripts/queue/sqs.wat` + two scratch-pad probes.
Retry kept. The milliseconds came back.

```
Summary [ 385.336s] 5216 tests run: 5216 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T05-12-17Z/`

## THE MECHANISM

**A match pays for unused arms.** Their AST is walked on every evaluation,
including the arm the happy path never takes.

The retry stone put `nap` / `once-put` (large nested matches) in a `let` on
every send, then moved the success continuation out of
`PutResponse::Success`. The DESIGN narrowed that to "body in a let, not a
match arm." The closures-as-allocations hypothesis was already dead (~2 µs).
The remaining fact, measured:

- A **large unused sibling arm** (retry closures, or a duplicated success
  body) sitting next to `Success` costs hundreds of milliseconds on this
  circuit, even when that arm is never taken.
- Named 6-arm match with **tiny** unused arms is free.
- Two tiny closures, even with a once-put-shaped body, are ~5 µs in a
  micro-probe — they are not 117 µs. The circuit cost is the **match form**,
  not construction on the happy path.

Ack got the same restructure. Drain did not move because acks run during
publish (cap 64). The unused-arm cost on ack is in `publish`, which is why
"ack costs nothing" was an incomplete control.

This is an interpreter fact. Recovered in `sqs.wat` without editing `src/`.
STOP-3 did not fire: the mechanism lives in `eval_match`, the workaround
does not.

## THE A/B — before any edit

`ps` before: grok 10.7 %, claude 6.0 %, else < 1 %. `wat --mcp` only.
Harness as the BRIEF wrote it, `sqs.wat` swapped in place.

```
without 853704d24~1   publish 23600 23625 23819    median 23625
with    HEAD retry    publish 24444 24475 24397    median 24444
```

**+819 ms, all in `publish`.** Drain ~200 ms both arms. Stop-1 did not fire
(roughly 900; variance ±150). `total=8000;distinct=8000;dup=0` both.

## VARIANT TABLE — one variable, 3 runs, probe each

Treat <300 ms as noise.

| variant | what moved | publish median | probe |
|---|---|---|---|
| A/B without | — | **23625** | n/a (no retry) |
| A/B with | retry as shipped | **24444** | RETRY=Ok (committed) |
| V1 | send: closures only in Transient | 24241 | RETRY=Ok; EXHAUST/CONSTRAINT/FATAL named; ACK=Ok |
| V2 | + success body in `if` after the match | 24425 | same Ok/named |
| V3 | old Success-inline + named arms, **no retry** | **23665** | RETRY=Lost `store put Transient` (expected) |
| V4 | Success-inline + **duplicated body in Transient** | **25362** | RETRY=Ok — unused large arm, *worse* |
| V5 | Success-inline, Transient still holds `once-put` AST | 24450 | RETRY=Ok |
| V6 | send Transient → tiny calls; ack Transient still large | 24157 | RETRY=Ok |
| V7 | **both** Transient arms tiny; retry/body in defns | **23740** | RETRY=Ok; EXHAUST/CONSTRAINT/FATAL named; ACK=Ok |

V3 vs without: +40 ms (noise). Named arms are free.
V4 vs V3: **+1697 ms** for a unused Transient arm that contains the success
body. That is the measurement that names the mechanism.
V7 vs without: +115 ms (noise). Retry kept.

## THE RECOVERY

Happy-path `Success` still holds the continuation inline (V3's fast shape).
`:Transient` is three lines: call `retry-put` / `retry-delete`, then
`send-after-put` / `ack-after-delete`. The large AST lives in module-level
defns, constructed once at load, not in a sibling match arm.

Probe (exact):

```
RETRY=send=Ok;total=10;distinct=10
EXHAUST=send=Lost:disconnected
  stderr: queue.send: store put transient, exhausted after 3
CONSTRAINT=send=Lost:disconnected
  stderr: queue.send: store put Constraint
FATAL=send=Lost:disconnected
  stderr: queue.send: store put Fatal
ACK=got=10;ack=Ok
```

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ⛔ A/B before any edit | ✅ +819 ms, 23625 vs 24444 |
| 2 | ★★ mechanism named, with the measurement | ✅ unused match-arm AST; V4 +1697 ms; V3 named arms free |
| 3 | ⛔ behaviour unchanged | ✅ probe as above |
| 4 | ⛔ every variant behaviour-checked | ✅ table |
| 5 | ⛔ retry not traded | ✅ a1/a2/a3 budget, named deaths |
| 6 | ⛔ no `_` on a store response | ✅ remaining `_` are ConnectOutcome / RecvOutcome |
| 7 | ⛔ floor | ✅ `Summary [ 385.336s] 5216 tests run: 5216 passed (6 slow), 22 skipped` |
| 8 | ⛔ delivery exact | ✅ `total=8000;distinct=8000;dup=0` |
| 9 | blast radius | ✅ `sqs.wat` + scratch-pad probes + this SCORE |

### ▪ reports

**a.** publish median after recovery: **23782** (×5: 23729 23752 23782 23806 23884).
publish+drain median **23958** vs A/B-without ~23842 (noise) vs A/B-with ~24661.

**b.** variant table above.

**c.** dup=0 every circuit run.

**d.** new probes:
`probe-what-a-large-fn-costs.wat` — large-body fn 5.3 µs/op vs tiny 2.2 µs;
let-vs-if continuation 0.47 µs. Closures and let-vs-if are not 117 µs.
`probe-what-a-six-arm-match-costs.wat` — 6-arm vs 2-arm **−40 ns/op**. Named
arms are not the cost.

## MICRO-PROBES (the ones that did not find 117 µs — that is the point)

```
tiny-us=17903; large-body-us=42325; large-type-us=14734
tiny-ns/op=2237; large-body-ns/op=5290; let-minus-if-ns/op=468
two-ns/op=1336; six-ns/op=1295; six-minus-two-ns/op=-40
```

Committed closure probes still ~1.6–2.4 µs. They measured the wrong
closures (tiny bodies, not sitting in an unused match arm).

## STOP-1 / STOP-2 / STOP-3 / STOP-4 / STOP-5

- **STOP-1** did not fire. A/B +819 ms.
- **STOP-2** did not fire. Retry kept.
- **STOP-3** did not fire as an exit. The mechanism *is* in the interpreter
  (`eval_match` over unused arm AST). Named, recovered in `sqs.wat`.
- **STOP-4** did not fire. Probe identical to the retry stone.
- **STOP-5** did not fire. `sqs.wat` + scratch-pad.

## NOT TOUCHED

`wat/`. `src/`. `SendResponse` / `AckResponse`. The lying k-of-n wrapper.

The success continuation is still duplicated (inline on `Success`, defn for
Transient). That is the price of keeping the taken arm's shape and the unused
arm tiny. Do not fold `Success` into the defn without measuring — V3's fast
path is the inline arm.

---

Tree uncommitted. Do not commit unless asked.
