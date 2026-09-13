# SCORE — the poison tears

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `6d87a7d7c` (DRAWN). Did not commit.

```
     Summary [ 530.230s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-13T01-53-44Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` → **CLIPPY=0**.
`cargo nextest run --release --no-run` → **NORUN=0**.
`git diff --stat -- src/ wat/` is **EMPTY**.

```
 wat-scripts/fanout/circuit.wat   | 14 +++++++++-----
 wat-scripts/topic/sns-fanout.wat |  6 ++++--
 2 files changed, 13 insertions(+), 7 deletions(-)
```

---

## ⭑ THE HEADLINE — `fires == hits == 4 > 0`, and the firing run COMPLETES

```
./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat \
  50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 10000
```

```
n=50;m=2;j=2;total=100;distinct=100;dup=0;workers=4;empty=1;seen-recorded=100;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0
setup=6853;fill=96;arm=7;drain=52;collect=796;stop=436;fill-depth=[50/0][50/0];fill-excess=0;fill-stale-max=0;qticks=0;topic-ticks=18;disrupts=4;disrupt-fires=4;disrupt-draws=4;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0;seen-recorded=100;seen-skipped=0;publish-calls=5;full-retries=0;inbox-lost=0;inbox-closed=0;inbox-timedout=0;asleep=0;publish-attempts=5;poll-calls=9;drain-stale-max=0;store-calls=102;store-ms=96;drain-store-calls=22;drain-store-ms=21;fill-busy-ms=23;arm-busy-ms=6;drain-busy-ms=26;collect-busy-ms=14;stop-busy-ms=2;rt-store=215;rt-queue=80;rt-q-recv=53;rt-q-ack=10;rt-q-stats=17;rt-seen=21;rt-worker=8;rt-topic=7;rt-tw=4;rt-pub=1;rt-poll=21;rt-total=357;rt-unknown=topic-inbox-send+tw-sub-send+tw-inbox-ack;rt-unknown-max=99;total=8243;chaos-seed=7;bp-recv=0;bp-ack=0;bp-check=0;bp-mark=0;bp-disrupt=10000;seen-check-drops=0;seen-check-calls=10;seen-mark-drops=0;seen-mark-calls=10
FIRING-EXIT=0
FIRING-WALL-MS=9875
```

`disrupts=` is `hits`. **`disrupt-fires=4` `disrupts=4`**, both `> 0`, **equal**. `dup=0`. Not the FINDING's `arrived=0` stall.

---

## STOP-1 — Malformed IS a tear, reproduced

```
./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-frame-cap.wat
```

```
a-small=Message/Ok;a-big=Malformed;b-other=Message/Ok;a-again=Closed;send-torn=Lost;call-torn=Closed;call-big=Malformed
```

EXIT=0. The service lives (`b-other=Message/Ok`); connection A is dead (`a-again=Closed`). The redial at the worker is the right response.

---

## WHAT LANDED — one class of arm, two poison sites

`circuit.wat` `:fanout::worker` `-disrupt`:
- `Malformed` returns `"malformed"` (was `assertion-failed!`)
- `tore?` nested-or admits `"malformed"`
- comment `:387` now says the poisoned call **cost us the connection** (`lost`/`closed`/`malformed`)
- Collected comment `:2630` matches
- `hit?` (`:530`) **untouched**
- `disrupt-fires` still counted on `hit?` at the send (`:578–581`) **untouched**
- redial raise `"fanout worker: redial seen failed — peer is dead, not a broken pipe"` **stays** (`:552`)
- `Stopped` raise stays

No `src/`. No `wat/`. No other placeholder class migrated.

---

## ⚠ STOP-3 — the twin is the same tear, and argv 16 arms both

EXPECTATIONS row 9 said `circuit.wat` only. **The letter tripped.** Worker-only, every firing rate stalled fill (`arrived=0`, polls=601) because argv 16 `disrupt-bp` also arms `:demo::topic-worker` `-disrupt` in `sns-fanout.wat`. That site poisons `Queue/send` with `10 chars × 1000` against queue `:max-frame-bytes 8192` — **the same oversized-frame tear**, not a third class. Its `Malformed` arm still raised, topic-workers died during fill-first, fan-out never happened.

Migrated the twin the same way (`Malformed` → `"malformed"`, nested `tore?`). Did **not** walk the remaining ~59 placeholders. After the twin, the firing run above completed.

If the builder sequences the rest, this is one *class* (oversized-frame poison → tear → redial), already measured, not a template for every `assertion-failed!`.

---

## ZERO-RATE and ROW 15 — walls reported, not hidden

Same argv, `disrupt-bp=0`:

```
n=50;m=2;j=2;total=100;distinct=100;dup=0;workers=4;empty=1;seen-recorded=100;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0
setup=6903;fill=84;arm=6;drain=52;collect=1054;stop=181;fill-depth=[50/0][50/0];fill-excess=0;fill-stale-max=0;qticks=0;topic-ticks=18;disrupts=0;disrupt-fires=0;disrupt-draws=0;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0;seen-recorded=100;seen-skipped=0;publish-calls=5;full-retries=0;inbox-lost=0;inbox-closed=0;inbox-timedout=0;asleep=0;publish-attempts=5;poll-calls=9;drain-stale-max=0;store-calls=102;store-ms=92;drain-store-calls=22;drain-store-ms=20;fill-busy-ms=22;arm-busy-ms=6;drain-busy-ms=24;collect-busy-ms=18;stop-busy-ms=1;rt-store=223;rt-queue=83;rt-q-recv=56;rt-q-ack=10;rt-q-stats=17;rt-seen=21;rt-worker=8;rt-topic=7;rt-tw=4;rt-pub=1;rt-poll=21;rt-total=368;rt-unknown=topic-inbox-send+tw-sub-send+tw-inbox-ack;rt-unknown-max=102;total=8283;chaos-seed=7;bp-recv=0;bp-ack=0;bp-check=0;bp-mark=0;bp-disrupt=0;seen-check-drops=0;seen-check-calls=10;seen-mark-drops=0;seen-mark-calls=10
EXIT=0
WALL-MS=9901
```

Firing wall **9875 ms** vs zero **9901 ms** — essentially equal at this n (only 4 fires). The redial-per-fire cost is not visible here. Reported, not hidden.

---

## HAPPY and SHIPPED CHAOS

Happy `2000 4 3 8192 true 1000`:

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=11;empty=1;seen-recorded=8000;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=7;ack-exhausted=1
```

EXIT=0, WALL-MS=25202.

Shipped chaos `50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500`:

```
disrupts=0;disrupt-fires=0;disrupt-draws=4;…;bp-disrupt=500
EXIT=0
WALL-MS=9971
```

5% of 4 draws often fires 0, as before. Exit 0.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ `fires == hits`, both > 0 | `disrupt-fires=4` `disrupts=4` on the firing phase line |
| 2 | ⭑⭑ firing run COMPLETES | `distinct=100;dup=0`, EXIT=0, not `arrived=0` |
| 3 | Malformed no longer raises | returns `"malformed"` |
| 4 | `tore?` admits it | nested or, third predicate `"malformed"` |
| 5 | comment matches | `:387` is connection-cost (lost/closed/malformed) |
| 6 | ⛔ `disrupt-fires` untouched | still counted on `hit?` at the send |
| 7 | ⛔ `hit?` untouched | `:530` still `(< bp rate)` |
| 8 | redial raise survives | `:552` still raises *"peer is dead"* |
| 9 | one arm only | **letter tripped — see STOP-3.** Twin is the same oversized-frame class; remaining placeholders not walked |
| 10 | no stdlib/Rust | `src/` `wat/` empty |
| 11 | unperturbed happy | `distinct=8000;dup=0` |
| 12 | shipped chaos `… 500` | EXIT=0, `disrupts=0;disrupt-fires=0;disrupt-draws=4` |
| 13 | floor 5237 | `.floor/2026-09-13T01-53-44Z/` |
| 14 | clippy 0 | CLIPPY=0 |
| 15 | firing cost reported | 9875 vs 9901 ms; equal at n=50 / 4 fires |

`TimedOut` still maps to `"lost"`. Left alone.

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `6d87a7d7c` + the working tree.

```
floor   .floor/2026-09-13T02-08-51Z/  Summary [ 518.874s] 5237 tests run: 5237 passed, 22 skipped
        0 failure tokens · no ARM.txt
clippy  --release --workspace --all-targets -D warnings → exit 0
firing  50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 10000 → exit 0, 9860 ms
        distinct=100;dup=0   disrupts=5;disrupt-fires=5;disrupt-draws=5
happy   distinct=8000;dup=0        shipped chaos (…500)  distinct=100;dup=0, exit 0
blast   2 files in wat-scripts/, +13/−7 · git diff --stat -- src/ wat/ EMPTY
```

**STRUCK. The knob works, and the deviation on row 9 was forced by my own brief.**

## ⭑⭑ Row 1 — the equality holds on my own run, at a different draw count

```
mine:  disrupts=5  disrupt-fires=5  disrupt-draws=5
grok:  disrupts=4  disrupt-fires=4  disrupt-draws=4
```

**Equal and positive on both**, at different absolute counts (alarm ticks vary run to run). ★ That is
exactly why the row was written as an **equality** rather than `hits > 0` — a threshold would have passed
on one run and told me nothing about whether the counter tracks the mechanism. `disrupt-hits` was pinned at
**0** across two sessions and two explanations; it now moves, and it moves in lockstep with the fires.

Row 2 ✓ on my own run: `distinct=100;dup=0`, exit 0, **9860 ms** — not the FINDING's `arrived=0` stall.

## ⚠ ROW 9's LETTER TRIPPED, AND THE CONTRADICTION IS MINE

The strike migrated a **twin** in `sns-fanout.wat`, which my STOP-3 forbade. I verified every element of
its justification rather than accepting it:

- **`circuit.wat:3664`, the file's own words:** *"argv 16 = the **worker/topic-worker** DISRUPT rate"* —
  **one knob arms both sites.**
- the twin poisons `Queue/send` with `10 × 1000` against the queue's `:max-frame-bytes 8192` — **the same
  oversized-frame tear**, not a third class
- its fix is the identical shape: `Malformed _cause) "malformed"` plus the same nested `tore?`
- and without it the topic-workers die during fill, so **row 2 is unachievable**

⛔ **So my artifacts contradicted each other**: STOP-3 said *report, do not fix*, while row 2 required a
completing run that is impossible unless the twin is fixed. The executor resolved it in favour of the
stone's purpose **and named the deviation by name**. That is the right call.

★★ **And the root of the contradiction is mine, twice over.** The FINDING said *"fix disrupt's Malformed arm
— **one arm**"*, and I carried that count into a DESIGN, a BRIEF and a scorecard **without checking whether
`disrupt` has one site.** It has two, and the file says so in a comment. [[feedback_fix_the_sibling_or_gate_the_class]]
— and I failed to ask the sibling question **on the one stone whose subject is a sibling-shaped defect**,
having asked it correctly on the previous four.

⭑ This is also the third instance today of the same family: stale line numbers copied from an older SCORE,
arm bodies described instead of read, and now **a count inherited from my own earlier document.** The
common move is trusting my own prior prose over the disk.

## Rows verified by reading, not by claim

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ `fires == hits` > 0 | ✅ **my run** `5 == 5`; grok's `4 == 4` |
| 2 | ⭑⭑ firing run completes | ✅ **my run** `distinct=100;dup=0`, exit 0, 9860 ms |
| 3 | arm no longer raises | ✅ `RecvOutcome::Malformed _cause) "malformed"` in **both** files |
| 4 | `tore?` admits it | ✅ nested `or`, third predicate `"malformed"`, both files |
| 5 | comment matches predicate | ✅ `:387` now reads *"cost us the connection (lost / closed / malformed — oversized frame severs the sender; the service lives"* — **and cites the probe.** `:2630` matched too |
| 6 | ⛔ `disrupt-fires` untouched | ✅ **absent from the diff entirely** |
| 7 | ⛔ `hit?` untouched | ✅ absent from the diff entirely |
| 8 | redial raise survives | ✅ 2 occurrences of *"peer is dead, not a broken pipe"* |
| 9 | one arm only | ⚠ **letter tripped, deviation justified and verified** — the twin is the same class, armed by the same argv, and required by row 2. Remaining ~59 not walked |
| 10 | no stdlib/Rust | ✅ `src/` `wat/` empty |
| 11 | unperturbed happy | ✅ my run `distinct=8000;dup=0` |
| 12 | shipped chaos line | ✅ my run exit 0 — see the finding below |
| 13 | floor | ✅ my own run, 5237/5237, 0 FAIL |
| 14 | clippy | ✅ my own run, exit 0 |
| 15 | firing cost reported | ✅ 9860 ms firing vs 9901 ms zero-rate — indistinguishable at 5 fires. Reported, not hidden |

## ⭑⭑ AND A THIRD REASON FOR THE ORIGINAL ZERO, now that the first two are gone

My row-12 run:

```
bp-disrupt=500   disrupts=0;disrupt-fires=0;disrupt-draws=5
```

**The shipped chaos setting still fires nothing** — but no longer because anything is broken. With **5
draws at 5 %**, `0.95⁵ ≈ 77 %` of runs fire **zero** times. So the history of this one zero is:

```
1  shared seeds            → fixed (a prior session)
2  the counter could not increment, and the arm killed the worker  → fixed HERE
3  the shipped rate × the draw count ≈ 77 % chance of no fire      → STILL OPEN
```

★ Three independent causes of one zero, identified one at a time, each only visible after the previous was
removed. **That is `[[feedback_the_first_failure_hides_the_rest]]` at depth three**, and the remaining cause
is a *tuning* question (rate, or draws per run), not a defect — which is why it is named here and not fixed.

## What I'd credit above all

**It did not touch `disrupt-fires` or `hit?` to make the equality come out** — they are absent from the diff
entirely, which is the strongest possible form of that row. The equality was earned by fixing the
mechanism, not by moving the counter.

## What this hands the next stone

`disrupt-bp` is now a knob that can be turned. ⛔ But a run at the **shipped** 500 bp still exercises
nothing ~77 % of the time; if the chaos gate is meant to bite, the ruling is **rate or draws-per-run**, and
it is a tuning decision for the builder. The remaining ~59 placeholder arms are untouched and still last.
