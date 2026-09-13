# DESIGN — accepted is a count

Builder's ruling **D1-c**: *"ASK THE STORE — a store call whose reply was lost → the queue QUERIES whether
the write landed, instead of reporting `Accepted 0` for a row that exists."* Third of the five.

⛔ **CORRECTION, caught by the cross-check pass before handover:** I justified taking D2 first by claiming its
chaos gate *"now watches this path."* **It does not.** The `disrupt` poison targets `Seen/check`
(`circuit.wat:537`) — a **worker→seen-service** call — while the §2d arms are the queue's **`Store/put`**
(`sqs.wat:574`). Different peers entirely.

★★ **And the honest consequence is a finding, not just a correction:** the **store-fault proxy is in no floor
test either** (`grep -rln faulting-store tests/**/*.rs` → nothing). D2 gated the disrupt injector and left the
store injector exactly as unwatched as it found it. **So this path is still gated by nothing**, and the
proxy-driven probe below is the only thing that exercises it. That is the next stone after this one, and it is
D2 applied to the other injector. The ordering was still defensible — D2 was cheapest and gates come before
changes — but **not for the reason I gave.**

**Drawn 2026-09-13. NOT STRUCK.**

## ⭑⭑ THE DEFECT IS SHARPER THAN "0 IS A LIE" — the field carries TWO MEANINGS

The arm says so itself (`sqs.wat:806`):

> *"Do not claim `Accepted n` — the put is unknowable. **`Accepted 0` is the caller's retry.**"*

So `0` was never a claim about the store — it is a **retry signal**, chosen deliberately by an author who
knew the put was unknowable. The defect is that **`Accepted n` is being used as two different things**, and
no caller reads it as the second one:

```
sns-fanout.wat:118   ((Accepted accepted) … Continue …)     reads n as a COUNT → under-reports a row
                                                            that IS in the store (real-rows=1)
sns-fanout.wat:847   ((Accepted n) (if (= n 1) nil          reads n as SUCCESS/FAILURE → raises
                       (assertion-failed! …)))
```

★ That is the counter-unit defect one level up: not a counter whose *unit* is assumed, but a **field whose
meaning is overloaded**. Neither caller is wrong about its own reading; the field cannot serve both.

## The fix: make `n` always a count, by asking

`rows`, `sk` and `take` are all in scope at the arm, and the store has the verb:
`Store::ScanRequest [pk sk-lo sk-hi limit cursor]` (`wat/query.wat`). The rows of one send share `pk = q` and
carry consecutive `sk`s, so **one base-table scan over their `sk` range answers exactly "which of these
landed."**

```
Lost | Closed | TimedOut on the put
  → redial (already there, sqs.wat:782)
  → SCAN (pk=q, sk-lo=min(rows.sk), sk-hi=max(rows.sk))
  → n = |scanned ∩ rows.sk|
  → Accepted n          ← a true count, in BOTH directions
```

A caller then retries because `n < requested`, not because `n` is a magic `0`. **Both existing callers become
correct without changing either.**

## The one contract decision

> **`Accepted n` is a COUNT and only a count.** It never doubles as a retry signal, a status, or a sentinel.

⚠ **And the residual, named rather than hidden:** if the scan on the **freshly redialed** peer also fails,
the store answered neither the write nor the probe. That is a **dead store, not a momentary failure** — the
same condition the existing redial already raises on (*"peer is dead, not a broken pipe"*,
`sqs.wat:785`). So it raises, consistently with this campaign's own line: momentary → a value, dead → a
raise. **No new variant, no `:Unknown`.**

## ⭑ THE SIBLING, ASKED FIRST — and the answer is that it is NOT the same defect

The ack side's `Lost`/`Closed`/`TimedOut` arms (`sqs.wat:1215/1248/1279`) return
`AckResponse::Ok`, with their own comment: *"Do not delete. Reply Ok so the worker does not hang. Visibility
+ Seen absorb a possible duplicate."*

**That is a different shape and a defensible one:** `Ok` carries **no count**, so nothing is overloaded, and a
missed delete is recovered by the visibility timeout re-delivering and `seen-ids` deduping. ⛔ **OUT of scope,
and the reason is on the record** — but with its residual risk named: `the-queue-knows-its-own-depth`
§2f measured that `seen-ids` is written on **only one of three** delivery paths, so *"Seen absorbs a
duplicate"* is weaker than it reads. **That is a separate stone and a separate measurement**; bundling it
here would pair a `SendResponse` contract fix with a dedup-coverage question.

## The four questions

**Obvious?** YES — a field named `Accepted` holding a count is what every caller already assumes; today one
of two callers is wrong and neither is at fault. **Simple?** YES — one scan on the failure path, no new type,
no new variant, no caller changes. **Honest?** YES, and it is the whole point: it **eliminates** the
unknowable rather than representing it, and it says plainly what happens when even the probe fails.
**Good UX?** YES — `:118` stops under-reporting and `:847` stops raising, with neither line touched.

## Scope

**IN:** the three send-put arms scan-and-count · `Accepted n` becomes a true count · the residual
(probe-also-fails ⇒ dead store ⇒ raise) · a probe using **the faulting proxy from
`the-store-can-fail`** to lose a put's reply and assert `n` equals the true count.

⭑ **The instrument already exists**: `wat-scripts/query/faulting-store.wat` with `drop-reply-bp` makes the
reply vanish *after the write lands* — which is exactly the state this stone is about. **The injector built
two stones ago is this stone's test rig.**

**OUT = REJECTED:**
- ⛔ **The ack side.** Different shape, defensible argument, residual named above. Separate stone.
- ⛔ **`:Unknown` / any new `SendResponse` variant.** D1-c was ruled over D1-b precisely to avoid a state
  callers must face but cannot resolve.
- ⛔ **Landing `PATCH-measure-variant.diff` (the −17.1 %).** This **unblocks** it; it does not bank it. ⚠ So
  `rt-store` must **not** improve here — say so in the grading.
- **Changing either caller** (`sns-fanout.wat:118`, `:847`). Both become correct *because* `n` becomes a
  count. Touching them would hide whether the fix worked.

## Trap-doors

1. ⛔ **The `sk` range can contain rows from OTHER batches.** Count `scanned ∩ rows.sk`, never the scan's
   length. This is the one way to produce a plausible wrong count.
2. ⛔ **`ScanRequest` is PAGED** (`limit` + `cursor`). A batch is at most the send's `take`; set `limit`
   accordingly and either follow the cursor or prove one page suffices. A silently truncated page is a
   silently wrong count.
3. **The scan must not touch the happy path.** It belongs inside the three failure arms only — if `store-calls`
   rises on the unperturbed run, it leaked.
4. ⛔ **Do not touch the ack side or either caller.**
5. ⛔ **NOTHING IN THE FLOOR WATCHES THIS PATH** — see the correction at the top. `probe_chaos_gate_has_teeth`
   covers the *disrupt* injector only, and the store-fault proxy is in no test. Treat the proxy probe as the
   **sole** evidence, and expect no regression signal from the floor at all.
   ⚠ A **floor timeout wall** still applies to anything new you add: headroom is ~1.2× and a 19–25 s test
   already timed out at 40 s under contention (`.floor/2026-09-13T04-00-06Z/`). **State runtimes against the
   wall, not just in seconds.**
6. **Prefer a RELATION to a threshold** in any new assertion — D2 proved a threshold's flake rate is a
   function of a parameter another stone may move (`0.6 %` became `~85 %` when n dropped).
