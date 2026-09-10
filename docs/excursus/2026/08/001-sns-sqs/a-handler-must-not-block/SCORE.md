# SCORE — a handler must not block

**SCORED as STOP-1.** Executor: claude, 2026-09-10, branch `sns-sqs`, HEAD `5fc703e56`. Did not commit.

**⛔ STOP-1 FIRED, and STOP-2 with it — they are the same fact from two sides, exactly as the stone
predicted.** The substrate affords the **outbound** half and not the **inbound** half. `:fanout::worker`
was **not** rewritten and **not one functional file was touched**: no `wat-scripts/fanout/circuit.wat`,
no `wat/service.wat`, no `src/`. Two files exist that did not before — one probe and this SCORE.

```
     Summary [ 483.739s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-10T08-10-54Z/` — `scripts/floor.sh` exit `0`, **no `ARM.txt`**, zero
`FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` in `clean.log`. That is the floor on the **final** tree, after
the timing runs. `5237` unchanged — no test added, removed or renamed.

⚠ **Stated plainly: the floor was run twice, both green, and neither run was a re-run after a red.** The
first (`.floor/2026-09-10T07-58-28Z/`, `Summary [ 485.349s] 5237 tests run: 5237 passed (7 slow), 22
skipped`) preceded two comment-only edits to the probe; the second is on the tree as it now stands, so the
Summary line above describes exactly what is on disk. **No red was seen at any point, so nothing was
re-run in the sense the rule forbids.**

---

## ⭑ THE ANSWER, IN ONE LINE

**A handler CAN issue a request to a declared peer and return. The reply CANNOT arrive as one of the
service's own ops.** The send half is a one-word change (`call-by-deadline` → `send`); the receive half
does not exist and cannot be authored.

The probe: **`wat-scripts/scratch-pad/probe-handler-issues-request-and-returns.wat`** — green, stable
across 3 runs, printing

```
"handler-send=Sent selectables-readable=yes reply-arrived-as-op=NO code=1107"
```

`1107` = `SendOutcome::Sent`(1)·1000 + `selectables` length(1)·100 + the reply(**7**) — and the 7 was
**fetched by a blocking `recv` in a later handler**, not delivered.

### The `file:line` that settles it

**`wat/service.wat:1566`** (with **`wat/service.wat:1594`**) — the select set's element type:

```
selectable-peer-ty `(:wat::kernel::Peer :- [~proto-reply-ty-ann ~service-op-ty-ann])
selectable-entry-ty `(:wat::core::Tuple :- [:wat::core::i64 ~selectable-peer-ty])
```

`Peer`'s arg order is `<S,R>` — **send-type first** (`src/types.rs:2279`; a timer is the
uninhabited-send `(Peer :- [Never O])`, `src/types.rs:6377`). So every member of `selectables` is a peer
this service **sends its own Reply on** and **receives its own Op from**: an accepted inbound client, or
an alarm timer. A peer *dialed at* another service is `(Peer :- [<their>::Op <their>::Reply])` — **wrong
in both positions.**

That is not a reading. It was put to the checker **against the real captured `selectables` of a real
generated serve loop**, and refused, verbatim:

```
#wat.check/TypeMismatch {:message ":wat::core::conj: parameter #2 expects
  :(wat::core::i64,(wat::kernel::Peer :- [probe::Asker::Reply probe::asker::Op]));
  got :(wat::core::i64,(wat::kernel::Peer :- [probe::Store::Op probe::Store::Reply]))"
  :callee ":wat::core::conj" :param "#2"}
```

---

## ★★ ROW 12 — THE BUILDER'S READING WAS WRONG, AND IT WAS WORTH FINDING OUT

> "`wat/service.wat:1564` types a selectable as `(Peer :- [proto::Reply service::Op])` … So the
> **inbound** half — a peer reply arriving as a service op — appears to exist."

**It does not.** `(Peer :- [proto::Reply service::Op])` is send=Reply, recv=Op — the **server** end of an
accepted connection. It says "clients speak to me"; it does not say "a peer's reply reaches me". And the
re-tag at `:1542` is the *other* direction: `retag-op` at **`wat/service.wat:2391`** takes a **client's
inbound surface op** and widens it into the `<service>::Op` superset. The superset is synthesized from
exactly two sources — the surface's public ops and the service's own leading-dash internal ops
(`wat/service.wat:1539-1543`). **A peer's Reply has no variant to be re-tagged INTO.**

### But the sub-question — "can an author reach `selectables`?" — has a surprising answer: **partly YES, and it does not help**

⚠ **`selectables` is lexically in scope inside a handler body.** Not by design — by unhygienic macro
capture. Op bodies are spliced **inline** into the serve arm inside a `let` (`wat/service.wat:1834`
builds the binding vector; **`wat/service.wat:1813`** says it outright: *"`selectables`/`idx` are bare —
literal identifiers in the GENERATED code"*). `(:wat::core::length selectables)` inside a handler
**type-checks and runs** — that is the `selectables-readable=yes` and the `·100` term in the probe's
`1107`.

It is still useless, for two independent reasons:

1. **Read-only.** The loop recurses on **its own** binding —
   `(~serve-name self l (:wat::core::foldl ~arm-fn selectables arms) …)` at **`wat/service.wat:2074`** —
   so a vector a handler rebuilds is discarded. A handler's only return channel is an `Outcome`.
2. **Wrongly typed.** The refusal above.

And it is grown at exactly **two** internal sites: `ServiceEvent::Connection` (an accepted inbound
client, **`wat/service.wat:2308`**) and `arm-fn` folding `Alarm`s into `after` timers
(**`wat/service.wat:1885`**). There is no `:selectables` clause (recognized clause list,
**`wat/service.wat:410`**), and **`:peers`** (**`wat/service.wat:811-829`**) is a declaration/manifest
clause — a dependency DAG plus a bijection check against ephemeral peer fields — that wires **nothing**
into the select set.

**So: my DESIGN's framing costs you a DESIGN, and the real shape is bigger than drawn.** Reported, as
row 12 asked.

---

## ⭑⭑ THE MISSING FORM, NAMED — and why `state` / `reply` / `sends` / `arms` cannot carry it

### The gap in one sentence

**The queue's push already works because the requester is one of the queue's `selectables` — an accepted
connection with a `conn-id` (`:queue::Waiter` holds `conn-id`, not a peer: `wat-scripts/queue/sqs.wat:84`;
the push is `(:wat::service::Directed :conn-id (:queue::Waiter/conn-id w) :reply …)` at
`wat-scripts/queue/sqs.wat:629` and `:650`). The worker's side has no mirror: the *client* end of that
same socket cannot be one of the worker's `selectables`.** One socket, in one service's select set,
structurally excluded from the other's. That is the whole defect, and the stone's note that the queue is
"already push" is exactly right — which is why the fix looked small and is not.

### Three independent gaps, not one

| # | gap | where it is fixed today |
|---|---|---|
| **G1** | **no way to ADDRESS an outbound peer from an outcome** | `Directed :- [R]` is `[conn-id <- i64, reply <- R]` (`wat/service.wat:71-73`). `conn-id` is minted by the loop for an **accepted inbound** connection (`wat/service.wat:2308`); a dialed peer never gets one and an author cannot mint one. `Alarm :- [O]` is `[delay, op]` (`wat/service.wat:67`) — addresses only the service itself. |
| **G2** | **no way to TYPE an outbound request as a payload** | `Directed`'s payload is `R` = the service's own Reply union, fixed by `:satisfies`. A request to a peer is a `<their>::Op`. No variant of `R` is one. |
| **G3** | **no way for the REPLY to come back as an op** | the type refusal above, plus `poll`/`select` are Rust intrinsics over a **homogeneous** `(Vector :- [(Peer :- [I O])])` (`infer_select_prime`, `src/check.rs:12305`) — there is no heterogeneous set to join even in principle — plus the `<service>::Op` superset has no peer-Reply source (`wat/service.wat:1539-1543`). |

### Why each existing field cannot carry it

- **`state`** — it already **holds** the peer (`:ephemeral [q <- (Peer :- [Queue::Op Queue::Reply])]`,
  `wat-scripts/fanout/circuit.wat:289`). But the serve loop only **threads** `state`; it is opaque to it
  (its type is the service's own `State`). Nothing in `poll`'s argument is derived from `state`, so a peer
  stored there is invisible to the multiplexer. `state` is where the peer is and can never be how the loop
  learns of it.
- **`reply`** — `(Option :- [R])`, one reply to the current invoker. Wrong direction, wrong type.
- **`sends`** — G1 and G2 above, together. It is a **reply** channel to parked requesters, by name and by
  type; the doctrine is explicit that an arm must not hold a caller's Peer (`wat/service.wat:60-62`).
- **`arms`** — `Alarm` = delay + the service's **own** `O`. `after` mints a `(Peer :- [Never O])` timer,
  which is a clock, not a connection to anything. A handler can schedule its own future; it cannot
  schedule someone else's answer.

### The form I would need (named, not built — opening the arc is the builder's ruling)

A **fifth** `Outcome::Continue` field, and it needs **three** substrate additions behind it, one of which
reaches `src/`:

```
:Continue [state, reply, sends, arms, asks <- (Vector :- [(Ask :- [O])])]
```

1. **A third source for the `<service>::Op` superset.** For each `:peers` surface `S`, a dash-prefixed
   re-tagged copy of every `S::Reply` variant — so a peer's reply **has** an op counterpart to arrive as.
   ★ The rewrite machinery already exists and is already type-loose at exactly this seam: `infer_retag_op`
   (`src/check.rs:12296-12303`) says *"op: inferred for error coverage; its type is not further
   constrained"* and returns arg[2] as the result type. `eval_retag_op` (`src/runtime.rs:16059`) does the
   runtime tag rewrite. This is the cheap third of the change.
2. **A select set that admits a dialed peer.** Either `selectables` becomes a sum (accepted-client |
   ask-peer) with the retag target carried per entry, or `poll` gains a second homogeneous vector for
   ask-peers. **This is the part that is in `src/`** — `poll` and `select` downcast every Vector element to
   a Peer opaque and are typed homogeneously (`infer_select_prime`, `src/check.rs:12305`).
3. **An honest name for an outbound peer.** `conn-id`'s mirror. The peer already lives in `:ephemeral`, so
   a `:peers`-scoped id minted at `:init` (or the surface keyword itself) would address it without an arm
   ever holding a Peer — preserving the rule at `wat/service.wat:60-62`.

⚠ **Scope, stated plainly: this is a `wat/service.wat` + `src/` change, i.e. STOP-2.** I stopped before the
first edit in either.

### ★ The one shape that IS available today — and why it is a different stone, not this one

Invert to **push**: give `:fanout::Worker` a `deliver` op, have `:queue::queue` hold a peer per subscriber
and issue `(:wat::kernel::send w (Worker::Op::Deliver envs))` — fire-and-forget, non-blocking on the queue
side (proven by the probe's `poke`), with the worker receiving deliveries as **ordinary inbound ops**, no
blocking receive anywhere. The worker's `deliver` returns `reply = None` so nothing strands.

It is genuinely expressible. It is **not** this stone:

- it needs a new `Queue` op to register subscribers, and `:queue::Queue` is in
  **`wat-scripts/queue/sqs.wat:38`** — outside the drawn radius ("`circuit.wat` should be the only file
  that changes");
- it moves visibility timeout, redelivery and at-least-once semantics from pull to push — a protocol
  redesign, and the before/after comparability the stone rests on ("the topology being identical") is gone;
- and it fixes **this one topology**, not the class. Every other service that asks a peer anything from a
  handler still goes deaf.

**That is the builder's call, not mine.**

---

## ⚠ ROW 13 — WHAT THIS IS ABOUT, SAID BEFORE ANY NUMBER

**The point is not the milliseconds.** The point is that **a service is deaf to every request while it
waits on a peer** — to `disrupts`, to `stop`, to health, to anything. `collect` is where the harness
happens to ask 12 workers two questions each, so `collect` is where the deafness shows up as a number.
Fixing `collect` is not the goal; `collect` falling would be a *symptom* that the goal was met.

**And it fixes one service, not the class.** `:fanout::worker` is one site. The finding is that the
*shape* — a blocking client call inside a handler — is available to, and unavoidable for, **every**
`defservice` that asks a peer anything. `:fanout::held-worker` at `wat-scripts/fanout/circuit.wat:943`
does it too, 5× faster and undocumented (named, untouched, per the trap-door).

---

## Rows

| # | row | verdict | evidence |
|---|---|---|---|
| 1 | the question is answered from the substrate, not opinion | **PASS** | probe `wat-scripts/scratch-pad/probe-handler-issues-request-and-returns.wat`, green ×3. **Outbound YES, inbound NO.** Settled by `wat/service.wat:1566` + `:1594` and the verbatim `conj` refusal against the real `selectables` |
| 2 | if NO, the missing form is named precisely | **PASS** | "THE MISSING FORM, NAMED" above — G1/G2/G3, one line per existing field, and the three additions with the `src/` one identified. **This row is what satisfies the stone** |
| 3 | if YES, the worker no longer blocks | **N/A — the answer was NO** | `wat-scripts/fanout/circuit.wat:471` unchanged. No `Wait::Immediate` substituted anywhere. `git status` shows no `circuit.wat` |
| 4 | correctness untouched | **PASS (baseline, and nothing was changed)** | `2000 4 3`: `distinct=8000 dup=0` ×3, inbox `accepted=2000` ×3. `20 4 3`: `distinct=80 dup=0` ×3, `accepted=20` ×3. No raise in any of the 6 |
| 5 | not a busy poll | **N/A** | nothing changed; `store-calls` baseline recorded below for the arc that lands the fix |
| 6 | the worker answers while working | **N/A — not achieved, and not achievable in radius** | `collect` unchanged. Baseline recorded |
| 7 | `drain` does not regress | **PASS trivially** | unchanged code; baseline `2437 ms` / `34 ms` |
| 8 | the stall gate still fires | **PASS** | `circuit.wat 5 1 0 32 false 0` → `rc=2`, `drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601 elapsed=12948` |
| 9 | the corpus loads | **PASS** | `PASS [ 483.732s] (5237/5237) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`, inside the final floor run — with the new probe in the tree. Also PASSed standalone (`245.470s`) when the probe was first added |
| 10 | the floor holds | **PASS** | `Summary [ 485.349s] 5237 tests run: 5237 passed (7 slow), 22 skipped` |
| 11 | blast radius | **PASS, tighter than drawn** | `git status --porcelain`: `?? wat-scripts/scratch-pad/probe-handler-issues-request-and-returns.wat` and this SCORE. **No `wat/`, no `src/`, and not even `circuit.wat`** |
| 12 | the inbound half really does exist | **★★ FAILS — and the failure is the builder's, as drawn** | section above. `selectables` is *readable* by unhygienic capture but read-only and wrongly typed; `:peers` wires nothing |
| 13 | the milliseconds are a consequence, not the point | **PASS** | stated above, before any number |

**Real edit-site count: 0.** Two files added (probe, SCORE); zero functional lines changed.

---

## The numbers — BASELINE ONLY. ⚠ There is no "after", because nothing was changed.

Box quiet (load average 0.07 at start), every run through `./scripts/capped.sh --limit 8g`, release binary
`target/release/wat` at HEAD `5fc703e56`. All values in ms.

### `circuit.wat 2000 4 3 8192 true 1000` — 3 runs

| phase | run 1 | run 2 | run 3 | mean | spread |
|---|---|---|---|---|---|
| `setup` | 12462 | 12397 | 12456 | 12438 | 65 |
| `fill` | 2609 | 2664 | 2625 | 2633 | 55 |
| `arm` | 17 | 45 | 17 | 26 | 28 |
| `drain` | 2433 | 2457 | 2420 | 2437 | 37 |
| **`collect`** | **5002** | **4188** | **4979** | **4723** | **814** |
| `stop` | 207 | 120 | 469 | 265 | 349 |
| `total` | 22734 | 21874 | 22968 | 22525 | 1094 |
| **`store-calls`** | **4643** | **4650** | **4631** | **4641** | **19** |

`distinct=8000 dup=0` and inbox `accepted=2000` in all three. `workers=11/10/12`, `empty=1`.

### `circuit.wat 20 4 3 8192 true 1000` — 3 runs (where the per-worker cost is cleanest)

| phase | run 1 | run 2 | run 3 | mean | spread |
|---|---|---|---|---|---|
| `setup` | 12436 | 12427 | 12453 | 12439 | 26 |
| `fill` | 138 | 67 | 67 | 91 | 71 |
| `arm` | 18 | 15 | 15 | 16 | 3 |
| `drain` | 37 | 31 | 33 | 34 | 6 |
| **`collect`** | **2961** | **4339** | **3184** | **3495** | **1378** |
| `stop` | 596 | 422 | 523 | 514 | 174 |
| `total` | 16188 | 17303 | 16277 | 16589 | 1115 |
| **`store-calls`** | **124** | **124** | **124** | **124** | **0** |

`distinct=80 dup=0`, `accepted=20`, `workers=8`, `empty=1` in all three.

⚠ **`collect`'s spread is the honest caveat on row 6's target.** Three earlier `20 4 3` runs (before the
canonical set, same box, same binary) gave `collect` = 3814 / 3778 / 4279. Across all six observations the
band is **2961–4339**, i.e. **±20 % around ~3.5 s**. The DESIGN's 50 ms row (~962) and 250 ms row (~4053)
are still separated by far more than that, so the mechanism is not in doubt — but **any future
"`collect` fell" claim needs ≥3 runs on both sides, because a single pair inside this band proves nothing.**

⚠ **`setup` is 12.4 s of the ~16.6 s `total` at `20 4 3`** — 75 % of the run is cold boot, ruled out of
scope by the DESIGN. Worth knowing before anyone reads `total` as a result.

---

## Changed outside the stated radius

**Nothing.** No `wat/`, no `src/`, no `wat-scripts/fanout/circuit.wat`, no `wat-scripts/queue/sqs.wat`, no
test, no config. Two files added:

- `wat-scripts/scratch-pad/probe-handler-issues-request-and-returns.wat` — the probe. Green; carries the
  verbatim checker refusal and every `file:line` in its header.
- `docs/excursus/2026/08/001-sns-sqs/a-handler-must-not-block/SCORE.md` — this file.

One transient RED probe (`RED-probe-peer-into-selectables.wat`) was written to obtain the verbatim
refusals and **deleted** — it cannot live under `wat-scripts/`, which the corpus gate type-checks whole.
Its output is quoted above and in the probe's header.

⚠ **One caveat on the kept probe.** It reads `selectables` from inside a handler, which works only because
of the unhygienic capture. **If a future hygiene fix reddens that line, that is correct behaviour: the
finding expired**, and the line is commented to say so.

Left **uncommitted**, as instructed.

---

# ⭑ THE ORCHESTRATOR'S GRADING — verified on my own reads

**Floor from `.floor/2026-09-10T08-10-54Z/clean.log`:**
`Summary [ 483.739s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 failure tokens, no `ARM.txt`.
**Blast radius:** two untracked files, **zero functional edits.** Row 11 ✅, STOP-2 honoured exactly.

## ⛔ ROW 12 FIRED. MY READING WAS EXACTLY BACKWARDS.

I built the DESIGN on `wat/service.wat:1566`'s `selectable-peer-ty (Peer :- [proto::Reply service::Op])`
and concluded *"a peer's reply already arrives as one of the service's own ops — half the mechanism
exists."*

Verified on my own read, `src/types.rs:2279`:

> *"Note the arg order `<S,R>` — connect's return is `Peer'<S,R>` (**send-type first**), the MIRROR of
> accept's `Peer'<R,S>`."*

**Send-type first.** So `(Peer :- [proto::Reply service::Op])` is *"a peer I send my Reply to and receive
my Op from"* — the **accept** side, a client connection this service serves. A peer obtained by
`connect` is `Peer<their::Op, their::Reply>`, **wrong in both positions.** The executor put it to the
checker against a real captured `selectables` and got the mismatch verbatim.

★★ **And the re-tag at `:1542` runs the other way too.** `retag-op` (`:2391`) widens a **client's inbound
op** into the superset, synthesized from exactly two sources — surface public ops and own leading-dash
internal ops (`:1539-1543`). A peer's Reply has **no variant to be re-tagged into.** Not half the
mechanism: none of it.

★ That is the eighth reading of mine to die on measurement today, and the row was written for exactly
this — *"I did not verify that an author can put a peer into that set."*

## ⭑ The answer, split, which is sharper than the question

**Outbound: YES.** `(:wat::kernel::send q op)` inside a handler is non-blocking and the handler returns.
**Inbound: NO.** The reply cannot come back as an op. So the blocking is not inherent to *asking* — it is
inherent to *hearing the answer*.

## ⭑⭑ The gap in one sentence, and it is the best line in this report

**The queue's push already works because the requester is one of the QUEUE's `selectables`** — an
accepted connection addressed by `conn-id` (`:queue::Waiter` holds a `conn-id`, not a peer, `sqs.wat:84`;
push at `:629`/`:650`). **The worker's side has no mirror: the client end of that same socket cannot be
one of the worker's `selectables`.**

Three independent gaps behind it, each cited: no way to **address** an outbound peer from an outcome
(`Directed` = conn-id + own `R`; `Alarm` = delay + own `O`), no way to **type** a foreign request as a
payload, and no **inbound route** (homogeneous `poll`/`select`, `infer_select_prime`,
`src/check.rs:12305`). The needed form — a fifth `asks` field, a third superset source, and a select set
admitting a dialed peer — reaches `src/`. **Hence STOP-2, correctly.**

## ★ One surprise worth keeping

**`selectables` is lexically in scope inside a handler body** — unhygienic capture, because op bodies are
spliced *inline* into the serve arm (`:1834` builds the binding vector; `:1813` notes `selectables`/`idx`
are "bare — literal identifiers in the GENERATED code"). `(:wat::core::length selectables)` type-checks
and runs. It is useless — read-only and wrongly typed — but it is a **macro-hygiene leak in the
substrate**, found incidentally, and nothing gates it.

## ⚠ A measurement caveat that lands on my earlier claims

`collect` at `20 4 3` spans **2961–4339** across six observations — **±20 %.** That is the executor's, and
it is right to raise it.

★ It does **not** overturn the mechanism: the wait sweep was 3 runs per cell and separated by **4.2×** and
**1.9×**, an order beyond this noise. It **does** retire any fine per-worker arithmetic — my
`726 + 338·workers` decomposition was already flagged under-determined at `1196720c3`, and this is the
number that explains why. **Any future "collect fell" claim needs ≥3 runs on both sides.**

## Grade

`1 ✅ (answered from the substrate, with the file:line) · 2 ✅ (the missing form named precisely) ·
3 n/a (NO branch) · 4 ✅ baseline · 5 ✅ (nothing changed) · 6 n/a · 7 ✅ · 8 ✅ (rc 2, drained-stalled) ·
9 ✅ · 10 ✅ · 11 ✅ zero functional edits · 12 ⛔ FIRED — my reading refuted · 13 ✅ honoured`

**STOP-1 and STOP-2 firing together is the whole value.** The stone cost me a DESIGN premise and bought
the exact shape of a substrate gap, cited to three files, with zero speculative code written.

## The ruling now owed to the builder

One shape **is** buildable today and the executor named it rather than building it: **invert to push** —
`kernel::send` from the queue into a `Worker/deliver` op with `reply None`. It needs a new `Queue` op
(`sqs.wat:38`, outside the drawn radius) and it **moves visibility and redelivery from pull to push.**

⚠ And that is not a neutral trade in this campaign: **SQS is a pull protocol, and this queue exists to
model it.** Inverting to push buys responsiveness by diverging from the referent — which is the same
class of decision as the GSI delete, where faithfulness to the interface was exactly what was at stake.
