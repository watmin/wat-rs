# SCORE — the queue knows its own depth

Struck on `sns-sqs`, from `a9d95d823`. Everything uncommitted.
**`wat-scripts/queue/sqs.wat` is left at HEAD** — see §0 for why, and the two patches beside this
file for what was built and measured.

**Result in one line:** ★ **STOP-2 fired, and it fired on a measured number, not an argument** —
`Store::DeleteResponse::Success` carries **no deleted count**, and the harness re-presents already-
deleted ids on every ack retry, so the sketch's `rows − (count ids)` drives the maintained row count
to **−40 on three of five queues in the standard run**, with *zero* injected faults and *zero*
redeliveries. The law is exact and reproduced three times: **excess acked ids = 10 × `ack-retries`.**
The admission half of the stone works and is worth **`count` 1962 → 768 (−60.9 %)** and
**`rt-store` 6661 → 5522 (−17.1 %)** over 5 interleaved pairs — but it rests on a value that is not
the store's, so it is reported, not landed.

**Floor, verbatim** (`.floor/2026-09-10T10-32-58Z/clean.log`, on the tree as left):

```
     Summary [ 483.720s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

0 lines matching `^ *(FAIL|TRY|TIMEOUT|ABORT|SIGSEGV)`, no `ARM.txt` written.

**STOPs:**

- **STOP-1 — did NOT fire, but it is answered, and the answer costs row 6.** The drift check *can* be
  placed for zero crossings — but **only on the one call the design wanted to delete.** §3.
- ⛔ **STOP-2 — FIRED.** §2. The third writer is the one the brief itself listed: **a partial delete.**
- **STOP-3 — did NOT fire.** `put + delete + count + scan` sums to `store-calls` with **remainder 0**
  on every tier of all 12 runs, both arms. §5.
- **STOP-4 — did not fire on the floor or the corpus gate.** ⚠ But **one of six runs of the measured
  build went red in my own experiment** and is reported whole in §6. It was not re-run away.

---

## 0. Why the file is left at HEAD

The brief says of STOP-2: *"Report the path; that is the complete result."* It is reported in §2.
Closing it needs a data structure the sketch does not name (§7) — a design decision that is the
builder's, not mine — and there is a **second, unclosable** drift path (§2c) plus **an unexplained
1-in-6 red** (§6). Landing an admission path that reads a value which reaches −40 would ship a queue
whose capacity check is not about the store. So:

- `wat-scripts/queue/sqs.wat` — **reverted to HEAD**, byte-identical.
- `PATCH-as-designed.diff` — the stone exactly as the sketch specifies (raise armed,
  `unacked = rows − visible`). **This is the build that raised.**
- `PATCH-measure-variant.diff` — the same, with the raise computed but disarmed and `unacked` left
  as the queried value, so the run completes. **Every number in §4 was taken with this file.**
- `git status --porcelain`: the SCORE + those two patches. Nothing else.

---

## 1. The refutation, in one line of a baseline run

⛔ **It is visible in a run of `HEAD`, before any edit.** From the orientation baseline
(`2000 4 3 8192 true 1000`):

```
tier=inbox   ;accepted=2000;refused=189;acks=2000;redeliveries=0;visible=0;unacked=0
tier=sub[0]  ;accepted=2000;refused=0  ;acks=2040;redeliveries=0;visible=0;unacked=0
tier=sub[1]  ;accepted=2000;refused=0  ;acks=2040;redeliveries=0;visible=0;unacked=0
tier=sub[2]  ;accepted=2000;refused=0  ;acks=2040;redeliveries=0;visible=0;unacked=0
tier=sub[3]  ;accepted=2000;refused=0  ;acks=2040;redeliveries=0;visible=0;unacked=0
                                              ↑ ack-retries=16
```

**`acks` (2040) exceeds `sends-accepted` (2000) by 40 on every subscriber queue, while
`visible=unacked=0` says the store is empty.** 2000 rows in, 2000 rows out, **2040 ids presented**.
`rows = 2000 − 2040 = −40`. The number that refutes the design was already being printed.

---

## 2. ⛔ STOP-2 — the third writer, named and measured

### 2a. The mechanism: `delete` does not say how much it deleted

`wat/query.wat:533-540`:

```
(:wat::core::defenum :wat::query::Store::DeleteResponse :wat::enum::Pure
  :Success        []
  ...
```

**`Success` is nullary.** The ack path has no observation of rows removed; `(count ids)` is the only
number available, and it is an **upper bound**. The trap-door in EXPECTATIONS — *"Decrement by what
was actually deleted, never by `count(ids)`"* — asks for a quantity the protocol does not carry.

### 2b. The live path that makes the bound bite: the ack-retry fold

`circuit.wat:785-808`. When `call-by-deadline` on `Queue/ack` does not answer within 200 ms, the
worker **re-sends the same `AckRequest` with the same ids**, up to the visibility limit. The queue
deletes again — removing nothing, because the first delete already landed — and bumps `acks` (and,
under the sketch, decrements `rows`) by `count(ids)` a second time. The queue's own drop-ack arm
(`sqs.wat:1097-1099`) is the same shape from the other side: it **performs the delete and suppresses
the reply**, which is precisely what makes the caller retry.

⭑ **The law, exact in three independent runs** — batches are `:limit 10`, so:

| run | `ack-retries` | Σ sub `acks` − Σ sub `accepted` | predicted `10 × retries` |
|---|---|---|---|
| baseline (HEAD) | 16 | 8160 − 8000 = **160** | 160 ✓ |
| drift run A | 5 | 8050 − 8000 = **50** | 50 ✓ |
| drift run B | 12 | 8120 − 8000 = **120** | 120 ✓ |

(the inbox tier is excluded — its acker, `sns-fanout.wat:552`, does **not** retry; it redials and
drops the ack. Its `acks` equalled its `accepted` in every run, and its drift was 0.)

### 2c. And the drift, read straight off the instrument

The measure build was run once with `Stats/redeliveries` temporarily carrying `maintained − queried`
(a reported field only; behaviour byte-identical, `distinct=8000 dup=0`). Final per-tier drift:

| tier | accepted | acks | **maintained − queried** |
|---|---|---|---|
| inbox | 2000 | 2000 | **0** |
| sub[0] | 2000 | 2000 | **0** |
| sub[1] | 2000 | 2040 | **−40** |
| sub[2] | 2000 | 2040 | **−40** |
| sub[3] | 2000 | 2040 | **−40** |

`ack-retries=12` for that run; **−120 = 12 × 10.** Every unit of drift is attributable.

### 2d. A SECOND writer that cannot be closed by a counter at all

⚠ Independent of the ack rule: `sqs.wat:745 / 779 / 810` — the send put's **`Lost` / `Closed` /
`TimedOut`** arms. Their own comment: *"Do not claim Accepted n — the put is unknowable."* The rows
**may** have landed; the handler cannot know, so no increment is available in either direction. The
same shape sits on the ack side at `sqs.wat:1208 / 1241 / 1272` (*"Do not delete"* — but the delete
was already sent, and its outcome is unknown).

★ **This is the structural difference the DESIGN's exactness argument misses.** A `count-index` query
is *robust* to an unknowable write — it asks the store afterwards. A maintained counter is not. The
paths did not fire in these runs (`inbox-lost=0 inbox-closed=0 inbox-timedout=0`), so this one is
reported from the code, not from a measurement.

### 2e. And a third thing found on the way — `sends-accepted` already under-counts

`sqs.wat:1626` `send-after-put` (the `PutResponse::Transient` → `retry-put` success path) returns
`Accepted n-ok` and passes `:counters` through **by reference** — it never bumps `sends-accepted`.
Any `rows` increment placed beside `sends-accepted` inherits that hole. Did not fire here
(`sends-accepted == accepted == 2000` on every tier).

### 2f. And `redeliveries` under-counts, for a reason that matters here

`seen-ids` is written **only** on the immediate `receive` reply path (`sqs.wat:872-917`). The two
*other* delivery paths — the waiter fold inside `send` (`sqs.wat:631`) and `-tick` (`sqs.wat:1344`)
— call the same `take` and hand envelopes to parked receivers **without touching `seen-ids`**. So
`redeliveries=0` on a tier line does **not** mean no message was delivered twice, and `seen-ids`
cannot be reused as the live-id set (§7).

---

## 3. ⛔ STOP-1 answered — the check is free, and that is exactly why row 6 cannot be had

**Where it is placed:** the **second `count-hi` inside the `depth` closure** — the one at `+inf`
(`sqs.wat:378`, `all-pair`), reached from `stats` (`sqs.wat:1247`). `depth` returns
`(visible, all − visible, ns)`, so `visible + unacked` **is** the store's own row count for this
queue, already on the wire. The check compares it against the maintained value and
**`assertion-failed!`s** on mismatch — a raise, not a counter, not a log line — naming both numbers,
`visible`, `unacked`, the queue, `sends-accepted` and `acks`.

**It costs zero crossings.** Verified mechanically, by histogramming crossing primitives over the
added-versus-removed lines of `PATCH-as-designed.diff`:

| primitive | added | removed |
|---|---|---|
| `Store/put`, `Store/delete`, `Store/scan-index`, `kernel::connect`, `kernel::send`, `kernel::recv`, `call-by-deadline`, `Queue/*` | **0** | 0 |
| `Store/count-index` | **0** | 1 (a comment; the live call site is `State/total`'s `apply`) |

Corroborated by §5: the reconciliation remainder is 0 and `count` per tier is exactly `2 × stats
calls` after the change.

### ★ The finding: rows 3–4 and row 6 are mutually exclusive

I enumerated every place the service touches the store:

| site | what it observes | usable as the check? |
|---|---|---|
| `send` → `total` (`:470`) | full range `[0, +inf)` | **this is the call the stone deletes** |
| `stats` → `depth` `count-hi @ now-ns` | `[0, now]` — visible only | no: not the row count |
| `stats` → `depth` `count-hi @ +inf` | **full range** | ✅ **the only one** |
| `take` → `scan-index` | `[0, now]`, truncated at the receive limit (10) | no |
| `ack` → `delete` | returns nothing (§2a) | no |
| `-tick` | queries nothing | no |

⛔ **`stats`'s `+inf` call is the only free full-range observation of the store on a path the stone
keeps.** Delete it (row 6) and there is nowhere left to place a free check; keep it and the check is
free but `stats` stays at `store-calls + 2`. **You cannot have both.** STOP-1's escape clause —
*"if you cannot place it free, stop and report that"* — does not fire, because it *can* be placed
free; what it costs is row 6 and about a third of the predicted `count` win.

★ And there is a reframing that makes the kept call honest rather than redundant: with the check in
place, that `+inf` count is no longer *observability*. It is **verification** — the price of the
maintained value being provable instead of asserted.

### ⚠ And a defect in the raise itself, found by running it

The as-designed build raised **inside a forked queue service, and its message never reached the
harness's stderr.** All the operator sees is the downstream corpse:

```
filled-unread: last=[1960/0][1960/0][1960/0][1960/0] outbox=-1 attempts=0 elapsed=22
```

`outbox=-1` is emitted by `sns-fanout.wat:196` only when `Topic/stats`' inner `Queue/stats` to the
**inbox** returns Lost/Closed/TimedOut — i.e. the inbox queue was dead 22 ms into the fill poll.
So the check fired and killed the queue as designed, and **the drift numbers it was carrying were
lost.** A raise whose text cannot be read is a weaker instrument than the design assumes. (The
numbers in §2c had to be extracted through a *reported field* instead.)

---

## 4. The measurement — 5 interleaved pairs, box quiet

⛔ **Interleaved B/A/B/A…, `cp`-swapping `sqs.wat` between runs** (`load-file!` reads it at runtime;
no rebuild). Load average 0.20 at start; nothing else running. `2000 4 3 8192 true 1000`.
6 pairs launched; **pair 1's AFTER arm went red (§6)** and is excluded from the means, not hidden.

**All runs, per arm** (`distinct=8000 dup=0` on every completed run):

| run | `rt-store` | `rt-total` | **count** | put | delete | scan | store-sum | **rem** | refused | fill | drain | collect | wall |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| BEFORE-1 | 6721 | 11280 | 1969 | 2011 | 1016 | 1733 | 6729 | **0** | 191 | 2620 | 2393 | 4883 | 22688 |
| AFTER-1 | — | — | — | — | — | — | — | — | — | — | — | — | ⛔ **RED, §6** |
| BEFORE-2 | 6564 | 11076 | 1954 | 2006 | 1009 | 1603 | 6572 | **0** | 190 | 2611 | 2405 | 4506 | 22334 |
| AFTER-2 | 5549 | 10107 | **770** | 2007 | 1009 | 1771 | 5557 | **0** | 189 | 2364 | 2494 | 5179 | 22789 |
| BEFORE-3 | 6743 | 11313 | 1961 | 2006 | 1018 | 1766 | 6751 | **0** | 187 | 2597 | 2423 | 5197 | 23287 |
| AFTER-3 | 5559 | 10125 | **760** | 2007 | 1014 | 1786 | 5567 | **0** | 189 | 2345 | 2434 | 5194 | 22883 |
| BEFORE-4 | 6680 | 11255 | 1964 | 2005 | 1009 | 1710 | 6688 | **0** | 190 | 2631 | 2407 | 5472 | 23419 |
| AFTER-4 | 5547 | 10100 | **790** | 2006 | 1014 | 1745 | 5555 | **0** | 190 | 2337 | 2484 | 4627 | 22286 |
| BEFORE-5 | 6669 | 11221 | 1964 | 2007 | 1009 | 1697 | 6677 | **0** | 190 | 2636 | 2456 | 4972 | 22683 |
| AFTER-5 | 5550 | 10093 | **770** | 2005 | 1008 | 1775 | 5558 | **0** | 187 | 2320 | 2410 | 4952 | 22793 |
| BEFORE-6 | 6650 | 11212 | 1967 | 2012 | 1007 | 1672 | 6658 | **0** | 189 | 2600 | 2407 | 5167 | 22977 |
| AFTER-6 | 5403 | 9924 | **750** | 2007 | 1014 | 1640 | 5411 | **0** | 188 | 2360 | 2411 | 4408 | 22416 |

**Paired means and spreads (5 pairs, i = 2…6):**

| metric | BEFORE mean [min..max] | AFTER mean [min..max] | Δ | Δ % | signal? |
|---|---|---|---|---|---|
| ★ **`count`** | 1962.0 [1954..1967] | **768.0** [750..790] | **−1194.0** | **−60.9 %** | ✅ ranges disjoint by 1164 |
| ★ **`rt-store`** | 6661.2 [6564..6743] | **5521.6** [5403..5559] | **−1139.6** | **−17.1 %** | ✅ disjoint |
| `rt-total` | 11215.4 [11076..11313] | 10069.8 [9924..10125] | −1145.6 | −10.2 % | ✅ disjoint |
| `put` | 2007.2 [2005..2012] | 2006.4 [2005..2007] | −0.8 | −0.04 % | none (by design) |
| `delete` | 1010.4 [1007..1018] | 1011.8 [1008..1014] | +1.4 | +0.14 % | none |
| `scan` | 1689.6 [1603..1766] | 1743.4 [1640..1786] | +53.8 | +3.2 % | **none** — ranges overlap heavily |
| **remainder** | **0** [0..0] | **0** [0..0] | 0 | — | STOP-3 clear |
| **inbox `refused`** | 189.2 [187..190] | 188.6 [187..190] | −0.6 | −0.3 % | **in band 186–191** ✅ |
| `expired-waiters` (Σ) | 317.6 [281..340] | 309.8 [289..326] | −7.8 | −2.5 % | none |
| `setup` | 12410 [12373..12470] | 12442 [12385..12484] | +32.6 | +0.26 % | none |
| ★ `fill` | 2615.0 [2597..2636] | **2345.2** [2320..2364] | **−269.8** | **−10.3 %** | ✅ **disjoint, 5/5 same sign** |
| `arm` | 17.4 | 17.4 | 0.0 | 0 % | none |
| `drain` | 2419.6 [2405..2456] | 2446.6 [2410..2494] | +27.0 | +1.1 % | none — overlap |
| `collect` | 5062.8 [4506..5472] | 4872.0 [4408..5194] | −190.8 | −3.8 % | **none** — before-spread alone is 21 % |
| `stop` | 412.8 [193..580] | 507.4 [348..733] | +94.6 | +22.9 % | **none** — before-spread is 3× the delta |
| **wall `total`** | 22940 [22334..23419] | 22633 [22286..22883] | −306.6 | **−1.34 %** | ⚠ **none — ranges overlap. NO speedup claimed.** |

### ⚠ Row 16, honoured explicitly

**No wall-clock speedup is claimed.** −1.34 % with fully overlapping ranges is noise, and that is the
**predicted** outcome: ~1140 crossings × ~2.8 ms ≈ 3.2 s serialised, against ~3× achieved
concurrency, on a 22.6 s run. The numbers that matter are `count` and `rt-store`, and both are
unambiguous.

★ The **one** phase term that does isolate is `fill` (−270 ms, 5/5 same sign, disjoint ranges) — and
it is the phase that is nothing but sends, which is exactly where the removed crossing lives. It is
reported as a phase effect, not as a run speedup.

### Where the win lands versus the DESIGN's prediction

DESIGN predicted `count` ~1997 → **~400**. Measured: 1962 → **768**. The gap is **not** a shortfall
in the admission change — it is §3. Admission's contribution is the whole 1194; `stats` still makes
its two calls because the second one **is** the drift check. Had row 6 also been taken, `count`
would be ~384 — the DESIGN's ~400 was a good number for a design that cannot also be proven.

---

## 5. The reconciliation identity (STOP-3)

`count + put + delete + scan = store-calls`, **remainder 0**, on **every tier of every one of the
11 completed runs**, both arms. Worked example, AFTER-2:

```
770 + 2007 + 1009 + 1771 = 5557 = Σ store-calls      remainder 0
```

The identity survives because the term removed from one side is removed from the other: admission
now carries `store-calls` / `count-calls` through **untouched** rather than `+1` (see the
`sc0`/`cc0` hunk in the patch). `rt-store` tracks `Σ store-calls` at a constant offset of 8
(`ensure-schema`, five services) in both arms — 6669→6661 and 5530→5522.

---

## 6. ⛔ THE RED IN MY OWN EXPERIMENT — reported, not re-run away

Pair 1's AFTER arm (`ab-AFTER-measure-1.log`), the measure build, **whole message, verbatim**:

```
#wat.kernel/AssertionFailure {:thread "main" :message "filled-never: last=[2010/0][2010/0][2010/0][2010/0] outbox=0 want=2000 attempts=8000 elapsed=356669" :location #wat.kernel/Location {:file "wat-scripts/fanout/circuit.wat" :line 2652 :col 14} :actual nil :expected nil :frames [#wat.kernel/Frame {:file "wat-scripts/fanout/circuit.wat" :line 2652 :symbol ":fanout::require!"} #wat.kernel/Frame {:file "wat-scripts/fanout/circuit.wat" :line 3074 :symbol ":fanout::run-with"} #wat.kernel/Frame {:file "src/freeze.rs" :line 1521 :symbol ":user::main"}] :upstream-chain nil}
```

**The exact arm:** `poll-until-filled*` (`circuit.wat:1363-1387`), the `filled-never` verdict —
`sweep-filled?` requires **`visible = n` exactly**, and every subscriber queue held **2010**, ten
past `n = 2000`. Once it overshoots it can never satisfy the gate, so the loop burned its full 8000
attempts (357 s). **Ten** is one full `:limit 10` fanout batch: an inbox row was re-processed after
its visibility expired — the at-least-once duplicate that `sns-fanout.wat:544-550` documents as
permitted (*"leaves the inbox entry to expire and be re-processed — duplicate deliveries to the subs
that already took it"*), and that the fill gate does not permit.

**What I can say:** 0 of 6 BEFORE runs and 1 of 6 AFTER runs. The inbox's maintained value was
exact (`acks == accepted`) in every completed run, so I have **no mechanism** linking it to the
maintained counter, and the honest reading is that the change alters send-path timing and this gate
is intolerant of a duplicate the system's own contract permits.
⛔ **What I will not say:** "timing", "flake", "pre-existing" — those describe my search. **The
arm is named, the log is kept, and I did not re-run that configuration to make it go away.** It is
a second reason this needs the builder before anything lands.

---

## 7. What closing STOP-2 would actually cost — for the builder, not decided here

Three candidate shapes, in increasing order of how much they change:

1. ⭑ **Make the store say what it deleted.** `Store::DeleteResponse::Success [n <- i64]`. sqlite has
   `changes()` for free; mem knows it. **This is the real fix** — it makes the decrement observable
   instead of estimated, and everything else follows. Cost: `wat/query.wat` (stdlib, frozen into the
   binary → the BOOTSTRAP dance) plus the Rust store impls plus every `DeleteResponse::Success`
   match arm in the corpus. Out of this stone's blast radius; squarely a builder ruling.
2. **A live-id set on the queue.** `live-ids <- PersistentSet[String]`: insert the `sk`s `send`
   generates, and decrement by `|ids ∩ live-ids|` on ack. Exact for §2a/§2b, zero crossings,
   bounded by `cap`. ⛔ Cannot reuse `seen-ids` — §2f shows two delivery paths never write it. Cost:
   a new `:ephemeral` field ⇒ ~30 `State` rebuild sites, hand-edited.
3. **An explicit `:Unknown` state** for §2d: after an unknowable put or delete, the maintained value
   is marked unknown and the **next admission does one `count-index`** to re-establish it. Exact,
   self-healing, and pays a crossing only where the queue genuinely does not know. This one is
   *required* on top of either of the above — neither 1 nor 2 closes §2d.

⚠ And note what this campaign already knows: **DESIGN's §"It is EXACT" is right about the
operations** — only accepted-send and ack change the row count, and expiry does not (§8, row 15).
It is wrong about the **magnitudes**, and the magnitude is where the counter lives.

---

## 8. Grading, row by row

| # | expected | verdict |
|---|---|---|
| 1 | `distinct=8000`, `dup=0`, inbox `accepted=2000`, no raise | ✅ on all 5 completed AFTER runs. ⛔ **and a raise on the as-designed build**, §3 — which is the check working |
| 2 | reconciliation remainder 0, every tier, every run | ✅ **0 everywhere, both arms, 11 runs** (§5) |
| 3 | drift check exists and RAISES | ✅ `assertion-failed!` on `maintained ≠ queried`, naming both. ⚠ and it **did** raise (§3) |
| 4 | drift check adds no crossing | ✅ verified mechanically — **zero** crossing primitives added (§3) |
| 5 | ★ admission stops querying | ✅ `sqs.wat:470` is a field read; `count` falls by exactly admission's share (§4) |
| 6 | ★ `stats` at `store-calls + 1` | ⛔ **NOT TAKEN, and it cannot be** together with rows 3–4. §3. This is the stone's main structural finding |
| 7 | ★ `count` ~1997 → ~400 | ⚠ **PARTIAL — 1962 → 768 (−60.9 %).** Direction and magnitude right for the admission half; the other half is row 6 |
| 8 | ★ `rt-store` falls, `rt-total` with it | ✅ 6661 → 5522 (−17.1 %); `rt-total` 11215 → 10070 (−10.2 %) |
| 9 | inbox `refused` in band 186–191 | ✅ BEFORE [187..190], AFTER [187..190]. **Admission did not become more permissive** |
| 10 | stall gate `5 1 0 32 false 0` → rc 2, `drained-stalled` | ✅ rc 2, `drained-stalled: … last=[0/0] outbox=5 acks=0 polls=601 elapsed=13321` |
| 11 | chaos `7 tests run: 7 passed` | ✅ `Summary [ 32.892s] 7 tests run: 7 passed, 5252 skipped` — **including `drop_ack_tiny`, the arm that manufactures §2b** |
| 12 | `every_wat_scripts_file_loads` PASS | ✅ PASS on the as-designed build (`Summary [ 243.989s] 1 test run: 1 passed`) **and** in the floor |
| 13 | floor Summary 5237/0/0 | ✅ `Summary [ 483.720s] 5237 tests run: 5237 passed (7 slow), 22 skipped` |
| 14 | blast radius = `sqs.wat` + the SCORE | ⚠ **SCORE + 2 patch files; `sqs.wat` at HEAD.** §0 |
| **15** | ★★ **only accepted-send and ack change the row count; expiry does not** | ⛔ **SPLIT — and the half that fails is the load-bearing one.** Below |
| 16 | ⚠ no wall-clock claim; say which paths ran | ✅ Below |

### Row 15, verified from the code rather than from the sentence

**The half that HOLDS** — `expiry does not change the row count`: ✅ **confirmed, and it is a
documented store property, not an inference.** `wat/query.wat:597-604`: *"REPLACE-BY-(pk,sk) —
DynamoDB PutItem. An incoming row whose (pk,sk) already exists completely replaces the old item."*
`take`'s re-put (`sqs.wat:291`) re-writes rows it just scanned, same `pk`/`sk`, new `isk`. A
redelivery is the same row again. `retry-put` and `retry-delete` re-send identical rows/keys and are
therefore idempotent on the count. **No redelivery re-put inserts. No store-side rewrite.**

**The half that FAILS** — `only accepted-send and ack change the row count`, read as the arithmetic
the sketch builds on: ⛔ **the operations are right, the magnitudes are not.** Ack's decrement is
`(count ids)`, which is an upper bound the protocol cannot tighten (§2a), and the harness presents
the same ids again on every retry (§2b) — measured **−40 on three of five queues, zero faults, zero
redeliveries**. Plus §2d: two arms where the row count moves and no increment exists at all.

★ The brief listed the candidates that would refute it: *"a redelivery re-put that inserts rather
than updates, **a partial delete**, a store-side rewrite, the `seen-ids` path."* **It is the partial
delete** — and `seen-ids` turns out to be a red herring for a different reason (§2f).

### Row 16, honoured

⚠ **No wall-clock speedup is claimed** — §4: −1.34 % with fully overlapping ranges. The one phase
that isolates is `fill` (−10.3 %, 5/5 same sign, disjoint), reported as a phase effect only.

⚠ **Which paths actually exercised the drift check**, so *"no drift observed"* is not claimed for
anything unexercised:

| path | exercised? | evidence |
|---|---|---|
| accepted send (`+take`) | ✅ | `sends-accepted=2000` × 5 tiers |
| **refused send** (`Accepted 0`, no change) | ✅ | inbox `refused=189` every run |
| ack, first attempt | ✅ | `acks` ≥ 2000 × 5 tiers |
| **ack, duplicate ids** (the refuting path) | ✅ | `ack-retries` 5/12/16; drift −40 |
| visibility expiry / re-put | ✅ | `expired-waiters` Σ ≈ 310/run; the §6 fanout duplicate |
| **`redeliveries` > 0 on the receive path** | ⛔ **NOT exercised** | `redeliveries=0` every tier — and §2f says that counter under-reports anyway |
| **`drop-recv-bp` / `drop-ack-bp` fault injection** | ⛔ **NOT exercised in the A/B** | both 0 in `2000 4 3 8192 true 1000`. The floor's `drop_ack_tiny` arm *does* drive it, and that is where the as-designed build would raise next |
| **store-link Lost / Closed / TimedOut** (§2d) | ⛔ **NOT exercised** | `inbox-lost=0 inbox-closed=0 inbox-timedout=0` |

---

## 9. Edit sites — the real count

The sketch implies four (Counters, admission, ack, stats). The patch has **11 hunks**:

`:queue::Counters` decl · `:init` constructor · admission block (`:470`) · **six** `Counters`
re-construction sites — refused send, accepted send, receive/redeliveries, ack `Success`, ack
`Transient`, `-tick` · `stats` drift check · `stats` `:unacked`.

`+92 / −21` lines. The carrier paid for itself exactly as `9f1392630` predicted: the ~22 `State`
rebuilds that pass `:counters` through by reference needed **no edit at all**. A field on the
carrier costs 8 sites; the same field on `State` would have cost 30.

⛔ **All 11 edited with an editor.** No python, no sed. python appears once in this stone, reading
logs to build the §4 table.

---

## 10. Reproduce

```bash
D=docs/excursus/2026/08/001-sns-sqs/the-queue-knows-its-own-depth

# the refutation, on HEAD, no edit at all — look at acks vs accepted:
./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000

# the stone as designed — raises inside the inbox queue during fill:
git apply $D/PATCH-as-designed.diff
./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000
#   → filled-unread: last=[1960/0]… outbox=-1   (the inbox is dead)

# the measured build — completes, and carries the win:
git checkout wat-scripts/queue/sqs.wat && git apply $D/PATCH-measure-variant.diff

./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 5 1 0 32 false 0   # rc 2
./scripts/capped.sh --limit 8g ./scripts/floor.sh                                                     # read the Summary
```

⛔ Interleave B/A/B/A by `cp`-swapping `sqs.wat` — `circuit.wat:41` `load-file!`s it at runtime, so
no rebuild is needed and a sequential block is not a measurement.

---

## 11. What this hands the next stone

★ **The win is real and it is bigger than the admission line suggests: −1194 `count`, −17 %
`rt-store`, −10 % `rt-total`, on a change of 11 hunks in one file.** It is sitting behind one
protocol gap.

⛔ **`Store::DeleteResponse::Success` is nullary.** That single missing `i64` is why a queue cannot
know its own depth by addition, and it is upstream of this stone, of `acks`-as-a-unit, and of any
future maintained quantity that a delete moves. **Fixing it is a stdlib + Rust-store stone, and it
converts this DESIGN from wrong to right without changing a line of its argument.**

⚠ **And two counters in `sqs.wat` do not count what their names say** — `redeliveries` misses two of
three delivery paths (§2f), and `sends-accepted` misses the retry-put path (§2e). That is the third
and fourth instance today of *a counter's unit being assumed rather than read*, which the previous
SCORE already named as the cheap prerequisite to closing `rt-unknown`. It is no longer a
prerequisite for one stone — it is a pattern.

---

# ⭑ THE ORCHESTRATOR'S GRADING

**Floor** `.floor/2026-09-10T10-32-58Z/clean.log`: `Summary [ 483.720s] 5237 tests run: 5237 passed
(7 slow), 22 skipped` — 0 failure tokens, no `ARM.txt`. **Tree:** `sqs.wat` byte-identical to HEAD
(`git diff --stat` → 0 lines), three untracked files. **Nothing landed, which is correct for a STOP.**

## ⛔ ROW 15 FAILED ON ME, AND THE REFUTATION IS A ONE-LINE FACT

Verified on my own read, `wat/query.wat:533`:

```
(:wat::core::defenum :wat::query::Store::DeleteResponse :wat::enum::Pure
  :Success        []          ← NULLARY
```

**The ack path cannot know how many rows it deleted.** My BRIEF said *"decrement by what was actually
deleted, never by `count(ids)`"* — **there is no such number to decrement by.** I wrote an instruction
that the substrate makes unfollowable, and I wrote it as a trap-door, as if I had checked.

★★ **And it is exactly the item on my own candidate list.** Row 15 named *"a partial delete"* among the
things that would refute me, and then I asserted the claim anyway. **Listing a risk is not testing it** —
the fifth estimate of mine to die today, and the first that died against a candidate I had already
written down.

**Measured drift: −40 on three of five queues, in the standard run, with zero injected faults and zero
redeliveries.** Law reproduced 3×: **excess acked ids = 10 × `ack-retries`.** The worker re-presents the
same ids after an unanswered `Queue/ack`, and the queue's drop-ack arm (`sqs.wat:1097`) **deletes and
then suppresses the reply** — which is what causes the retry. A second path cannot be closed at all: the
send-put and ack `Lost`/`Closed`/`TimedOut` arms, where *"the put is unknowable."*

## ⭑⭑ The structural finding: the instrument and the optimisation are the same call

Rows 3–4 (a free drift check) and row 6 (halve `stats`) are **mutually exclusive.** The only free
full-range observation of the store on any path this stone keeps is **`depth`'s `+inf` `count-hi`** —
which is precisely the call row 6 exists to delete. `take`'s scan is `[0,now]` truncated at 10, `delete`
returns nothing, `-tick` queries nothing.

★ That is not a scoping mistake in the BRIEF; it is a property of the design, and it would have bitten
whoever built it. **Naming it is worth more than the stone.**

## What the patches measured, and it is real

5 interleaved pairs, ranges **disjoint** on both headline numbers:

| | before | after | Δ |
|---|---|---|---|
| `count` | 1962 [1954–1967] | **768** [750–790] | **−60.9 %** |
| `rt-store` | 6661 [6564–6743] | **5522** [5403–5559] | **−17.1 %** |
| `rt-total` | 11215 | 10070 | −10.2 % |
| wall `total` | 22940 | 22633 | −1.3 %, **ranges overlap — no speedup claimed** ✅ row 16 |
| inbox `refused` | 187–190 | 187–190 | ✅ row 9, admission not more permissive |

`fill` — the send-only phase — isolates at **−10.3 %, same sign 5/5, disjoint.** So the win is real and
it lands where the theory said: on the send path.

## ⛔ TWO THINGS BEYOND THE ASK, AND BOTH MATTER MORE THAN THE ROWS

**1. A raise inside a forked queue never reaches stderr.** The as-designed drift check fired and its
message was invisible — all an operator sees is `outbox=-1`, a dead inbox. **A raise whose text cannot be
read is a weaker instrument than the design assumed**, and my contract decision leaned on it: *"a raise,
because a silent drift would corrupt admission."* The raise **is** silent, from outside. That is a
substrate observability gap and it is unrelated to this stone.

**2. ⛔ ONE RUN IN SIX WENT RED, AND IT IS CAPTURED WHOLE:**

```
filled-never: last=[2010/0][2010/0][2010/0][2010/0] outbox=0 want=2000
              attempts=8000 elapsed=356669
circuit.wat:2652  :fanout::require!  ← poll-until-filled*
```

**2010 delivered where 2000 were wanted — an overshoot of exactly one batch — and 356 seconds in fill.**
The executor reported it whole, did not re-run it away, and states plainly that no mechanism links it to
the counter. ★ Note the coincidence it did **not** claim: the ack-drift law is *10 × `ack-retries`*, and
the overshoot is exactly **10**. Both involve a batch of ten being re-presented. **That is a hypothesis,
not a finding**, and it belongs to whoever chases the red.

## Grade

`1 ✅ · 2 ✅ (remainder 0, every tier, both arms, 11 runs) · 3–4 ✅ but mutually exclusive with 6 ·
5 ✅ · 6 ⚠ available only at the cost of 3–4 · 7 ✅ (−60.9 %) · 8 ✅ (−17.1 %) · 9 ✅ · 10 ✅ · 11 ✅
(chaos 7/7, including `drop_ack_tiny` — the arm that manufactures the drift) · 12 ✅ · 13 ✅ · 14 ✅ ·
**15 ⛔ FAILED — mine** · 16 ✅ honoured`

**STOP-2 firing is the whole value.** The measured −17 % on `rt-store` is banked in a patch that
**cannot be landed as designed**, and that is the correct outcome: it costs me an exactness argument and
buys a substrate defect with a one-line proof.

## The ruling now owed

**`Store::DeleteResponse::Success` is nullary, and that is itself the defect of the day's recurring
class: an operation that does not report what it did.** DynamoDB's `DeleteItem` reports consumed
capacity and can return the old item; a `Success` that cannot say how many rows went is **strictly less
informative than the referent.**

★ Give `Success` a deleted-count and **all three tensions resolve at once**: the maintained counter
becomes exact, admission goes free, and the drift check moves off `depth`'s `+inf` call — un-blocking
row 6. It reaches `wat/query.wat`, so it is the builder's to open.
