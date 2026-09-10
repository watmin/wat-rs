# SCORE — the fill poller gets what the drain poller got

**SCORED.** Executor: claude, 2026-09-10, branch `sns-sqs`, HEAD `7eca4e4f1`. Did not commit.
**One functional file modified — `wat-scripts/fanout/circuit.wat` — and nothing else.** No `wat/`, no
`src/`, no `sqs.wat`, no `sns-fanout.wat`, no `.config/nextest.toml`, no test file. `require!`,
`sweep-drained?`, `sweep-unread?`, `snapshot-str`, `sweep-acks`, `poll-until-drained*` all untouched.

```
     Summary [ 483.727s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-10T17-45-28Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**, **zero**
`FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. Run on the **final** tree, after every
timing run, box otherwise quiet. **Nothing was re-run to make anything go away.** `5237` unchanged — no
test added, removed or renamed. The 7 SLOW warnings are the same seven the drain stone's floor carried,
and the two circuit tests inside them sat at **19.856 / 20.269 s** here against **19.366 / 19.959 s** on
the earlier floor of the same tree and 19.4 / 20.2 s on the drain stone's — no shift.

⚠ **Two floors were run, and both are reported.** The first (`.floor/2026-09-10T17-35-19Z/`,
`Summary [ 485.338s] 5237 tests run: 5237 passed (7 slow), 22 skipped`) preceded a one-line **comment**
correction — a stale line citation inside `poll-until-filled*`'s header. That line is parsed by the
corpus gate like any other, so the floor was run again rather than letting a green describe a tree that
no longer existed. Both are green; the one quoted above is the final tree's.

**⛔ STOP-3 FIRED, and it is the deliverable's most important line.** The sibling audit found not one
more instance but **three**, and together they are a *class* — see "THE SIBLING AUDIT" below. Nothing
was fixed. The ruling is the builder's.

**⛔ AND A RED WAS CAPTURED TODAY, in code this stone did not touch.** `2000 4 3 8192 true 0`
(inbox visibility at its 200 ms default) exited 2 in the **drain** poller. Captured whole, arm named,
not re-run, not dispositioned — see "THE CAPTURED RED" below. It is the evidence for audit item (i).

---

## ⭑ THE HEADLINE — the overshoot is real, it is 40, and it now passes *and prints*

`fill-excess=` is a new field on the report line. Under **4-way self-contention** it came out **40 on
two of four runs** — `2000 4 3 8192 true 1000`, all four `rc=0`, `distinct=8000 dup=0`:

| run | rc | `fill` | `fill-depth` | `fill-excess` | `fill-stale-max` |
|---|---|---|---|---|---|
| conc1 | **0** | 5841 | `[2010/0][2010/0][2010/0][2010/0]` | **40** | 0 |
| conc2 | 0 | 5786 | `[2000/0][2000/0][2000/0][2000/0]` | 0 | 0 |
| conc3 | 0 | 5960 | `[2000/0][2000/0][2000/0][2000/0]` | 0 | 0 |
| conc4 | **0** | 5795 | `[2010/0][2010/0][2010/0][2010/0]` | **40** | 0 |

⛔ **`[2010/0][2010/0][2010/0][2010/0]` against `want=2000` is the captured red's snapshot, character
for character** (`43efddb6a`: `filled-never: last=[2010/0][2010/0][2010/0][2010/0] outbox=0 want=2000
attempts=8000 elapsed=356669`). The same state now exits **0** in 5.8 s, with the excess on the line.
Under the old `=` these two runs were the 356 s, 40 000-round-trip failure. **This is the stone's whole
claim, reproduced twice on this box.**

⚠ **And a second, unasked-for observation, offered as data and not as a mechanism:**
`2000 4 3 8192 true 1000 0 0 0 50` — the same run with `inbox-vis-ms` at **50 ms** instead of 1000 —
reported `fill-depth=[2020/0][2020/0][2020/0][2020/0]`, `fill-excess=80` (20 per queue), `rc=0`.
Short inbox visibility, larger excess. That is a
correlation on one run. **It is not a mechanism and this SCORE does not claim one** (see row 15).

---

## ⭑⭑ THE CONSTANTS, AND THE HONEST STATEMENT OF WHERE THEY CAME FROM

### `K = :fanout::fill-stale-polls = 600` polls

**Measured `fill-stale-max = 0` on every passing run — ten of them.** `2000 4 3 8192 true 1000` ×3
alone and ×4 concurrent, `2000 4 3 8192 true 0`, `50 2 2 8192 true 0` ×2. Not one poll without an
arrival, anywhere.

⚠ **AND I AM NOT ALLOWED TO CALL THAT A MEASUREMENT OF K, because it is measured over ~3.6 polls.**
`(rt-poll − poll-calls) / (m+1)` = `(358 − 340) / 5` ≈ **3.6 polls per run**. The fill poller barely
runs: the topic inbox is capped at 64 (`circuit.wat:2664`) and that **backpressure couples publishing
to fan-out** — a publisher cannot get far ahead of the workers draining the inbox, so by the time
`join-publishers` returns the fan-out is essentially finished and the poller finds the queues already
full. `fill-stale-max = 0` says *"no silence was sampled"*, not *"no silence is possible"*.

★ That is also the mechanism behind the FINDING's whole shape: this poller's give-up path is close to
unreachable on a healthy run, which is why the `=` defect cost 356 s **only** where the completion test
was *unsatisfiable* rather than merely slow.

**So K is sized against the longest LEGITIMATE silence the fill path can have**, which is a property of
the scenario, not of the poller: a fan-out batch that is refused or lost is re-presented only after the
**inbox visibility timeout** expires — `:fanout::inbox-vis-default-ns` = **200 ms** (`circuit.wat:2604`),
**1000 ms** as the standard CLI run passes it — plus the topic worker's own receive wait. Call it
~1.25 s at the 1000 ms setting.

**Measured poll cost: 27.4 ms/poll** — 602 polls in 16 499 ms, taken from the witnessed stall below
(5 ms sleep plus `m+1` `Queue/stats` crossings). Therefore:

| | |
|---|---|
| K in polls | 600 |
| K in wall time, measured | **~16.5 s** |
| vs the ~1.25 s legitimate gap at `inbox-vis-ms=1000` | **13×** |
| vs the 200 ms default | **82×** |
| vs the largest streak yet observed (0) | unbounded, and *that is the weak part* |

⭑ **And 600 is deliberately the same number `:fanout::drain-stale-polls` uses.** Two pollers on one box
with two different K values is precisely the asymmetry this stone exists to remove. ⚠ A caller who sets
`inbox-vis-ms` anywhere near 16 s must revisit it; the ceiling arm still bounds that case.

⚠ **The drain stone's lesson was applied and it changed nothing here, which is itself worth stating.**
That stone measured `drain-stale-max=197` on a passing run against a first guess of 200 — three polls
from a false red. I measured before choosing. The measurement came back 0 and was *uninformative*, so
the number rests on the visibility-timeout argument plus the deliberate symmetry, and I have labelled
which part is measured and which is reasoned rather than presenting the pair as one thing.

### `CEILING = :fanout::fill-ceiling-ms(pairs) = 30000 + 12 × pairs` ms

The unconditional backstop for the one world the stall arm cannot see: a system that keeps delivering
(redelivery churn; or, under `fill-first? = false`, a consumer draining as fast as the fan-out fills)
and never reaches `n`. Bounded on both sides:

- **below, by the work.** Measured healthy `fill` phases — which include publish *and* fan-out *and*
  the poll, so they **over**-state the poller's share: 2586 / 2601 / 2636 ms for 8000 pairs, 5786–5960
  under 4-way contention, 81 / 84 ms for 100 pairs. That is 0.32–0.75 ms/pair; **12 ms/pair is 16–37×
  it.** The 30 s floor is ~24× the ~1.25 s legitimate no-arrival silence and 150× the 200 ms default.
- **above, by the runner** — a ceiling-hitting fill must still print its verdict inside nextest's kill
  or the arm is destroyed (arc 278's empty TIMEOUT). 8000 pairs → **126 s**, which fits the `r2_drop_*`
  90/180 s override and does **not** fit the 15/30 s default.
- **above K's wall window** at every scale, or a genuine stall would be mislabelled a timeout: K's
  measured window is ~16.5 s, under the 30 s floor. Measured, not projected — the stall arm fired at
  601 polls / 16 499 ms.

⚠ **NAMED, NOT ASSUMED: no test in the floor reaches this code at all.** Every `:user::*` fixture
passes `fill-first? = false` (`circuit.wat:3133-3172`), and for that mode the caller returns
`(Tuple "" 0 0)` without polling. `poll-until-filled*` is reachable **only from the CLI**. So the
runner constraint above is a warning for whoever flips that flag, not a constraint this tree tests —
and rows 10/11/12 below are *regression* evidence, **not coverage evidence**, which I say again there.

---

## THE ROWS

| # | what | result |
|---|---|---|
| 1 | ⛔ **THE CHECK CAN STILL GO RED** | ✅ **PASS — EXECUTED, not reasoned.** `circuit.wat 50 2 2 32 true 0` → exit **2**, `filled-stalled`, verbatim below. The ceiling arm was witnessed separately. Argument in full below |
| 2 | ⛔ **the unsatisfiable test is gone** | ✅ **PASS.** `(:wat::i64::>= (:wat::core::first d) n)` at `circuit.wat:1422`. Witnessed live: two 4-way runs completed with `visible=2010` per queue where the old `=` could never have completed |
| 3 | ⛔ **the overshoot is REPORTED, not swallowed** | ✅ **PASS, with a NON-ZERO witness.** `fill-excess=40` on two of four concurrent runs, `fill-excess=80` at `inbox-vis-ms=50`, `fill-excess=0` on every clean run. It is on the report line, not in a comment |
| 4 | ⛔ **the happy path is unchanged** | ✅ **PASS ×3 interleaved.** `distinct=8000`, `dup=0`, `empty=1`, `rc=0` on all three. `fill` **2610.3 ms before → 2607.7 ms after, −0.10 %** — unreadable. Table below |
| 5 | ★ **progress-bounded, not attempt-bounded** | ✅ **PASS.** `left`/`total` are gone from `poll-until-filled*`; the give-up arms are `(>= stale' (fill-stale-polls))` and `(>= el ceiling-ms)`. `grep -n 'left\b' ` over the file returns only *other* functions (`join-publishers*`, `poll-until-visible-zero*`, `publish-until-accepted!*` — see the audit) |
| 6 | ★ **three verdicts, distinguishable** | ✅ **PASS.** `filled-stalled` and `filled-timeout` are distinct strings, each naming the signal (`arrived`), `polls`, `elapsed`, `want`, the snapshot and the outbox, and the timeout additionally `stale`/`stale-max`/`ceiling`. **Both give-up arms were EXECUTED**, not read off a `format`. ⚠ `filled-unread` was **not** executed — this harness has no knob for an unreadable stats reply; it is unchanged in position (still tested first, still outranking both) and only its fields grew. Stated as an untested arm, not claimed |
| 7 | ★ **no new crossings per poll** | ✅ **PASS.** `rts' = rts + (count qclients) + 1` — byte-identical. `sweep-arrived` folds the sweep already taken; `sweep-excess` folds `fill-sweep`, which already existed. Live: `rt-poll` 358/368/348 before vs 358/348/358 after; `rt-total` mean 11 264 → 11 257 (**−0.06 %**) |
| 8 | ★ **the sibling audit is reported** | ✅ **REPORTED — and ⛔ STOP-3 FIRED.** Two negatives and **three** positives, all four named sites plus two I was not asked about. Section below |
| 9 | **the drain stall gate still fires** | ✅ **PASS.** `circuit.wat 5 1 0 32 false 0` → exit **2**, `drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601 elapsed=13129`. 601 polls / 13 129 ms — the drain stone measured 601 / 13 172. Undisturbed |
| 10 | **chaos still passes** | ✅ **PASS.** `Summary [ 33.042s] 7 tests run: 7 passed, 5252 skipped`. ⚠ It does **not** exercise the new code (`fill-first? = false`) |
| 11 | **the corpus loads** | ✅ **PASS.** `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` PASS, inside the green floor. No file added to `wat-scripts/` |
| 12 | **the floor holds** | ✅ **PASS.** Summary at the top of this file, verbatim. 0 FAIL / 0 TIMEOUT / 0 ABORT / 0 SIGSEGV / 0 TRY, exit 0, no `ARM.txt`. ⚠ It does not cover the new code |
| 13 | **blast radius** | ✅ **PASS.** `git status --porcelain` → `M wat-scripts/fanout/circuit.wat` plus this SCORE. Nothing forced elsewhere |
| 14 | ★★ **`Σ(visible+unacked+acks)` is monotone** | ✅ **HOLDS — and for a STRONGER reason than the design gave.** STOP-2 did **not** fire. Citations below |
| 15 | ⚠ **no speedup; the overshoot is not explained** | ✅ **honoured, and the `total` delta is disclosed rather than banked.** Below |

---

## ⛔ ROW 1 — HOW I CONVINCED MYSELF THE CHECK CAN STILL GO RED

Making a check load-tolerant is exactly how a check becomes unfailable, so an argument alone was not
acceptable. Three, in increasing order of weight.

**1. The structure.** Exactly one path in `poll-until-filled*` returns `""`, and it is the completion
test `(and (sweep-filled? sweep n) (= box 0))`. Every other path returns a non-empty string. There is
no arm that says "give up and call it fine". The change to `sweep-filled?` **weakened one conjunct and
left the other alone**: `visible >= n` replaced `visible = n`, and `unacked = 0` is still an equality,
because an in-flight row at fill time is a genuinely incomplete fill and nothing can push that count
above zero except a claim.

**2. Both give-ups are unconditional, and the second does not depend on progress at all.** The stall arm
fires at K consecutive no-arrival polls. The ceiling arm fires at `elapsed >= ceiling-ms` **regardless
of progress**, so the one world the stall arm cannot see is still bounded and still red.

**3. I built the failure and watched it fail.** The drain stone's witness was `j=0` — no consumers, so
nothing can ever ack. **The fill equivalent is not `j=0`**: with nothing consuming, a fill *succeeds*.
The fill equivalent is **a queue that cannot hold what is wanted**: `sub-cap` below `n`, with
`fill-first? = true` so the workers are armed only *after* the fill and nothing drains. Then the sub
queues fill to capacity, `Queue/send` replies `Accepted 0` and bumps `sends-refused`
(`sqs.wat:487-521`), the topic worker's `ok` collapses to 0 so the inbox entry is never acked, and the
whole system settles: arrivals stop, and `Σ(visible+unacked+acks)` plateaus.

`./target/release/wat wat-scripts/fanout/circuit.wat 50 2 2 32 true 0` — **rc = 2**, wall 24.7 s:

```
#wat.kernel/AssertionFailure {:thread "main" :message "filled-stalled: no arrival progress in 600
  polls; last=[31/0][31/0] outbox=19 want=50 arrived=62 polls=602 elapsed=16499"
  :location #wat.kernel/Location {:file "wat-scripts/fanout/circuit.wat" :line 2816 :col 14}
  :frames [ … :symbol ":fanout::require!"  … :symbol ":fanout::run-with"  … ":user::main"]}
```

`arrived=62` — 31 per queue × 2, with `acks=0` — is the whole story in one field, and it is exactly what
the old single `filled-never` verdict could never say. ⚠ Note that this scenario **also** failed before
the change (`filled-never: last=[31/0][31/0] outbox=19 want=50 attempts=100 elapsed=2347`): the point
is not that it newly fails, it is that it now fails **with the reason attached** and on a bound that a
busy box cannot trip on its own.

**And the third arm, witnessed too, because an unexecuted `format` is an unverified `format`.** No
natural run reached the ceiling, so `fill-ceiling-ms` was temporarily set to `1000 + 0 × pairs`, the
same stall scenario was run, and the file was restored and verified **byte-identical by `md5sum`
(`fc1e248dc950b1fd38e351ce8e44803f`) before anything else happened**:

```
filled-timeout: ceiling 1000ms reached with no 600-poll stall; last=[31/0][31/0] outbox=19
  want=50 arrived=62 polls=47 elapsed=1010 stale=45 stale-max=45
```

★ `stale=45 stale-max=45` against a 600-poll K is the reader's own check on the verdict's claim: it
came nowhere near a stall, so "slow, not stuck" is the honest reading — which is the entire reason
those two fields are printed rather than the wording being trusted.

**4. And a fourth, which I did not plan.** During the battery, `2000 4 3 8192 true 0` exited **2** — in
the *drain* poller. The harness's checks are demonstrably still capable of failing on this tree, in a
way nobody arranged. See below.

---

## ★★ ROW 14 — `Σ(visible + unacked + acks)` IS MONOTONE, AND THE DESIGN'S REASON WAS THE WEAKER ONE

**STOP-2 did not fire.** Verified from `wat-scripts/queue/sqs.wat`, not from the design's sentence.

**The design worried about the wrong hazard, and the code rules it out structurally.** The stated fear
was *"a redelivery that re-increments `visible` without incrementing anything else"*. **That cannot
happen, because `visible` and `unacked` are not two counters — they are one count split at `now`.**
`depth` (`sqs.wat:339-381`) issues exactly two `count-index` calls over the same `by-visible-at`
partition and returns:

```
visible = |isk in [0, now]|
unacked = |isk in [0, +inf)| - visible
```

So `visible + unacked` **is** `|isk in [0, +inf)|` — the row count for that queue. A redelivery moves a
row's `isk` from the future to the past; a claim moves it from the past to the future. Either way the
row crosses the split and **the sum is unchanged**. Double-counting across the two terms is likewise
impossible: the second term is defined by subtracting the first.

**The real question is therefore only about rows, and about `acks`:**

| event | Δ(rows) | Δ(acks) | Δ(sum) |
|---|---|---|---|
| `send` accepted (`sqs.wat:542`, the only `Store/put` of queue rows) | +take | 0 | **+** |
| `send` refused — queue at cap (`sqs.wat:487-521`) | 0 | 0 | 0 |
| `ack`, delete `Success` (`sqs.wat:1046-1073`) | −(rows actually deleted, ≤ `count ids`) | +`(count ids)` | **≥ 0** |
| `ack`, delete `Transient` → `retry-delete` (`sqs.wat:1104-1135`) | −(≤ `count ids`) | +`(count ids)` | **≥ 0** |
| `ack`, delete `Lost` / `Closed` / `TimedOut` (`sqs.wat:1145-1235`) | **0 — "Do not delete"** | **0** | 0 |
| redelivery / claim / visibility expiry | 0 (row crosses the split) | 0 | 0 |

The three failure arms are the ones I most expected to break it, and they are the safest of all: they
neither delete nor bump `acks`. And a **double ack** of a redelivered row deletes nothing the second
time while still adding `(count ids)` to `acks` — the sum strictly *increases*, which is the safe
direction. `Store/delete` is called on queue rows at exactly two places, `sqs.wat:1039` and
`sqs.wat:1532` (`retry-delete`), and both are inside the `ack` handler that bumps the counter.

**⛔ AND THE TWO PLATEAUS I FOUND, both of which are correct behaviour rather than blind spots:**

1. **Saturation.** `count-index` takes `limit` = `cap + 1` (`sqs.wat:469`, `mem.wat:589-606` — a
   `take lim` over the range), so `visible + unacked` **stops counting up at cap+1**. A queue at cap
   would plateau. But **a queue at cap refuses sends** (`sqs.wat:487`), so the fill genuinely is not
   progressing — the verdict `filled-stalled` is right, and this is exactly the mechanism the row-1
   witness exercises.
2. **A decrease** is reachable only if a queue process restarted and reset `Counters/acks` to 0
   (`sqs.wat:456`) while its store kept the rows. `stale'` treats *any* non-increase as no progress, so
   that surfaces as a red rather than as silence — the same conservative choice `sweep-acks` made.

⚠ And one thing I checked because the third slot of a tuple named `depth` is *not* always acks:
`depth`'s own third slot is `depth-ns`, an elapsed time (`sqs.wat:381`). `:fanout::depth-of`
(`circuit.wat:1174-1183`) does **not** read it — it reads `Stats/visible`, `Stats/unacked`,
`Stats/acks`, three separate fields off one reply. The three-term sum is the three fields I think it is.

---

## ⛔ THE SIBLING AUDIT — STOP-3 FIRED. THREE MORE, AND THEY ARE A CLASS.

### The two negatives, stated because the negative is the result that stops this recurring

**`:fanout::sweep-unread?` — CHECKED, THE SIBLING IS FINE.** It tests `(= (first d) -1)` and looks at
only the first slot, which is exactly the shape that would normally be one-sided. It is not, and the
reason is that `-1` is not a magnitude, it is a **whole-tuple sentinel**: `:fanout::depth-of`
(`circuit.wat:1174-1183`) constructs `(Tuple -1 -1 -1)` at all four of its failure arms and never sets
one slot alone. So there is no second-sided sentinel to miss. It also cannot false-positive: counts are
non-negative, and `unacked = all − visible ≥ 0` because the visible range is a prefix of the all range
under the same `take lim`. Both pollers call it in the same position, first, outranking everything.
**No change wanted.**

**`:fanout::snapshot-str` — CHECKED, THE SIBLING IS FINE.** Both pollers call it identically and it
behaves identically for both; there is no asymmetry to repair. It does drop the third column (`acks`),
printing only `[{v}/{u}]` — the format predates the third slot's arrival — but both verdicts print the
aggregate (`acks=` on the drain, `arrived=` on the fill) beside it, so nothing is unreachable, only
un-split per queue. **Named, not fixed:** it is cosmetic, symmetric, and outside the stated radius.

### ⛔ (i) `:fanout::sweep-drained?` — THE FIFTH INSTANCE, AND IT HAS A CAPTURED RED FROM TODAY

```
(:wat::core::and (:wat::core::= (:wat::core::first d) 0)
                 (:wat::core::= (:wat::core::second d) 0))     ← unacked must EQUAL 0
```

**This is the same class as the defect just fixed: an equality completion test against a quantity the
system can pin off-target.** It is not the same *repair* — draining genuinely wants zero, so `>=` is
meaningless here; the defect is that `unacked` can be stuck above zero forever. The tree already knows
this. `.config/nextest.toml`'s `r2_drop_*` override says it in its own expiry note:

> *"`fanout::queue-drained?` waits for `unacked == 0`, which is **UNSATISFIABLE after any
> redelivery**. See `docs/arc/2026/06/278-rules-engine/FINDING-unacked-is-receives-minus-acks.md`."*

**And I hit it today, at full weight, on this tree.** See "THE CAPTURED RED" below: `unacked = 10`
pinned on one of four sub queues while `visible = 0` and `outbox = 0`, `acks = 8050`.

★ **A genuine consolation, and an argument for this stone:** because the drain poller is *already*
progress-bounded, its unsatisfiable completion test cost **601 polls / 24.6 s and a named verdict**
instead of a budget-exhausted 356 s and a `drained-never`. The bounded poller converts an
unsatisfiable completion test from a hang into a diagnosis. That is what this stone just did for the
fill side too.

### ⛔ (ii) `:fanout::poll-until-visible-zero*` — A THIRD POLLER, ATTEMPT-BOUNDED, AND IT IS IN THE FLOOR

`circuit.wat:2230-2250`. Same shape the drain stone removed and this stone just removed: `left` seeded
at **4000 attempts**, one give-up verdict (`visible-never-zero: last={v}/{u} attempts={a}
elapsed={ms}`), no progress signal, no distinction between "the queue stopped emptying" and "the poller
ran out of budget". Called at `circuit.wat:3347` and `circuit.wat:3435`, inside
`:user::redelivery-is-visible` and `:user::redelivery-mid-processing` — **both of which the floor
runs** (`redelivery_is_visible_as_a_message_duplicate`, `redelivery_mid_processing_never_loses`). Its
completion test `v = 0` is reachable, so it is not *unsatisfiable*; it is attempt-bounded and
single-verdict, which is the other half of the same defect.

### ⛔ (iii) `:fanout::join-publishers*` — ATTEMPT-BOUNDED, AND ITS VERDICT CARRIES NO NUMBERS AT ALL

`circuit.wat:2204-2228`. `left` seeded at **120000** iterations on a 1 ms timer, and the give-up is:

```
(:wat::kernel::assertion-failed! "fanout: publishers never done" :wat::core::None :wat::core::None)
```

No snapshot, no `elapsed`, no attempt count, no publisher state — **strictly less than `filled-never`
carried before this stone**. It sits in `fill`, immediately before the poller this stone rebuilt, on
every run in the harness.

### The fourth item the BRIEF named: the `n × m` budget at the call site

**CHECKED, and it is the change rather than a defect.** `(:fanout::poll-until-filled qclients topic n
(:wat::i64::* n m))` at `circuit.wat:2833` is **textually unchanged**; what the number buys changed
from an attempt count to `fill-ceiling-ms(pairs)`. The parameter is renamed `attempts` → `pairs` and the
comment at the site now says so. No caller arity changed, so nothing else was forced.

### ⛔ THE RULING I AM NOT MAKING

Four sites in one file share one defect — an observer that gives up on a budget, or completes on an
equality the system can pin, and in either case emits one verdict for several worlds. Two are now
repaired (`poll-until-drained*`, `poll-until-filled*`); two are not (`poll-until-visible-zero*`,
`join-publishers*`); one is a completion test that needs a semantic decision, not an operator
(`sweep-drained?`). **At four instances this wants a gate — a lint, or a shared poller combinator —
not four separate repairs, and that ruling is the builder's.** I fixed nothing outside the fill poller.

---

## ⛔ THE CAPTURED RED — `2000 4 3 8192 true 0`, exit 2, in the drain poller

Captured whole, on the first look, from `scripts/capped.sh` output that was never piped through a
pager. **Not re-run.** It is item (i)'s evidence.

```
#wat.kernel/AssertionFailure {:thread "main" :message "drained-stalled: no delivery progress in 600
  polls; last=[0/0][0/10][0/0][0/0] outbox=0 acks=8050 polls=668 elapsed=24572;check-exhausted=0;
  mark-exhausted=0;ack-retries=6;ack-exhausted=0"
  :location #wat.kernel/Location {:file "wat-scripts/fanout/circuit.wat" :line 2844 :col 13}
  :actual nil :expected nil
  :frames [#wat.kernel/Frame {:file "wat-scripts/fanout/circuit.wat" :line 2844
                              :symbol ":fanout::require!"}
           #wat.kernel/Frame {:file "wat-scripts/fanout/circuit.wat" :line 3251
                              :symbol ":fanout::run-with"}
           #wat.kernel/Frame {:file "src/freeze.rs" :line 1521 :symbol ":user::main"}]
  :upstream-chain nil}
```

**THE EXACT ARM:** `:fanout::poll-until-drained*`'s **stall arm** —
`(:wat::core::if (:wat::i64::>= stale' (:fanout::drain-stale-polls)) …)` at `circuit.wat:1661` — not
the ceiling arm (`elapsed=24572` is under the 30 s floor), not `drained-unread`, not the completion
test. Each of those four would predict a different mechanism, and the one that fired predicts this
one: **`last=[0/0][0/10][0/0][0/0]` — sub queue 1 holds `unacked = 10` with `visible = 0`, and it never
clears**, while `outbox = 0` and `acks = 8050` (50 above the 8000 expected). Ten rows claimed and never
acked, 600 consecutive polls with Σacks frozen. That is audit item (i) firing.

**What I did NOT do, and what I will not call it.** I did not re-run it. I have not called it a flake,
timing, environmental, pre-existing, or unrelated to my change. What I can state, as measurement:

| | scenario `2000 4 3 8192 true 0` | result |
|---|---|---|
| after side | run 1 | **rc 2** — the red above |
| after side | run 2 | rc 0, `fill=2624 fill-excess=0 fill-stale-max=0` |
| before side (`circuit-before.wat`, HEAD's file) | run 1 | rc 0, `fill=2634` |
| before side | run 2 | rc 0, `fill=2617` |

1 red in 2 after-side runs, 0 in 2 before-side runs. **That is far too small a sample to attribute**,
and I am not attributing it. What I can say about mechanism, from the code: my diff touches
`sweep-filled?`, `sweep-arrived`, `sweep-excess`, `poll-until-filled*`, `poll-until-filled`, and two
report-line bindings. **None of them is on the drain path**, and `sweep-drained?` /
`poll-until-drained*` / `drain-stale-polls` / `drain-ceiling-ms` are byte-identical to HEAD.
**But there is a route worth stating out loud rather than leaving for someone else to find:** a fill
that overshoots used to raise `filled-never` and end the run *before* the drain phase; under `>=` it
passes and the run now *reaches* the drain, where `sweep-drained?`'s own pinnable equality can bite. If
that is what happened, this stone did not create the defect — it removed the earlier failure that was
hiding it. **A hypothesis, not a finding**, and exactly the kind of thing the builder's ruling on the
class should take into account.

---

## ROW 4 / ROW 15 — THE INTERLEAVED TABLE

`./target/release/wat <file> 2000 4 3 8192 true 1000`, each run under `./scripts/capped.sh --limit 4g`,
box otherwise quiet, **strictly alternating before → after → before → after → before → after** in one
loop. The "before" side is HEAD's `circuit.wat` copied verbatim to a scratch tree with `../topic` and
`../queue` symlinked (`md5 c622392fe500ffabcb85f613b9a3439c`, identical to HEAD's), so both sides run
against the **same binary** — the `.wat` is read at runtime, so only the program differs.

| pair | side | rc | setup | **fill** | arm | drain | collect | stop | total | `poll-calls` | `rt-poll` | `rt-total` | `fill-excess` | `fill-stale-max` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | before | 0 | 12514 | **2630** | 17 | 2439 | 5225 | 478 | 23306 | 340 | 358 | 11231 | — | — |
| 1 | after | 0 | 12464 | **2636** | 18 | 2435 | 5523 | 441 | 23520 | 335 | 358 | 11296 | 0 | 0 |
| 2 | before | 0 | 12450 | **2607** | 18 | 2494 | 4879 | 707 | 23159 | 350 | 368 | 11266 | — | — |
| 2 | after | 0 | 12447 | **2601** | 17 | 2454 | 4996 | 455 | 22973 | 330 | 348 | 11233 | 0 | 0 |
| 3 | before | 0 | 12409 | **2594** | 18 | 2432 | 5536 | 668 | 23659 | 330 | 348 | 11295 | — | — |
| 3 | after | 0 | 12483 | **2586** | 18 | 2466 | 4688 | 207 | 22450 | 340 | 358 | 11241 | 0 | 0 |

`distinct=8000`, `dup=0`, `empty=1`, `fill-depth=[2000/0][2000/0][2000/0][2000/0]` on all six.

| quantity | before mean | after mean | Δ |
|---|---|---|---|
| **`fill`** | 2610.3 ms | 2607.7 ms | **−0.10 %** |
| `drain` | 2455.0 ms | 2451.7 ms | −0.13 % |
| `rt-total` | 11 264 | 11 257 | −0.06 % |
| `total` | 23 374.7 ms | 22 981.0 ms | −1.68 % |

⚠ **ROW 15, AND I AM NOT BANKING THAT 1.68 %.** The BRIEF is explicit that a reported speedup means
the completion path changed. It did not, and the numbers say where the 394 ms lives: `collect`
(5225/4879/5536 → 5523/4996/4688) and `stop` (478/707/668 → 441/455/207) — **two phases this stone does
not touch, whose own run-to-run spread (657 ms and 500 ms) is larger than the difference between the
means.** `fill` itself moved −0.10 %, which is a tenth of that spread and is noise. **There is no
speedup here and I am not claiming one.** The win is entirely in the failure path, and a green run's
numbers cannot show it:

| | before | after |
|---|---|---|
| a genuine under-fill at n=2000, m=4 | 8000 attempts × 5 = **40 000** crossings, measured **356.7 s** | 601 polls × 5 = **3 005** crossings, ~**26.8 s** at that run's own measured 44.6 ms/poll → **13.3× fewer, 13.3× shorter** |
| a fill that keeps arriving and never reaches n | the same 40 000 / 356.7 s | bounded by the ceiling: 126 s at 8000 pairs, ≈ 14 100 crossings — **2.8× fewer**, and *unconditional* |
| the 2010-of-2000 overshoot | 40 000 crossings, 356.7 s, `filled-never` | **rc 0**, `fill-excess=40` on the line |

⚠ **AND THIS STONE DOES NOT EXPLAIN THE OVERSHOOT.** `>=` makes 2010 a *pass*; the mechanism is
unknown. The one new datum is the `inbox-vis-ms=50 → fill-excess=80` observation at the top, which is a
correlation on a single run and is offered as a lead, not a cause. **The ack-drift coincidence stays
unclaimed and I did not test it:** the law measured at `ae2c15622` is
`excess acked ids = 10 × ack-retries`, and this overshoot is exactly 10 per queue. ⚠ I have **no
measurement that bears on it either way.** The one figure I could be tempted to press into service —
`acks=8050` with `ack-retries=6` on the captured red — is worth nothing here, because that run was
**aborted mid-drain**: `acks` was still moving when the poller gave up, so 8050 is not a final count and
50-vs-60 is not a comparison. **Naming a number that cannot answer the question is how a coincidence
becomes a claim, so the hypothesis stays exactly where the DESIGN left it: untested.**

### Under 4-way self-contention, both sides, all eight runs rc 0

| side | `fill` | `total` | `fill-excess` |
|---|---|---|---|
| after ×4 | 5841 / 5786 / 5960 / 5795 | 34534 / 34121 / 34700 / 33771 | **40 / 0 / 0 / 40** |
| before ×4 | 5749 / 5724 / 5724 / 6244 | 33684 / 34343 / 33407 / 33578 | (field did not exist) |

`fill` 2.6 s → 5.8 s at 4-way on both sides; the poller is not what is slowing down, the box is.

---

## ROW 6 — THE THREE VERDICTS, ALL EXECUTED

```
""                (completion — unchanged in position; >= on visible, = on unacked)

filled-unread:    last={s} outbox={b} want={n} arrived={a} polls={p} elapsed={ms}
                  ← outranks both give-ups, tested first, as before

filled-stalled:   no arrival progress in 600 polls; last=[31/0][31/0] outbox=19 want=50
                  arrived=62 polls=602 elapsed=16499                       ← EXECUTED, rc 2

filled-timeout:   ceiling 1000ms reached with no 600-poll stall; last=[31/0][31/0] outbox=19
                  want=50 arrived=62 polls=47 elapsed=1010 stale=45 stale-max=45
                                                        ← EXECUTED, rc 2, ceiling temporarily 1 s
```

`filled-unread` is the one arm I did **not** execute — it needs an unreadable `Queue/stats` reply, which
this harness has no knob for. It is unchanged in position and structure; only its fields grew
(`want`/`arrived`/`polls` in place of `attempts`), and `format` arity is checked at load, which the
corpus gate exercises. **Stated as an untested arm rather than claimed.**

---

## THE EDIT SITES — the real count

`228 insertions, 32 deletions, 18 hunks, 1 file.` **Seven definitions, nine edit sites:**

| # | site | what |
|---|---|---|
| 1 | `:fanout::sweep-arrived` (`:1345`) | **new** — the progress signal, folded over the sweep already taken |
| 2 | `:fanout::sweep-excess` (`:1365`) | **new** — Σ max(visible − n, 0), the overshoot |
| 3 | `:fanout::sweep-filled?` (`:1413`) | `=` → `:wat::i64::>=` on the visible term; header rewritten |
| 4 | `:fanout::fill-stale-polls` (`:1465`) | **new** — K = 600 |
| 5 | `:fanout::fill-ceiling-ms` (`:1487`) | **new** — 30000 + 12 × pairs |
| 6 | `:fanout::poll-until-filled*` (`:1519`) | rewritten: `left`/`total` → `ceiling-ms`/`prog-prev`/`stale`/`stale-max`/`polls`; 2-tuple → 3-tuple; one verdict → three |
| 7 | `:fanout::poll-until-filled` (`:1568`) | `attempts` → `pairs`; seeds `prog-prev = -1` |
| 8 | `:fanout::run-with`, the fill-poll block (`:2832-2849`) | `(Tuple "" 0)` → `(Tuple "" 0 0)`; two new bindings `fill-stale-max`, `fill-excess` |
| 9 | `:fanout::run-with`, the report line (`:3066`, `:3074-3075`) | `fill-excess={fx};fill-stale-max={fsm}` + their two arguments |

**THE COMPILER WAS THE CENSUS, and I verified that rather than asserting it.** With site 8 left at the
old 2-tuple on a scratch copy, the checker refuses to start and names all four consequences:

```
#wat.check/CheckErrors {:message "4 type-check errors"
  :wat::core::if: parameter else-branch expects :(wat::core::String,wat::core::i64,wat::core::i64);
                  got :(wat::core::String,wat::core::i64)        ← line 2834
  :wat::core::first  … got :?16543                               ← line 2835
  :wat::core::second … got :?16545                               ← line 2836
  :wat::core::third  … got :?16547                               ← line 2840 }
```

Nothing outside `circuit.wat` was named, by the checker or by the corpus gate.

---

## WHAT I DID NOT TOUCH

`:cap`, `sub-cap`, `vis-ms`, `inbox-vis-ms`, `:max-entries`, the 5 ms poll wait, the 1 ms join timer,
`:fanout::require!`, `:fanout::drain-stale-polls`, `:fanout::drain-ceiling-ms`,
`:fanout::poll-until-drained*`, `:fanout::sweep-drained?`, `:fanout::sweep-unread?`,
`:fanout::snapshot-str`, `:fanout::sweep-acks`, `wat-scripts/queue/sqs.wat`,
`wat-scripts/topic/sns-fanout.wat`, `.config/nextest.toml`, anything under `src/` or `wat/`, and every
test file. No `.wat` was added to `wat-scripts/` — the two scratch artefacts (HEAD's `circuit.wat` for
the A/B, and the arity probe) live outside the repo because they are *copies of a live program*, not
scratch programs, and putting a stale duplicate of `circuit.wat` under `wat-scripts/` would have made
the corpus gate a graveyard rather than a gate.

---

## RUNTIME

Predicted 2–3 hours. Actual ≈ 2.5 h wall, of which ~35 min was the two floors and ~25 min the timing
batteries. The predicted risk (rows 1 and 14) was correct in shape and cheaper than expected: row 14
resolved from four `sqs.wat` sites in one read, and row 1's witness took one guess — `sub-cap < n`
with `fill-first? = true` — after `j = 0` was reasoned out as *not* the fill equivalent. **The
unbudgeted cost was the audit**, which found three sites instead of zero and produced a captured red.

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

**Floor** `.floor/2026-09-10T17-45-28Z/clean.log`: `Summary [ 483.727s] 5237 tests run: 5237 passed
(7 slow), 22 skipped` — 0 failure tokens, no `ARM.txt`. **Blast radius:** `circuit.wat` + the SCORE.

**Row 1, run by me — the row I was most afraid of:**

```
circuit.wat 50 2 2 32 true 0   →  rc 2
filled-stalled: no arrival progress in 600 polls; last=[31/0][31/0] outbox=19
                want=50 arrived=62 polls=602 elapsed=14277
```

**The check can still go red.** Row 3 verified too — `fill-excess` is a reported field, reading 0 when
there is none.

⚠ **I did NOT reproduce the overshoot.** Four concurrent runs gave `fill-depth=[2000/0]`,
`fill-excess=0`, rc 0 on all four. The executor saw it 2 of 4. **It is intermittent, and their
observation stands as theirs, not as something I confirmed.**

## ⭑⭑ ROW 14 HELD — and for a better reason than my design offered

I feared *"a redelivery re-incrementing `visible` without incrementing anything else."* The executor
showed that **cannot happen**: `depth` (`sqs.wat:339-381`) issues two `count-index` calls over one
partition and returns `visible = |[0,now]|`, `unacked = |[0,+inf)| − visible`. So **`visible + unacked`
*is* the row count, split at `now`** — a redelivery or a claim moves a row **across the split**, and the
sum cannot change or double-count.

★ That is stronger than verification: **the risk I named is structurally impossible**, with the reason.
And two plateaus were found and both are correct behaviour — `count-index` saturates at `cap+1` (but a
queue at cap refuses sends, which *is* a stall), and a counter reset reads as no progress.

## ⛔ STOP-1's ANSWER CORRECTED MY BRIEF

I wrote *"the drain stone got its witness from `j=0`; find the fill equivalent."* **`j=0` is not the fill
equivalent** — with nothing consuming, a fill *succeeds*. The equivalent is a queue that **cannot hold
what is wanted**: `sub-cap < n` with `fill-first? = true`. My instruction pointed at the wrong shape and
the executor found the right one.

★ They also witnessed the **ceiling** arm by temporarily shrinking it, then restored the file and
**md5-verified** it. And `filled-unread` is reported as **not executed** — no knob produces an unreadable
stats reply — rather than quietly counted as covered.

## ⭑ K and CEILING: measured, and the measurement came back uninformative — said plainly

`fill-stale-max = 0` on all ten passing runs, **but over only ~3.6 polls per run**, because the inbox
cap of 64 backpressures publishing against fan-out, so the poller finds the queues already full. So the
constant could not be chosen from data, and K instead rests on the longest **legitimate** silence
(inbox visibility + receive wait ≈ 1.25 s) against a measured **27.4 ms/poll** → K ≈ 16.5 s, 13× that
gap.

★★ **Which part is measured and which is reasoned is labelled.** That is the discipline the drain stone
paid for — it measured `drain-stale-max=197` on a *passing* run against a first guess of 200, three polls
from a false red — and it is rarer than it should be to see a null measurement reported as null instead of
dressed up.

## ⛔⛔ STOP-3 FIRED: THREE MORE, AND I SAID "ONE OF TWO"

Two negatives reported, which is the part that stops this recurring:
**`sweep-unread?` — fine** (`-1` is a whole-tuple sentinel, so the one-slot test misses nothing).
**`snapshot-str` — fine** (symmetric across both pollers).

And three positives:

- ⛔ **`sweep-drained?`** — `unacked == 0`, the same pinnable equality. ★★ **And `nextest.toml` already
  cites `FINDING-unacked-is-receives-minus-acks.md`** — this was already written down and never fixed.
- ⛔ **`poll-until-visible-zero*` (`:2230`)** — **a THIRD poller.** 4000 attempts, one verdict, **and it
  is in the floor.**
- ⛔ **`join-publishers*` (`:2204`)** — **120 000 attempts**, and its give-up carries **no numbers at
  all.**

★★★ **I named this "we fixed one of two pollers." There are three, plus a 120 000-attempt join.** The
class now stands at **four live sites**, one of them already documented in a `FINDING` the test config
points at. **The executor fixed nothing outside the fill poller**, which is correct: at four sites this
wants a gate, not four repairs.

## ⛔ A CAPTURED RED IN CODE THE EXECUTOR DID NOT TOUCH

```
drained-stalled … last=[0/0][0/10][0/0][0/0] outbox=0 acks=8050 polls=668 elapsed=24572
circuit.wat:1661   ← poll-until-drained*'s STALL arm (not the ceiling; 24.6 s is under the 30 s floor)
```

**Ten rows claimed and never acked** — audit item (i), `sweep-drained?`'s pinnable equality, firing.
Sample 1/2 after-side, 0/2 before-side: **too small to attribute, and the executor does not attribute
it.** One route is stated as a hypothesis and labelled: `>=` lets an overshooting run *reach* the drain,
where that equality can bite. **Not re-run, not dispositioned.**

## Row 15 honoured, with a lead handed over as data

`fill` 2610.3 → 2607.7 ms (**−0.10 %**). `total` moved −1.68 % and the executor **declined to bank it** —
the 394 ms lives in `collect` and `stop`, whose own spreads (657 / 500 ms) exceed the difference between
the means. ★ Declining a favourable number is the rarer discipline.

★ And one new lead, offered as data rather than diagnosis: **`inbox-vis-ms=50` gave `[2020/0]×4`,
`fill-excess=80`.** So the overshoot **scales with a shorter inbox visibility** — the first mechanical
handle on defect (a) that anyone has found.

## Grade

`1 ✅ (witnessed, and by me) · 2 ✅ · 3 ✅ · 4 ✅ · 5 ✅ · 6 ✅ (3 arms executed, `filled-unread`
declared untested) · 7 ✅ · 8 ✅✅ (audit reported, negatives included, **STOP-3 fired**) · 9 ✅ · 10 ✅ ·
11 ✅ · 12 ✅ · 13 ✅ · 14 ✅ **held, structurally** · 15 ✅ honoured`

## The ruling now owed

**Four live sites of one class: a give-up bounded by attempts rather than progress, with a verdict that
cannot distinguish "the system stopped" from "I ran out of budget" — and in two cases an equality the
system can pin off-target.** One is already named in a `FINDING` that `nextest.toml` cites.

★ Repairing them one at a time is what produced this: two pollers fixed, a third undiscovered until
someone looked. **The gateable property is one sentence** — *a give-up path must be bounded by progress
or by wall clock, never by an attempt count, and must name which bound it hit* — and it is the builder's
to open.
