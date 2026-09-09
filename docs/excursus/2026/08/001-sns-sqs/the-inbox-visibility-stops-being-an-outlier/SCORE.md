# SCORE — the inbox visibility stops being an outlier

Struck on `sns-sqs`, from `ea7fc81ba`. Everything uncommitted.

**Result in one line:** the inbox visibility is now `inbox-vis-ms` (CLI `argv 11`, `run-with`'s 16th
parameter), the sweep chose **200 ms**, and the ~5.2 s slow-mode drain falls to **357–405 ms** — but
the sweep also **refuted the trade-off the DESIGN was built around**: duplicate fan-out happens at
5 s too, and its volume is flat across a 50× range of this knob.

**Floor, verbatim** (`.floor/2026-09-09T22-34-05Z/clean.log`):

```
     Summary [ 483.614s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

0 lines matching `^ *(FAIL|TRY|TIMEOUT|ABORT|SIGSEGV)`, no `ARM.txt`. **No STOP fired.**

---

## ⭑ THE SWEEP

`50 2 2 32 false 0 0 1000 42 <inbox-vis-ms>` — 5 values × (3 idle + one 8-concurrent burst) = **40
cells**. `slow?` is `drain-stale-max > 0`, i.e. the drain actually saw a no-progress streak; it
separates the two clusters cleanly in every cell. `sub-acc` is per-sub `accepted` — **n=50, so any
value above 50 is duplicate fan-out.**

### Idle (×3, box quiet before each batch)

| inbox-vis-ms | run | dup | inbox `redeliveries` | sub-acc | drain (ms) | slow? |
|---|---|---|---|---|---|---|
| **5000** | r1 | 0 | 0 | 50/50 | 66 | no |
| | r2 | 0 | 0 | 50/50 | **5165** | **YES** (stale 199) |
| | r3 | 0 | 0 | 50/50 | 110 | no |
| **1000** | r1 | 0 | 0 | **59**/50 | **1148** | **YES** (45) |
| | r2 | 0 | 0 | 50/50 | **1147** | **YES** (49) |
| | r3 | 0 | 0 | 50/50 | 66 | no |
| **500** | r1 | 0 | 0 | 50/50 | 73 | no |
| | r2 | 0 | 0 | 50/50 | 73 | no |
| | r3 | 0 | 0 | 50/50 | 70 | no |
| **200** | r1 | 0 | 0 | 50/50 | 70 | no |
| | r2 | 0 | **9** | **59**/50 | **372** | **YES** (12) |
| | r3 | 0 | 0 | 50/50 | 87 | no |
| **100** | r1 | 0 | 0 | **59**/50 | **357** | **YES** (13) |
| | r2 | 0 | 0 | 50/50 | 75 | no |
| | r3 | 0 | 0 | 50/**59** | **341** | **YES** (11) |

### 8-concurrent (one burst of 8 per value, all through `capped.sh`)

| inbox-vis-ms | w | dup | inbox `redeliveries` | sub-acc | drain (ms) | slow? |
|---|---|---|---|---|---|---|
| **5000** | w1 | 0 | 0 | 50/50 | 141 | no |
| | w2 | 0 | 0 | 50/50 | **5292** | **YES** (302) |
| | w3 | 0 | 0 | 50/50 | 204 | no |
| | w4 | 0 | **9** | **60**/50 | **5316** | **YES** (299) |
| | w5 | 0 | 0 | 50/50 | 196 | no |
| | w6 | 0 | 0 | **59**/50 | **5249** | **YES** (313) |
| | w7 | 0 | 0 | 50/50 | 142 | no |
| | w8 | 0 | 0 | **59**/50 | **5281** | **YES** (306) |
| **1000** | w1 | 0 | 0 | 50/50 | **1281** | **YES** (41) |
| | w2 | 0 | 0 | 50/50 | 155 | no |
| | w3 | 0 | 0 | 50/50 | **1259** | **YES** (36) |
| | w4 | 0 | **9** | **60**/50 | **1191** | **YES** (35) |
| | w5 | 0 | 0 | 50/50 | **1241** | **YES** (35) |
| | w6 | 0 | 0 | 50/50 | 147 | no |
| | w7 | 0 | 0 | 50/**60** | **1156** | **YES** (42) |
| | w8 | 0 | 0 | 50/50 | **1202** | **YES** (52) |
| **500** | w1 | 0 | **9** | 50/50 | **739** | **YES** (20) |
| | w2 | 0 | 0 | 50/50 | 107 | no |
| | w3 | 0 | 0 | 50/50 | 134 | no |
| | w4 | 0 | 0 | 50/**59** | **739** | **YES** (25) |
| | w5 | 0 | 0 | 50/**59** | **643** | **YES** (24) |
| | w6 | 0 | 0 | 50/50 | 177 | no |
| | w7 | 0 | 0 | 50/**60** | **587** | **YES** (22) |
| | w8 | 0 | **9** | **60**/50 | **675** | **YES** (25) |
| **200** | w1 | 0 | 0 | 50/50 | 110 | no |
| | w2 | 0 | 0 | 50/50 | 145 | no |
| | w3 | 0 | 0 | 50/50 | 243 | no |
| | w4 | 0 | 0 | 50/50 | 259 | no |
| | w5 | 0 | 0 | 50/50 | 215 | no |
| | w6 | 0 | **9** | **60**/**59** | **405** | **YES** (8) |
| | w7 | 0 | 0 | 50/50 | 257 | no |
| | w8 | 0 | **10** | **60**/**60** | **365** | **YES** (12) |
| **100** | w1 | 0 | 0 | 50/50 | 117 | no |
| | w2 | 0 | 0 | **59**/50 | **358** | **YES** (15) |
| | w3 | 0 | 0 | 50/50 | 171 | no |
| | w4 | 0 | 0 | 50/50 | 134 | no |
| | w5 | 0 | 0 | 50/**59** | **342** | **YES** (15) |
| | w6 | 0 | 0 | 50/50 | 150 | no |
| | w7 | 0 | 0 | 50/50 | 156 | no |
| | w8 | 0 | 0 | 50/50 | 173 | no |

### Aggregate — 11 runs per value

| inbox-vis-ms | slow-mode drain (ms) | slow-mode incidence | runs with duplicate fan-out | excess bodies |
|---|---|---|---|---|
| 5000 | **5165 … 5316** | 5 / 11 | 3 / 11 | **28** |
| 1000 | 1147 … 1281 | 8 / 11 | 3 / 11 | **29** |
| 500 | 587 … 739 | 5 / 11 | 4 / 11 | **38** |
| **200** | **357 … 405** | 3 / 11 | 3 / 11 | **48** |
| 100 | 341 … 358 | 4 / 11 | 4 / 11 | **36** |

Fast-mode drain is 66–259 ms at every value — this knob does not touch it.

---

## ⭑⭑ TWO THINGS THE SWEEP FOUND THAT NOBODY PREDICTED

### 1. The payoff SATURATES at ~200 ms, and the floor is the receive's own wait

`5165 → 1147 → 587 → 357 → 341`. Slow-mode drain tracks the constant **1:1 down to 500 ms**
(1.03×, 1.15×, 1.3× of the visibility) and then **stops**: 200 ms gives 357–405 and 100 ms gives
341–358 — the same number, inside run-to-run spread, at half the visibility.

★ **The residual ~350 ms floor is the worker's own receive wait.** `sns-fanout.wat:417` claims with
`:wait (UpTo (Milliseconds 250))`, so once the visibility drops below that wait, the *wait* bounds
how fast an expired entry can be re-claimed — not the visibility. The DESIGN named 250 ms as the
**hazard** boundary; the sweep shows it is also the **payoff** boundary, and they are the same number
for the same reason. **Nothing below 200 ms buys latency**; it only widens the window in which an
entry expires under a live claim.

### 2. ⛔ THE POSITED TRADE-OFF IS NOT IN THE DATA

The DESIGN's contract decision rests on: *"A shorter visibility means earlier redelivery, which means
the worker's fan-out can be re-issued while the first attempt is still in flight — duplicate work."*

**Duplicate fan-out happens at 5000 ms too — 3 of 11 runs, 28 excess bodies — and at 100 ms it is
2… 4 of 11 with 36.** Across a **50× range of this knob** the incidence sits at 3–4 of 11 and the
excess-body count sits in a 28–48 band with no ordering. The knob does not cause duplicate fan-out.

★ **What causes it is an asymmetric partial refusal, and the code says so.** `ok` is *"the LONGEST
PREFIX of `items` that EVERY subscriber took"* — the min over subs of `nacc` (`sns-fanout.wat:445`).
When sub[0] takes all 10 of a batch and sub[1] takes 1, only that 1 is acked; the 9-item suffix stays
unacked, redelivers, and is re-sent **to sub[0], which already holds it**. That is exactly the
`59/50` and `60/50` rows, always 9 or 10 — one batch. The visibility sets **how long** that recovery
takes, not **whether** it happens. It is visible in the `sub-ref` column: every duplicate-fan-out run
has a refusal, and the symmetric ones (`1/1` with equal `nacc`) produce none.

⛔ So `redeliveries` and `drain` are **not** two ends of a trade-off — the DESIGN's premise. Latency
is purchasable here at no measured cost in duplicate work.

### 3. ⛔ And the instrument the DESIGN nominated is PARTLY BLIND

Row 6 asked for inbox `redeliveries` as "the cost". **It under-reports.** `idle-1000-r1` shows
`sub-acc=59/50` — nine bodies delivered twice, which requires nine inbox redeliveries — with
`redeliveries=0`. Six such rows exist in the table.

The counter is incremented in exactly one place, `sqs.wat:888–917`, on the **synchronous** `-receive`
path. Every other delivery site hands envelopes to a **parked waiter** as a `Directed` reply —
`sqs.wat:623` (a `send` fulfilling a waiter), `:1337` and `:1699` (tick paths) — and each of those
carries `seen-ids` through **unchanged** and never touches `redeliveries`. Since `seen-ids` is never
pruned (only grown, at `:917`), a `0` is a real claim about the sync path and says nothing about the
waiter path.

★ The positive control is in the table: `idle-200-r2` and `load-200-w8` DO report 9 and 10, so the
counter works — it is the coverage that is partial. **`sub[i].accepted > n` is the instrument that
actually sees duplicate fan-out**, and it is the column I graded row 6 on. Not fixed here (it is
`sqs.wat`, outside the radius); named for the builder.

**Reachability was positive-controlled before any sweep cell ran**, so a flat table could not be
mistaken for a dead knob: `12 2 2 32 false 0 0 0 0 1` → `inbox-rd=2`, vs `…0` → `inbox-rd=0`.

---

## The chosen default: **200 ms**

`(:wat::core::defn :fanout::inbox-vis-default-ns [] -> :wat::core::i64 200000000)`

1. **It is where latency saturates.** 100 ms is indistinguishable from 200 (341–358 vs 357–405) and
   puts the visibility at 0.4× the receive's own 250 ms wait for no gain.
2. **It costs nothing measurable.** Duplicate fan-out is flat in this knob and present at 5 s.
3. **It makes the corpus uniform.** All seven `:demo::mk-tw` sites now agree. The prior was 200 ms and
   the data landed on it — ★ but note it landed there **because of the 250 ms receive wait**, which
   is a different reason from "six sites already use it".
4. **It is unconditional.** It does *not* inherit `vis-ms`'s 1000 s no-drops branch, per the
   trap-door: an inbox entry that never redelivers turns one lost ack into a permanent stall.

⚠ **Honest gap: 250–300 ms was not swept.** The saturation argument points at *"at or just above the
receive wait"*, and 250 is the un-tested candidate that would sit on the safe side of the hazard at
the same latency. 200 is the swept value; 250 is a hypothesis. **A knob is the right home for that
question now, which is what this stone actually delivers.**

---

## Rows, graded

| # | expected | result |
|---|---|---|
| 1 | ⛔ `dup = 0` at EVERY value | ✅ **0 in all 40 sweep cells** + 3 write-path runs + 2 old-default controls. STOP-1 did not fire |
| 2 | ⛔ happy path intact | ✅ ×3 at the 200 ms default: `distinct=8000 dup=0`, inbox `accepted=2000 acks=2000`, all four subs exactly `2000`, no raise, rc 0 |
| 3 | ⛔ stall gate still fires | ✅ `5 1 0 32 false 0` → **rc=2**, `drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601 elapsed=13028` |
| 4 | ★ sweep complete, all four columns | ✅ 5 values × (3 idle + 8-way) = 40 cells, every column above, **plus** `sub-acc` and `sub-ref`, which row 6 needed and `redeliveries` could not supply |
| 5 | ★ the 67× term shrinks | ✅ **worst drain 5316 → 405 ms (13×)**. The slow/fast ratio goes ~80× → ~5.8×. ⚠ And it shrinks **1:1 with the constant down to 500 ms only** — below that the receive wait, not this value, is the bound |
| 6 | ★ cost quantified, not asserted | ✅ quantified — **and the nominated instrument was wrong.** Excess bodies 28/29/38/48/36 for 5000/1000/500/200/100; incidence 3,3,4,3,4 of 11. Idle/loaded split in both tables. ⛔ `redeliveries` under-reports (see above), so the number is from `sub[i].accepted` |
| 7 | `sns-fanout.wat` untouched | ✅ `git status --porcelain` = ` M wat-scripts/fanout/circuit.wat`, nothing else |
| 8 | five… seven chaos scenarios pass | ✅ `Summary [ 31.945s] 7 tests run: 7 passed, 5252 skipped` |
| 9 | corpus loads | ✅ `PASS [ 483.607s] wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` |
| 10 | floor holds | ✅ `Summary [ 483.614s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — exactly the predicted 5237/22, 0 FAIL, 0 TIMEOUT |
| 11 | blast radius | ✅ `circuit.wat` + this SCORE. Nothing forced outside it. STOP-3 did not fire |
| **12** | ★★ **the outlier is wrong** | ✅ **your recommendation holds — but not for your reason.** 5 s is a 13× latency penalty for **zero** measured duplicate-work saving, so the shorter value is a net win. ⛔ *"Short visibilities cost more in premature redelivery"* — the mechanism you priced the trade at — **is not in the data at all.** You were right that the outlier was wrong and wrong about what made it wrong. **STOP-2 did not fire** |
| **13** | ⚠ **not credited with the chaos reds** | ✅ **This stone fixed nothing that was red.** All 7 chaos scenarios were already passing at `18a86fe27` via the progress-bounded poller, and they pass here **by that same mechanism** — the poller is untouched. No rate, cap, `:max-entries`, `sub-cap`, or `vis-ms` was changed |

**Grade: `1 ✅ · 2 ✅ · 3 ✅ · 4 ✅ · 5 ✅ · 6 ✅ · 7 ✅ · 8 ✅ · 9 ✅ · 10 ✅ · 11 ✅ · 12 ✅ · 13 ✅`.
No STOP fired.**

---

## The edit — real site count

The BRIEF said **"the one site to change"**. The real count is **13 sites in 12 hunks**, one file:

| what | sites | why |
|---|---|---|
| the `mk-tw` literal (`:2323`) — **the BRIEF's site** | 1 | `5000000000` → `inbox-vis`, and its stale comment rewritten |
| `run-with` signature — `inbox-vis-ms`, 16th param | 1 | |
| `inbox-vis` let binding | 1 | `ms → ns`, `0` = default |
| new `:fanout::inbox-vis-default-ns` + header comment | 1 | gives the constant a name, a home, and the sweep table |
| `run-with` callers — `run*`, `run-p*`, `run-chaos*`, `run-drop*`, `drop-recv-tiny`, `drop-ack-tiny`, `:user::main` | **7** | ⚠ **forced by the arity change.** Every caller positions by index |
| CLI — usage string, comment block, `argv 11` threading | 3 | |

★ **The seven forced call sites are the DESIGN's deferred smell arriving on schedule.** It named
*"`run-with`'s parameter list is at 15 and this makes 16"* as out of scope; the cost of that deferral
is measured here at 7 mechanical edits for one semantic one. Still not done — it is a corpus
migration and belongs in `wat/fix.wat`, not here.

**Nothing changed outside the stated radius.** `sns-fanout.wat`, `sqs.wat`, `.config/nextest.toml`,
and `scripts/` are untouched.

---

## ⚠ Named for the builder, not acted on

1. ⛔ **`sqs.wat`'s `redeliveries` counter is blind on the waiter path** (`:623`, `:1337`, `:1699`
   deliver envelopes without consulting or updating `seen-ids`). It reads 0 while duplicate delivery
   is demonstrably happening. Any future reasoning about redelivery that quotes this counter is
   quoting a partial instrument. This is a one-site fix in `sqs.wat` and it was out of radius.
2. ★ **250 ms is the un-swept candidate** the saturation argument actually points at — at or just
   above `sns-fanout.wat:417`'s receive wait, same latency as 200, on the safe side of the hazard.
3. ⚠ **The write path cannot see this knob at all.** `2000 4 3 8192 true 1000` gives drain 2461/2475/2510
   at 200 ms and 2490/2606 at 5000 ms — indistinguishable, because with `drop-ack-bp 0` no inbox entry
   is ever left unacked. **The default change is free on the happy path and only acts under fault.**
   That is the right shape, and it also means the write path is not evidence for the choice.
4. ★ **The two tiny chaos cells no longer cross nextest's 15 s SLOW warning** — 10.8–11.8 s here,
   against the 16.3/16.6 s recorded at `18a86fe27`, because a slow-mode run now waits out 200 ms
   instead of 5 s. The DESIGN predicted exactly this (*"they fall back under it on their own"*).
   `.config/nextest.toml` was correctly not touched. ⚠ **This is not a red being fixed** — those cells
   were green before and after; only their duration moved.
5. ⚠ **`sub-cap 32` refusals are the actual generator** of every slow-mode event in this scenario. The
   duplicate fan-out follows from the min-over-subs ack rule meeting a partial refusal. Reducing
   *that* — not this timeout — is what would remove duplicate work. Not swept, not touched: the
   trap-door forbade moving `sub-cap`, correctly, since comparability needed one variable.
6. ⚠ **Untouched, as specified:** the inbox's absent *fault* injection (`circuit.wat:2178`'s three
   hardcoded zeros — now materially cheaper to build, per the DESIGN), the residual drain slope,
   `scan-index`, `count-index`, `TakeAcc`'s per-waiter-per-fold allocation, the poller's 300
   round-trips, and `poll-until-filled*`'s attempt bound.

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

**Floor from `.floor/2026-09-09T22-34-05Z/clean.log`:**
`Summary [ 483.614s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 lines matching
`FAIL|TRY|TIMEOUT|ABORT|SIGSEGV`, no `ARM.txt`.

**Row 3, run by me** — the gate must still be able to go red:
`circuit.wat 5 1 0 32 false 0` → **rc=2**, `drained-stalled … outbox=5 acks=0 polls=601`.
**Row 10:** `M wat-scripts/fanout/circuit.wat` + the SCORE. `sns-fanout.wat`, `sqs.wat`,
`.config/nextest.toml` untouched. The default is a named function,
`:fanout::inbox-vis-default-ns` → `200000000` at `:2280`, unconditional — so it cannot inherit
`vis-ms`'s 1000 s branch.

## ⭑ I closed the gap the executor flagged, and it confirms their default

The SCORE honestly records that **250–300 ms was never swept** and that the saturation argument
points at "at or just above the 250 ms receive wait." I ran 250 at their power — 3 idle + 8
concurrent, same protocol:

| inbox-vis-ms | slow | slow-drain (ms) | excess bodies | `dup` |
|---|---|---|---|---|
| 250 | 4/11 | 350 · 374 · 375 · 399 | **18** | 0 |
| 200 | 3/11 | 349 · 356 · 376 | **27** | 0 |

Latency is indistinguishable. **200 ms stands as the default.**

## ⛔ TWO OF MY CONTRIBUTIONS TO THIS STONE WERE ERRORS

### 1. My "hazard" was a misreading of long-poll semantics

The DESIGN's central warning — *"at 200 ms the visibility is SHORTER than the receive's 250 ms wait,
so an entry can expire while the worker that claimed it is still working"* — **is false.** Read at
`sqs.wat:917-960`: the receive replies immediately with `envs` when the take found rows; the
`Wait::UpTo d` arm constructs a `:queue::Waiter` **only when there is nothing to return.** The wait
precedes the claim. There is no window in which a claimed entry sits unworked for the remainder of a
wait.

★★ **And the executor's explanation is better than the one I gave them.** They wrote: *"once
visibility < wait, the wait bounds re-claim."* That is the correct mechanism and it checks out
arithmetically — 200 ms visibility **plus** up to 250 ms before any receiver picks the entry back up
≈ the observed 350–400 ms floor. Shrinking visibility below the wait cannot help, because the
**re-claim** wait dominates. My version located the effect in the wrong half of the cycle.

### 2. My reading of the excess-bodies column was a trend read into noise

I challenged their *"flat across a 50× range"* by pointing at `28 → 29 → 38 → 48` as a monotone 71 %
rise toward the short end. **My own 200 ms cell returned 27 where theirs returned 48 — for the same
configuration.** That between-run gap is larger than the 200↔250 gap I was trying to resolve, so the
statistic cannot support any trend at this power. **Their "flat" was right and my objection was the
error I have been correcting in others all session.**

## ⭑⭑ Three findings the DESIGN did not predict, and one refutes its premise

1. ⛔ **The trade-off this stone was built on is not in the data.** I designed a sweep to price
   *latency against duplicate work*, asserting shorter visibility buys the first at the cost of the
   second. Duplicate fan-out happens at **5000 ms too** (3/11, 28 excess bodies) and does not track
   the knob. **The knob sets how long recovery takes, not whether duplication happens.**
2. ★★★ **The real cause of duplicate fan-out is named:** `ok` is the **min over subscribers**
   (`sns-fanout.wat:445`), so on an asymmetric partial refusal a subscriber that accepted the whole
   batch **keeps it** while the unacked suffix redelivers to everyone. That is the safety property
   working — and it means the system does 28–48 bodies of provably wasted work per 11 runs, invisible
   at the consumer because `seen` absorbs it. **This is a new, independent finding and it outranks the
   constant.**
3. ⛔ **The instrument I nominated in row 6 is partly blind.** Inbox `redeliveries` increments only on
   the synchronous `-receive` path (`sqs.wat:888-917`); the waiter-fulfilment paths (`:623`, `:1337`,
   `:1699`) pass envelopes through unchanged. Six rows show `sub-acc=59/50` with `redeliveries=0`. The
   executor graded on `sub[i].accepted > n` instead and kept a positive control in the table. ★ Fifth
   time today I nominated an instrument that answers a narrower question than the one asked.

## Row 12 — my recommendation holds, and not for my reason

5 s is a **13× latency penalty** (5316 → 405 worst-case) for **zero measured** duplicate-work saving.
So the outlier was wrong. But my argument for why — *"it trades latency for duplicate work"* — is
refuted: there was no trade. ★ I was right that one site disagreeing with six was suspicious, and
wrong about what made it wrong. **A pattern-match that happens to point at a real defect is still a
pattern-match**, and the SCORE says so.

## Row 13 — honoured, and worth stating precisely

All seven chaos scenarios were **already green** at `18a86fe27` and pass here by that same
progress-bounded poller, untouched. Their *duration* moved (16.3/16.6 s → 10.8–11.8 s, back under
nextest's 15 s warning) — a duration is not a red being fixed, and the two cells the previous stone
named for a ruling have retired themselves.

## Grade

`1 ✅ · 2 ✅ · 3 ✅ (run by me) · 4 ✅ · 5 ✅ · 6 ✅ (on a better instrument than the one I named) ·
7 ✅ · 8 ✅ · 9 ✅ · 10 ✅ · 11 ✅ · 12 ✅ conclusion, ⛔ my reasoning refuted · 13 ✅`

No STOP fired. 13 edit sites in one file — of which **7 are forced `run-with` callers**, the
16-parameter smell the DESIGN named as out-of-scope arriving on schedule: seven mechanical edits to
carry one semantic one.

## What this hands the next stone

★ **`ok = min over subs` costs 28–48 wasted fan-out bodies per 11 runs** under 10 % ack-reply loss,
and the knob cannot reduce it. That is now the sharpest open item on the reliability list — ahead of
wiring the inbox into fault injection, because it is a *measured* inefficiency in the safety property
rather than an untested path.
⚠ And `sqs.wat`'s inbox `redeliveries` counter should count on every delivery path, not just the
synchronous one. Out of radius here; named.
