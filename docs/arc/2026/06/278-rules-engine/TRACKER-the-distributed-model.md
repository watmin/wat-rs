# TRACKER — the distributed model, and the side quests

Opened 2026-09-02. **Living document — update in place, do not append a second copy.**
No arc number is minted here; this tracks work inside arc 278 until the builder rules otherwise.

## Baseline as of 2026-09-02 (measure against this, five runs, quiet box)

```
2000×4×3, mem-store, topic cap 16, queue cap 32, K=10
  publish+drain 8.3–8.7 s   →  921–954 deliveries/s   (±1.8%)
  e2e max 152–197 ms        →  t3→t4 >1 s count = 0
  total=8000; distinct=8000; dup=0
  wall 22.6 s, of which ~12.5 s is process spawn/reap (builder's boot-time item, NOT this arc)
```

Same fixture this morning: **109 deliveries/s, e2e ~12 s, non-deterministic.**

---

## THE MAIN LINE

### 1. Prove redelivery works — ✅ DONE 2026-09-02
`probe-visibility-redelivers.wat`, gated by `tests/services/probe_queue_visibility.rs`:
`first=got;while-inflight=none;after-expiry=got;same=yes`.

### 1-historical. Why it needed proving — NOT STARTED, probe, small
`sqs.wat:62` states the intent (*"the message stays invisible until its visibility timeout"*) but
**nothing exercises redelivery after visibility expiry**, and the circuit sets `visibility-ns` to
10¹² ns precisely so it never happens.

Everything below assumes retry-on-no-ack works. **Establish it before building on it.**

### 2. Topic durability via its own queue-service — ✅ STRUCK 2026-09-03
Costs **6.1× throughput** (921–954/s → 149–161/s, e2e 152–197 ms → ~700 ms), and ~3.5× of that
survives a linear store. See `FINDING-durability-is-store-op-bound.md`.

### 2-historical. The gap it closed — NOT DRAWN, stone
`publish` returns `Ok` for a message held in **`:ephemeral` state** (`sns-fanout.wat`, `outbox`).
The queue puts to a store before replying `Ok`; the topic does not. The two services disagree about
what acceptance means and only one is honest.

The shape, composed entirely from existing parts:

```
topic-service = publish surface + ONE queue-service instance + J internal workers
```

- `publish` writes **N rows, one per subscription**, to the internal queue, then replies `Ok`
- internal workers consume it, call `Queue/send` on the subscriber queue, **ack on success**
- subscriber `Full` → **don't ack** → visibility expires → retried

Per-subscription tracking is the point: SNS is **at-least-once per subscription**, not best-effort —
retry policies and DLQs attach per subscription. The `(message, subscriber)` row is the unit; a
message-level unit would re-deliver to everyone on one subscriber's retry.

★ **Do not write a second delivery engine.** A durable topic with background retry *is* a queue with
subscribers as consumers, and the queue already has level-triggered wakeup, a depth bound, parked
waiters and batching.

### 3. Packet loss injection — IN PROGRESS, and it split into four
The reactor is `wat/service.wat` — **wat, not Rust** — so the drop lives where the serve loop
sends, and its *placement in the loop* is what decides the fault:

| drop lands | work happened? | caller knows? | duplicate on retry? |
|---|---|---|---|
| before dispatch | no | no | no |
| **after the arm, before the reply-send** | **yes** | **no** | **YES** ← the acceptance criterion |

- **3a. `:wat::rand::`** — ✅ STRUCK 2026-09-03. Two verbs, named apart, classified apart.
- **3b. Bound every wait, and report** — ⛔ **SUPERSEDED 2026-09-03, and it was aimed at the wrong
  rung.** Bounding the waits is rung 2. The root is `:wat::time::Nanosecond 0` and `wait-ns 0` — a
  MODE SPELLED AS A MAGNITUDE, whose identity element silently means "don't". Now four stones:

  | stone | what | state |
  |---|---|---|
  | **A** | `:wat::time::NonZeroDuration` (`NonZeroU64`). Zero-as-a-wait has no form | ✅ **STRUCK 2026-09-03** |
  | **B-pre** | time types cross a service boundary (`edn_to_typed_value_inner` + `decode_declared_field`) | ✅ **STRUCK 2026-09-03** |
  | **B** | queue `wait-ns i64` → `wait <- Queue::Wait`, `:Immediate` / `:UpTo [NonZeroDuration]` | ✅ **STRUCK 2026-09-03** |
  | **D** | ✅ **STRUCK** — the helper vocabulary that hung the floor — `take-one`, `wait-pending`/`wait-inflight`, `q-depth`'s `(Tuple 1 1)`, `accept!`, the lying comment at `sns-fanout.wat:145`, the `1`-vs-`-1` sentinels, **and `pending`/`in-flight` → `visible`/`unacked`** | **NEXT.** Owns the live race |
  | **C** | ✅ **STRUCK 2026-09-04** — the naming sweep — `Alarm :after`→`:delay` (64 sites/25 files, **stdlib, BOOTSTRAP**), `Millisecond`→`Milliseconds` (56 sites) | LAST. Closes no defect |

  ### ⛔ THE ORDER, RULED BY THE BUILDER 2026-09-03 — D → chaos (3c/3d) → C

  **Not C then D.** Written down because it is counter-intuitive (C is mechanical, D needs design)
  and because a later self will be tempted to sweep the easy renames first:

  1. **D closes a live defect; C closes none.** The race is proven and deterministic. It has passed
     three times since the timeout — **that is the coin landing the other way, not a fix.**
  2. **★ THE CHAOS WORK CANNOT BE READ UNTIL D LANDS.** 3c/3d inject dropped replies. A dropped
     reply **plus** an unbounded spin in `wait-pending` is another unfalsifiable hang — the exact
     failure mode that made the original red produce an EMPTY ARM. Injecting faults into a system
     whose helpers convert timing misses into silent stalls means the results cannot be trusted in
     either direction. This is the decisive argument.
  3. **C would churn the files D must edit.** `sns-fanout.wat` and `circuit.wat` carry both the
     `Alarm :after` sites and the lying helper names. C-then-D rebases D onto a freshly-churned
     corpus for nothing.
  4. **D's shape is already proven in a committed probe** — `probe-refused-retry-self-consumes.wat`,
     `gap=300 → delivered; raced=yes-and-VISIBLE`, and its presence-wait already uses
     `:wait (Wait::UpTo …)` since Stone B. Same position B was in before its strike.

### ⛔ WHERE CHAOS ACTUALLY STANDS — 2026-09-04

**The order (D → chaos → C) is COMPLETE.** What follows is the honest ledger, because "is this done"
was asked and the answer was not obvious from the stone list.

#### ✅ DONE AND ON THE FLOOR — random failure with graceful recovery, CLIENT side

| what | evidence |
|---|---|
| seeded, self-arming, rate-gated severs | `-disrupt`; **24 severs/run, identical across five runs** — the seed replaying at 8000 msgs / 12 workers |
| the client redials and continues | 17 fatal `Closed` arms converted; the dead-peer wall still fires when redial truly fails |
| the invariant survives | `total=8000; distinct=8000; dup=0` ×5, all twelve workers finish |
| off by default | rate 0 arms **no alarm at all**; floor runs `disrupts=0` |
| a duplicate produced AND absorbed | `redelivery_is_absorbed_by_the_consumer`, `seen-dups > 0` — **but from visibility expiry, not from chaos** |

#### ⛔ NOT DONE — three things, ONE cause

1. **Server-side handle killing** (the server's selectable vec).
2. **The server discarding a lost client** — the substrate does it (`service.wat:64`, *"a vanished
   waiter … is not an error — keep serving"*), but **we have never asserted it.** Inherited, not
   demonstrated.
3. **A duplicate arising FROM chaos** — `seen-dups=0` under 24 severs. Predicted with its mechanism
   and confirmed: *arms run to completion, so an alarm fires between them* and a client-side sever
   can never land mid-claim.

**All three live at the reply-send inside the serve loop.** That is the seam.

#### THE REMAINING STONES — updated 2026-09-05

| stone | what | state |
|---|---|---|
| **R1** | the seam — `send-keep-serving?` at `service.wat:3108`, five callers | ✅ **STRUCK** (3 drafts; v1 a wrong type, v2 a wrong proof) |
| **T1** | a client has a deadline — races reply vs timer, discards, redials, retries | ✅ **STRUCK.** `seen-dups=0` ×5 — the deadline does not fire in health, which is the row passing |
| **S40** | the goldens pin ABSOLUTE LINE NUMBERS to assert a span — **MEASURED 2026-09-05, and it does NOT block R2** | **open, but demoted** |
| **S41** | ⛔ **STALE AND WRONG — the deadline is 200 ms, not 5000 ms** (`circuit.wat:423` records the change). Its evidence was a dead service, not saturation | **struck from the order** |
| **R2** | the drop in the seam | ⛔ **NOT STRUCK.** Blocked by S40 |
| **3d** | the reply-drop | ⛔ **REFUTED** — no userland form. A reply is sent, or deferred; there is no "caller told nothing and carries on" |

★★ ⛔ **R2 WAS NEVER BLOCKED. IT WAS BROKEN BY ONE TOKEN — measured 2026-09-05.**

`circuit.wat:121` held `(:wat::core::None :fanout::Seen::Reply)` — **the exact phantom form arc 109's
`NOTE-none-is-not-a-function.md` documents**, in the one arm the drop reaches. The A/B, one token,
same fixture, same seed:

| `circuit.wat:121` | `r2_drop_before_tiny` |
|---|---|
| `(:wat::core::None :fanout::Seen::Reply)` | ⛔ **TIMEOUT 30 s, empty arm** |
| `:wat::core::None` (the NOTE's ruling) | ✅ **PASS 8 s** — `total=100;distinct=100;dup=0;seen-firsts=100;`**`seen-dups=5`** |

★★ **`seen-dups` MOVED UNDER CHAOS.** First time outside a deterministic gate — the number this whole
arc has been chasing. Careful about the path: drop-before never writes, so its own retry is a `First`;
the 5 dups arrive via **visibility expiry made reachable by the drop's delay**. Chaos-caused, not
chaos-retried. Rate-0 is byte-identical (`distinct=8000; dup=0; seen-dups=0`).

★ **And the phantom form borrowed its surroundings' meaning, exactly as the NOTE predicted.** Its
symptom — a 30 s timeout with an empty arm — was written up as *"each dropped claim costs a 5000 ms
deadline; 10 % of 8000 is saturation"* and minted as S41. The deadline has been **200 ms** since D2
(`circuit.wat:423`, whose comment records the 5000 ms → 200 ms change *and why*), and the retry is
bounded to **3 attempts**. Neither number can produce a 30 s hang. **S41 was a measurement of a dead
service written up as a claim about backpressure.**

#### ⛔ WHAT IS STILL OPEN — and it is the predicted defect, not a blocker

`r2_drop_after_tiny` **still times out with the token fixed.** The mechanism is on the disk:

- `circuit.wat:491` — `outs1 (if first? (conj outs0 Outcome) outs0)`. **A `Dup` emits no outcome**, and
  the message is acked either way.
- drop-**after** writes the ledger, then drops the reply → the retry sees `already?` → `Dup` → **no
  outcome for that message, ever.**

That is `DESIGN-the-unknowable-state`'s prediction verbatim: *"Nobody emits an outcome for that
message… a stranding. The consumer claims before it emits."* The 2×2 now **discriminates on one
variable** — before completes, after hangs.

⚠ **What is NOT established: the number.** `distinct < total` has never been *observed*, because the
drain waits unbounded and the stranding presents as a hang. **The instrument fails exactly where the
defect appears** — the same class as the original red and as Stone D. **The stone in front of R2 is
therefore: bound the drain so a stranding reports a number instead of a timeout.** Not S40, not S41.

★ ⛔ **S40 WAS MEASURED AND THE RULING ABOVE IS WRONG — 2026-09-05.** I inserted one comment
line at `wat/service.wat:2` (above the guard at 896), rebuilt, and ran the cluster. Four tests red,
and **the entire diff was eight integers**: `896→897`, `903→904`, `913→914`, `921→922`, twice each.
Every other field — `reason`, `message`, the fixture spans — byte-identical. `UPDATE_EDN=1 cargo
nextest run --release -E 'test(peers_bijection)'` restored all five to green in **one command**, and
`git diff` on the goldens showed **only `:line` fields moved**. Reverted; binary rebuilt; 5/5 green.

**So the goldens are not an obstacle. They are an 8-integer re-capture.** "Blocked by S40" was
reasoned, never run — the third time this campaign that a stone was drawn from an unprobed premise.

★ **The real defect the probe found is wider than S40 was written.** The census: **10 golden files**
pin an absolute line inside a *stdlib* `wat/*.wat` — `service.wat` (896, 913), `core.wat` (1412,
1464, 1919, 1947), `Record.wat` (145). An edit near the top of any of them reds goldens in proportion
to how early you edit, and **the redness carries no information** — the `reason` string is already
asserted literally in the `.rs`, so the pinned line discriminates nothing the test does not already
check.

★ **And the danger is the cure, not the disease.** `UPDATE_EDN=1` writes the emission VERBATIM. A
blind bless after a change that *also* moved the diagnostic text would swallow the regression and go
green while asserting something new. The safety property is **read the diff and confirm only
`:line`/`:col` moved** — which is rung 1, a convention, exactly where `probe_arc278_peers_bijection.rs`
already keeps one hand-written `contains("probe::Echo")` assertion because a capture cannot be
trusted alone. Rung 3 — normalize spans for `wat/*.wat` at capture — would delete the class AND
remove every occasion to invoke the blind bless.

★ **R2's blocker, precisely.** `state` **is** in scope at all five send sites, so a drop is feedable
from `:durable`. But durable fields are **per-service**, so a generic `drop?` hits `stats` as well as
`claim` — and adding a field to every Record sits **before line 896**, tripping the bijection
goldens. **The goldens went from instrument (R1 v3's proof that nothing shifted) to obstacle.**

★ **And the table still has an unmeasured half.** Drop-before at 8000 hung publish —
`never-accepted; depth=744; cap=64; elapsed=60000`. Each dropped claim costs a 5000 ms deadline plus
a retry; 10 % of 8000 is saturation, not chaos. A tiny `n=12` run gave `seen-dups=0` for **both**
placements and was correctly **not** read as a result — too small to force hits.

### ⛔ THE LIVE RACE — the floor is GREEN and that is the dangerous part

**Floor 5213/5213 at `.floor/2026-09-03T22-45-39Z/` (my run).** The test below has now **passed
three times** since its one timeout. ⛔ **It is not fixed.** The mechanism is untouched and
reproducible on demand at `wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat`, 3/3:

```
gap=0    recovered-after-naps=0    would-return
gap=300  recovered-after-naps=-1   SPINS-FOREVER
```

`take-one` destructively consumes the message `wait-pending` then waits for. **Stone D is the
disposition. A green floor is not.** Do not re-run it to a green and call it closed — a green run is
the coin landing the other way.

★ **A race whose failure mode is an unfalsifiable hang is worse than one that asserts:** the losing
run leaves no evidence and every winning run reads like a fix. Three greens in a row is exactly what
that looks like from the inside.

#### The original red, kept verbatim
`.floor/2026-09-03T09-14-58Z/` — `5199 passed, 1 timed out`.

```
TIMEOUT [ 30.015s] probe_async_publish::refused_subscriber_is_retried_not_dropped
stdout: running 1 test / (test timed out)
```

**Not disposed.** Established: the rand stone changed no `.wat`; the probe uses `:vis-ns 200000000`
(the 200 ms window deliberately kept on the refusal probe); `:demo::wait-inflight` and
`:demo::wait-pending` are **unbounded** (nap 1 ms, recurse); alone it runs 1336/1366/1402 ms.
**Not established:** whether load stretched it >20× or the retry never fired — and that is the
defect, not the timing.

★ **The class: an unbounded wait in a floor-driven test converts a timing miss into an
unfalsifiable hang.** The ARM is empty because there is nothing to print. Same family as a
truncating pager and a piped exit code — all three destroy the evidence that would name the failure.
**3b was superseded** (see the four stones above): bounding the waits is rung 2, and the root was a
mode spelled as a magnitude. **Stone D carries what remains** — the destructive-read-as-absence-check
that is the actual mechanism here.

### 3-historical. Packet loss injection — the original entry
The one fault domain never simulated. IPC has been giving us reliable networking for free.

★ **The acceptance criterion is that it BREAKS something:** `dup=0` is a property of the transport,
not of the design. At-least-once under loss produces duplicates by definition — a lost ack means
redelivery. So the honest invariant becomes `distinct=8000; dup ≥ 0`, and the choice is idempotent
consumers or store-side dedup by message id.

**If loss is injected and `dup` stays 0, the loss is not being modelled.**

---

## SIDE QUESTS — found in passing, none chased

| # | what | status |
|---|---|---|
| S1 | ~~Adapter's 1 ms retry poll~~ **OBSOLETE — `:fanout::adapter` was deleted by the durable-topic stone.** No adapter, no poll. | dissolved |
| S14 | **The topic-worker sends and acks ONE ROW AT A TIME** — `bodies` is a `Vector` used with a single element, and one `Queue/ack` per row. The batching surface survived the adapter's deletion; its use did not. | ready |
| S1-old | ~~(superseded)~~ Its trigger has fired (bounding every stage worked). Same repair already made twice; short. | ready |
| S2 | **Store swap to sqlite in the circuit.** Measured 1.29× at cap 16, 1.75× batched. The codemod exists (`fix-circuit-to-sqlite.wat`), is idempotent, diff verified store-only. | ready |
| S2b | **The store is now 1.68×**, up from 1.29× before durability — the durable topic made it hotter exactly as predicted. Promotes S2 from convenience to load-bearing. | ready |
| S13 | **The circuit asserts exactly-once on an at-least-once system.** `probe_ex001_fanout` requires `total == distinct`; a visibility expiry during processing produces a legitimate duplicate and reds the floor (it did, 26 vs 24). Widening the window 200 ms → 5 s is correct SQS configuration and **does not remove the class** — the assertion still depends on a timing margin. Resolution is an idempotent consumer (dedupe by envelope id), and **item 3 forces it anyway**. | open |
| S3 | **`mem-store` writes are O(table).** 1000/2000/4000 rows → 6.5/20.8/90.0 s. At the current workload it is a **1.29× perf term**; it is also the differential **oracle**, so an O(n²) oracle slows every differential as the corpus grows. `SCORE-perf-3` claimed "writes go linear" and is corrected in place. | open |
| S4 | **`:wat::core::Vector`'s `conj` is O(n)**, `PersistentVector`'s is O(1) — 17× at n=4000, **no stated complexity contract anywhere**, and the name points the wrong way. **23 files** accumulate into a `Vector` via `conj` in a `foldl`. Already fixed once at one call site (`stream->vec`) and **regrew** in the topic's outbox. Rides the builder's corpus migration; wants a contract and possibly a lint, not just a migration. | builder's |
| S5 | **Substrate `ensure-alarm` outcome.** The level-triggered wakeup uses a hand-maintained `armed?` flag; an outcome meaning *"ensure an alarm exists for op X"* makes both armed-twice and armed-zero-times unrepresentable. Rung 3 for a class currently at rung 2. Needs `wat/`. | open |
| S6 | **Nonexistent stdlib enum variants are not resolve-checked.** `:wat::core::Option::Nope`, `:wat::kernel::RecvOutcome::Bogus` all pass `--check` and die at run time; a same-file enum is rejected. Cost real time twice today. | open |
| S7 | **Duration-0 `after` never fires at process tier.** Verified; thread fires, process is silent, no diagnostic. Locus transparency break. Untouched. | open |
| S8 | ~~The outbox cursor~~ **OBSOLETE — the topic's outbox was deleted by the durable-topic stone.** | dissolved |
| S8-old | ~~(superseded)~~ The rebuild was ~0.47 ms/delivery; K=10 tick-batching amortises it. **Its share needs re-measuring** before it is ruled on again — the last two rulings both expired for reasons I mis-dated. | re-measure |
| S9 | **`body-key` is dead code** in `circuit.wat` and the sqlite variant — defined, never called. So "the same body delivered twice" is checked by nothing. | trivial |
| S10 | **CORRECTED and WORSE.** Alive at `sqs.wat:253` (`contains? b`, which is why a `contains? body` grep missed it — token, not form). The queue appends a stamp; the topic-worker now **splits and re-joins the body per message to strip it back off**. Production code adds a field that other production code removes. | ready |
| S10-old | ~~(superseded)~~ — production code branching on payload content for instrumentation. Least-bad given the surface was frozen, but a wart. | wart |
| S11 | **Promotion ruling:** `wat-scripts/{topic,queue}` → `wat/`. | builder's |
| S12 | **The capstone** — telemetry instrumenting the circuit. Still unbuilt; the in-band trace has partly superseded its purpose. Worth re-scoping rather than building as originally drawn. | re-scope |
| S15 | **A zero-duration timer WEDGES TEARDOWN.** `after(process, Nanosecond 0)` returns `Closed` and the program cannot exit — `EXIT=124`, 3/3. Its control, identical but for that one cell, exits 0. Repro committed: `scratch-pad/probe-zero-duration-disarms-at-process.wat` + `-control.wat`. Stone A makes it unreachable **from wat**; `Value::Duration(0)` built in Rust still reaches `timerfd_settime`. | open, substrate |
| S16 | **The SCORE diverges from today's runtime.** `SCORE-the-sane-circuit.md:43` recorded `process ns=0 -> TIMED-OUT (500 ms guard)` on 2026-09-01. On 2026-09-03 the recv returns `Closed` promptly. Both cannot describe one behaviour. **The 09-01 line is left standing, not overwritten**, until someone establishes whether the runtime changed or the guard shape saw a different thing. | open |
| S17 | **`Duration`'s own storage is still `i64` and still a caller contract.** `value.rs:291-294` says direct Rust construction (`Value::Duration(-n)`) bypasses the guard. Stone A does **NOT** close this — an earlier DESIGN draft claimed it did, which was false. Note the deferred fix as named (`u64`) would not have caught zero either. | open |
| S18 | **A zero-duration timer manufactures a spurious `Closed`** — the substrate's word for "the peer went away", indistinguishable from a severed connection. Feeds the open `Closed`-after-sever item, which treats `Closed` as fatal-and-real. | open |
| S19 | **`:wat::time::+` cannot add two durations.** Both `+` and `-` require an `Instant` on the left, so `(+ (Hour 1) (Minute 30))` has no form and no substitute verb exists in the 41-row surface. | open |
| S20 | **`wait` has five senses in the tree**, incl. `src/process/handle.rs:98` `wait_or_cached_exit`, where one path blocks in `waitid` and the other burns CPU in `std::hint::spin_loop()`. The caller cannot tell from the name which they get. | open |

---

## METHOD RULES EARNED TODAY — these cost stones

1. **A perf row needs a distribution, not a sample.** Five runs minimum. Re-running a *timing*
   measurement is not the flake re-run the floor doctrine forbids — that rule protects **red
   assertions**, where a green re-run destroys evidence. For a timing row the spread *is* the evidence.
2. **A perf finding measured under one regime does not transfer when the regime changes.** The store
   was correctly ~1% with shallow queues and 5.6× with deep ones. I closed that lane and had to
   reopen it two stones later.
3. **Check `ps` before any timing measurement.** A contended box produced a false reading.
4. **`assert s.count(old)==1` on scripted edits catches false MEASUREMENTS, not just bad rewrites.**
   It stopped "50 ms fixes the variance" from shipping off three lucky samples.
5. **Three samples of a bimodal distribution look unimodal.**
6. **A 2×2 with one variable uncontrolled is not a 2×2**, and its numbers are worse than none.
