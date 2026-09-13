# SCORE — the store can fail

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `004457a5d` (DRAWN). Did not commit.

```
     Summary [ 528.595s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-13T01-06-26Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` exit **0**.
`cargo nextest run --release --no-run` exit **0**.
`git diff --stat -- src/ wat/` is **EMPTY**.

---

## ⭑ THE HEADLINE — the six arms fire, and they do not raise

BRIEF/EXPECTATIONS said those arms `assertion-failed!` and the tier would die. **The disk disagrees.** `queue.send` Lost/Closed/TimedOut (`sqs.wat:780/814/845`) Continue with `Accepted 0` and redial. `queue.ack` siblings Continue with `Ack Ok`. Followed the disk.

```
./target/release/wat wat-scripts/scratch-pad/probe-store-can-fail.wat
```

```
passthrough send=Accepted 1;elapsed-ms=3;real-rows=1
die send=Accepted 0;elapsed-ms=2;real-rows=1
timedout send=Accepted 0;elapsed-ms=10003;real-rows=1;drops-fired=1
```

| arm | knob | observed (verbatim) | write landed |
|---|---|---|---|
| store put TimedOut `:845` | `drop-reply-bp=10000` | `Accepted 0` after **10003 ms**; `drops-fired=1` | `real-rows=1` |
| store put Lost (or Closed) `:780`/`:814` | `die-bp=10000` | `Accepted 0` after **2 ms** | `real-rows=1` |
| rates 0 | — | `Accepted 1` after 3 ms | `real-rows=1` |

The unknowable write is genuine: the real store has the row, the caller got no success reply.

---

## WHAT LANDED

`wat-scripts/query/faulting-store.wat` — `:satisfies :wat::query::Store`. Six forwards: `ensure-schema put delete count-index scan scan-index`. Put/delete forward **then** roll. `drops-fired` incremented where `None` is produced; `dies-fired` where `Outcome::Stop` with `None` is produced. ensure-schema/scan/scan-index/count-index always pass through (queue init would hang if ensure-schema were faulted at 10000 bp).

No `sqs.wat` edit. No stdlib. No Rust. Queue takes an address; the probe points it at the proxy.

---

## STOP-5 — a caller-side deadline works

Generated `Queue/send` is 10 000 ms. The store TimedOut is also 10 000 ms, so they race and the **client** sees `TimedOut` instead of the arm's `Accepted 0`.

`call-by-deadline` on the queue peer with **20 000 ms** waits out the store timeout and still receives `Accepted 0`. That is the arm. A callee's `:deadline-ms` remains inert (DESIGN's probe); a **caller**-side deadline is real.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | TimedOut fires | `Accepted 0` at 10003 ms, `drops-fired=1` — the `:845` Continue, not a raise |
| 2 | Lost or Closed | `Accepted 0` at 2 ms — `:780`/`:814` Continue. First death is Lost; both arms reply the same |
| 3 | write landed | `real-rows=1` on the **real** mem-store after every fault |
| 4 | forward then suppress | put/delete call `Store/<op>` then roll |
| 5 | count at suppression | `drops-fired` on the `None`; `dies-fired` on the `Stop` |
| 6 | six features | all six impls; none stubbed |
| 7 | pass-through | `Accepted 1` at rates 0 |
| 8 | no stdlib/Rust | `src/` `wat/` empty |
| 9 | unfaulted happy | `distinct=8000;dup=0` — proxy **not** inserted |
| 10 | floor 5237 | `.floor/2026-09-13T01-06-26Z/` |
| 11 | clippy 0 | yes |
| 12 | 10 s respected | timedout sized; 20 s caller deadline so the arm's reply is visible |

The next stone can migrate those Continues (or not). This one only made them fire.

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `004457a5d` + the working tree.

```
floor   .floor/2026-09-13T01-18-05Z/  Summary [ 520.650s] 5237 tests run: 5237 passed, 22 skipped
        0 failure tokens · no ARM.txt
clippy  --release --workspace --all-targets -D warnings → exit 0
happy   distinct=8000;dup=0   (proxy NOT inserted)
tree    ZERO modified files · git diff --stat -- src/ wat/ EMPTY
```

**STRUCK. All rows pass on my instruments — and the stone refuted my own EXPECTATIONS, correctly.**

## ⛔⛔ I WAS WRONG ABOUT THE SIX ARMS, IN THREE DOCUMENTS

DESIGN, BRIEF **and** EXPECTATIONS all said those arms `assertion-failed!`, that *"the tier dies"*, and
that they *"are among the 61 placeholders"*. **All three statements are false.** I read the arms:

```
sqs.wat:780  ((RecvOutcome::Lost _cause)
               (let [fresh (match (connect (Record/store-addr …)) ((Connected p) p) …)
                     s'    (State … :store fresh …)]
                 … Outcome::Continue …))
```

They **redial a fresh store peer and Continue.** The only `assertion-failed!` is on the *redial itself*
failing (*"peer is dead, not a broken pipe"*). **They were already migrated.** The strike followed the disk
over my brief, which is the correct precedence and the third time this campaign it has been exercised.

★ **Where my error came from is specific and worth naming:** earlier today I *printed those exact lines*
to correct my own stale line numbers — and read only the **arm heads**. I then described the bodies from
the FINDING's placeholder framing. **I had the file open and described it instead of reading it.**
[[feedback_cite_an_exemplar_do_not_describe_one]], with the citation already on my screen.

## ⭑⭑ The headline holds, on my own run

```
passthrough send=Accepted 1;elapsed-ms=3;real-rows=1
die         send=Accepted 0;elapsed-ms=2;real-rows=1
timedout    send=Accepted 0;elapsed-ms=10003;real-rows=1;drops-fired=1
```

Byte-identical to the strike's, true exit 0. **The unknowable write is genuine**: `real-rows=1` on the
real store after every fault, and the caller got no success reply. `TimedOut` at **10003 ms** matches the
10 000 ms deadline I probed; `die` lands in **2 ms** because a socket close is an event.

Three arms that had never executed in the life of this campaign now have captured evidence.

## ★★★ AND THE TIER DOES DIE — one level up, for a different reason

My prediction was right in outcome and **wrong in mechanism**, which matters because the mechanism is what
a fix would target. The queue's arm survives; its **caller** does not:

```
sns-fanout.wat:847   ((Accepted n) (if (= n 1) nil (assertion-failed! "send-one not fully accepted")))
sns-fanout.wat:118   ((Accepted accepted) … Outcome::Continue …)      ← survives, and UNDER-REPORTS
```

So the measured §2d chain, end to end:

1. the store call fails → the queue **redials and returns `Accepted 0`** (no raise)
2. **the write landed anyway** — `real-rows=1`
3. caller `:118` treats 0 as *"none admitted"* and continues — **a row is in the store and counted as not
   sent**
4. caller `:847` (`send-one`) **raises** on `n ≠ 1`

⭑ **That is the input the `:Unknown` A/B ruling actually needs**, and it is sharper than the question I
drew: the queue is already robust; the *reporting* is what lies. `Accepted 0` after a landed write is the
defect, not the redial.

## Rows 4 and 5 — the two rejection criteria

**Row 4 ✅ forward-then-suppress, not skip.** `forwarded` is bound by calling `Store/put real req` **before**
the roll (`let` is sequential), and no path produces `None` without having forwarded. The write really lands;
the condition under test is the real one.

**Row 5 — the letter is not met and the invariant is, and I am passing it deliberately.** The counters
increment in `rec'`, built *before* the branch, so not literally at the `None`. But `drop?` is **one
boolean driving both** the increment and the `None` — the counter cannot move unless the suppression
happens, which is the property `disrupt-hits` violates and the reason the row exists.

★ **My brief asked for a LOCATION in a language where state must be built before the branch.** That is my
defect, not the strike's: [[feedback_gate_on_the_property_not_the_path]] — *state what must be true, never
where to look.* The property is "no increment without a suppression"; I wrote "count at the suppression
site". The executor honoured the property.

## Row 6 — six features, and my census was wrong first

My first pattern (`^ +\(<feature> \[`) found **5**, missing `ensure-schema` — which sits at `:65` as
`  [(ensure-schema` (a `[` before the `(`). An indentation-agnostic re-run finds **all six**, none stubbed.
Fourth time today a form assumption in one of my greps produced a wrong count, and the fourth time the
positive control caught it before I quoted it.

## ★ STOP-5 fired usefully — a CALLER-side deadline is real

The generated `Queue/send` is also 10 000 ms, so it races the store's 10 000 and the client sees its own
`TimedOut` instead of the arm's `Accepted 0`. `call-by-deadline` on the queue peer at **20 000 ms** waits it
out and receives `Accepted 0`. So: **a callee's `:deadline-ms` is inert (my probe); a caller-side deadline
works.** That is the missing half of the DESIGN's finding and it was reported, not assumed.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ `TimedOut` fires | ✅ my run: `Accepted 0` at **10003 ms**, `drops-fired=1` |
| 2 | ⭑⭑ `Lost`/`Closed` fires | ✅ my run: `Accepted 0` at **2 ms** |
| 3 | ⭑ the write landed | ✅ `real-rows=1` after every fault |
| 4 | forward then suppress | ✅ read the diff — forward precedes the roll, no skip path |
| 5 | counted at suppression | ✅ **on the invariant**, not the letter — one boolean drives both (my brief's defect, not the strike's) |
| 6 | six features | ✅ all six; my first census said 5 and was wrong |
| 7 | pass-through | ✅ `Accepted 1`, and at rates 0 no RNG is consumed at all |
| 8 | no stdlib/Rust | ✅ **zero** modified files |
| 9 | unfaulted happy | ✅ `distinct=8000;dup=0`, proxy not inserted |
| 10 | floor | ✅ my own run, 5237/5237, 0 FAIL |
| 11 | clippy | ✅ my own run, exit 0 |
| 12 | 10 s cost respected | ✅ small-n probe; no command resembling a hang |

## What I'd credit above all

**It followed the disk over three of my documents.** My EXPECTATIONS pre-committed to "the tier dies
because the arms raise" — a claim that would have let a strike report a raise it never saw. Instead it read
the arms, said they Continue, and produced the real behaviour. ⭑ An executor that contradicts the brief
**with a citation** is the whole point of grading against the ground.

## What this hands the next stone

The `:Unknown` A/B ruling now has its measurement — **and the question has moved.** The queue does not need
`:Unknown` to survive; it already redials. What needs deciding is **what `Accepted n` should say when the
write may have landed**, given one caller under-reports it and another raises on it. ⛔ That is a
`SendResponse` contract question, squarely the builder's, and it is **not** the same question the earlier
SCORE asked.
